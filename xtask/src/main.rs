//! `cargo xtask <command>`; see `cargo xtask help`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context as _, Result, bail};
use xtask::cross::{self, Context, Invocation, Options};
use xtask::package::{self, App, Entry, ManifestValues};
use xtask::targets::Target;
use xtask::{pinned_toolchain, policy, symcheck, workspace_root};

const HELP: &str = "\
cargo xtask <command>

Commands:
  cross          cross-build for a Tizen target (see `cargo xtask cross --help`)
  check-symbols  check a built TV library's dynamic symbols: <lib.so> [--entry <symbol>]
  diag pull      fetch the diagnostic app's report through `sdb forward`: [--port <p>] [--no-forward]
  package        stage (and with --sign, sign) a TPK: --app <diag|counter|long_list>
                 --target <armv7|x86_64|aarch64> [--sign <profile>]
  tizen-deps     check the Tizen dependency graph: --target <armv7|x86_64|aarch64>
  policy         check the workspace pins, lints, release profile, alias and toolchain
  help           show this text
";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<ExitCode> {
    let Some((command, rest)) = args.split_first() else {
        print!("{HELP}");
        return Ok(ExitCode::SUCCESS);
    };
    match command.as_str() {
        "cross" => {
            if rest.iter().any(|a| a == "--help" || a == "-h") {
                println!("{}", cross::USAGE);
                return Ok(ExitCode::SUCCESS);
            }
            run_cross(&Options::parse(rest)?)
        }
        "check-symbols" => {
            let (library, entry) = match rest {
                [library] => (library, symcheck::DEFAULT_ENTRY),
                [library, flag, entry] if flag == "--entry" => (library, entry.as_str()),
                _ => bail!("usage: cargo xtask check-symbols <lib.so> [--entry <symbol>]"),
            };
            let toolchain = pinned_toolchain(&workspace_root())?;
            let ok = check_symbols(&rustlib_bin(&toolchain)?, Path::new(library), entry)?;
            Ok(if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
        "diag" => run_diag(rest),
        "package" => run_package(rest),
        "tizen-deps" => run_tizen_deps(rest),
        "policy" => {
            let errors = policy::check(&policy::Inputs::read(&workspace_root())?);
            for error in &errors {
                eprintln!("{error}");
            }
            Ok(if errors.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
        "help" | "--help" | "-h" => {
            print!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
        other => bail!("unknown command `{other}`; run `cargo xtask help`"),
    }
}

fn run_cross(options: &Options) -> Result<ExitCode> {
    Ok(match cross_build(options)? {
        Some(_) => ExitCode::SUCCESS,
        None => ExitCode::FAILURE,
    })
}

/// Runs a cross build (and, with `--check-symbols`, the symbol check). Returns the selected
/// `cdylib`s it built, or `None` when the build or a check failed.
fn cross_build(options: &Options) -> Result<Option<Vec<PathBuf>>> {
    let root = workspace_root();
    let toolchain = pinned_toolchain(&root)?;
    let own_metadata = cargo_metadata(&toolchain, None, None)?;
    let tv_libs = cross::cdylib_packages(&own_metadata);
    // A workspace outside this repository keeps its lockfile under `artifacts-temp/locks/`, seeded
    // from its committed `Cargo.lock`, so a build never rewrites a file in another checkout.
    let lockfile = match options.manifest_path() {
        Some(path) => {
            let manifest = std::fs::canonicalize(path)
                .with_context(|| format!("resolving --manifest-path {path}"))?;
            let workspace =
                cross::workspace_root_of(&manifest, |p| std::fs::read_to_string(p).ok());
            match cross::foreign_lockfile(&root, &workspace) {
                Some(kept) => {
                    std::fs::create_dir_all(kept.parent().unwrap_or(&root))?;
                    let committed = workspace.join("Cargo.lock");
                    if committed.is_file() {
                        std::fs::copy(&committed, &kept)
                            .with_context(|| format!("copying {}", committed.display()))?;
                    }
                    Some(kept)
                }
                None => None,
            }
        }
        None => None,
    };
    let metadata = match options.manifest_path() {
        Some(path) => cargo_metadata(&toolchain, Some(Path::new(path)), lockfile.as_deref())?,
        None => own_metadata,
    };
    let target = options.target;
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let searched = cross::rootstrap_candidates(
        target,
        std::env::var(target.rootstrap_env()).ok().as_deref(),
        std::env::var("TIZEN_SDK").ok().as_deref(),
        home.as_deref(),
    );
    let gcc_searched = xtask::gcc::candidates(
        target,
        std::env::var(xtask::gcc::env(target)).ok().as_deref(),
        std::env::var("TIZEN_SDK").ok().as_deref(),
        home.as_deref(),
    );
    let ctx = Context {
        gcc_toolchain: gcc_searched.iter().find(|p| p.is_dir()).cloned(),
        gcc_toolchain_searched: gcc_searched,
        rustlib_bin: rustlib_bin(&toolchain)?,
        clang: std::env::var("TIZEN_CLANG").unwrap_or_else(|_| "clang".into()),
        rootstrap: searched.iter().find(|p| p.is_dir()).cloned(),
        rootstrap_searched: searched,
        needs_link: cross::selection_needs_link(&metadata, options, &tv_libs),
        lockfile,
        inherited_rustflags: cross::inherited_rustflags(
            std::env::var("CARGO_ENCODED_RUSTFLAGS").ok().as_deref(),
            std::env::var("RUSTFLAGS").ok().as_deref(),
        ),
        root,
        toolchain,
    };
    let invocation = cross::plan(options, &ctx, &tv_libs)?;
    let status = execute(&invocation)?;
    if !status {
        return Ok(None);
    }
    let target_dir = metadata["target_directory"]
        .as_str()
        .map(PathBuf::from)
        .context("cargo metadata has no target_directory")?;
    let libraries = cross::built_libraries(
        &target_dir,
        options.target,
        options.release,
        &cross::selected_cdylibs(&metadata, options, &tv_libs),
    );
    if options.check_symbols {
        if libraries.is_empty() {
            bail!("`--check-symbols`: the selected packages build no cdylib");
        }
        let entry = options.entry.as_deref().unwrap_or(symcheck::DEFAULT_ENTRY);
        let mut ok = true;
        for library in &libraries {
            ok &= check_symbols(&ctx.rustlib_bin, library, entry)?;
        }
        if !ok {
            return Ok(None);
        }
    }
    Ok(Some(libraries))
}

fn run_tizen_deps(args: &[String]) -> Result<ExitCode> {
    use xtask::tizen_deps;
    let target = match args {
        [flag, target] if flag == "--target" => Target::parse(target)?,
        _ => bail!("usage: cargo xtask tizen-deps --target <armv7|x86_64|aarch64>"),
    };
    let root = workspace_root();
    let toolchain = pinned_toolchain(&root)?;
    let tree = |package: Option<&str>| -> Result<String> {
        let mut command = Command::new("cargo");
        command
            .env("RUSTUP_TOOLCHAIN", &toolchain)
            .current_dir(&root)
            .args(["tree", "-Zjson-target-spec", "--target"])
            .arg(target.spec_path(&root))
            .args(["-e", "normal", "--prefix", "none", "--format", "{p}"]);
        match package {
            Some(package) => command.args(["-p", package]),
            None => command.arg("--workspace"),
        };
        let output = command.output().context("running cargo tree")?;
        if !output.status.success() {
            bail!(
                "cargo tree failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    };
    let workspace = tizen_deps::parse_tree(&tree(None)?);
    let mut violations = tizen_deps::desktop_crates(&workspace);
    let wgpu = match tizen_deps::single_wgpu(&workspace) {
        Ok(version) => Some(version),
        Err(violation) => {
            violations.push(violation);
            None
        }
    };
    let diag = tizen_deps::parse_tree(&tree(Some("dxn_gpui_tizen_diag"))?);
    violations.extend(tizen_deps::diag_violations(&diag, wgpu.as_deref()));
    for violation in &violations {
        eprintln!("{}: {violation}", target.spec_name());
    }
    if violations.is_empty() {
        println!(
            "{}: {}; one wgpu ({}), shared by the diagnostic app, which has no GPUI crate",
            target.spec_name(),
            tizen_deps::OK_LINE,
            wgpu.as_deref().unwrap_or("-")
        );
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

/// `sdb` on `PATH`, else in the Tizen SDK's `tools/`.
fn find_sdb() -> Option<PathBuf> {
    let on_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join("sdb"))
            .find(|p| p.is_file())
    });
    on_path.or_else(|| {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        xtask::targets::sdk_root(std::env::var("TIZEN_SDK").ok().as_deref(), home.as_deref())
            .map(|sdk| sdk.join("tools/sdb"))
            .filter(|p| p.is_file())
    })
}

fn run_diag(args: &[String]) -> Result<ExitCode> {
    use dxn_gpui_tizen_diag::transport;
    let no_forward = args.iter().any(|a| a == "--no-forward");
    let args: Vec<&String> = args.iter().filter(|a| *a != "--no-forward").collect();
    let port = match args.as_slice() {
        [pull] if *pull == "pull" => transport::DEFAULT_PORT,
        [pull, flag, port] if *pull == "pull" && *flag == "--port" => {
            port.parse().context("--port")?
        }
        _ => bail!("usage: cargo xtask diag pull [--port <p>] [--no-forward]"),
    };
    if !no_forward {
        let sdb = find_sdb().context(
            "`sdb` not found on PATH or in the Tizen SDK (`<SDK>/tools/sdb`); install the SDK or pass --no-forward",
        )?;
        let status = Command::new(&sdb)
            .args(transport::sdb_forward_args(port))
            .status()
            .with_context(|| format!("running {}", sdb.display()))?;
        if !status.success() {
            bail!("`sdb forward tcp:{port} tcp:{port}` failed; is a TV connected (`sdb devices`)?");
        }
    }
    let pulled = transport::pull(("127.0.0.1", port), std::time::Duration::from_secs(180))
        .with_context(|| format!("connecting to 127.0.0.1:{port}; is `sdb forward tcp:{port} tcp:{port}` set up and the diag app running?"))?;
    let dir = workspace_root().join("artifacts-temp/diag");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(transport::report_file_name(
        &pulled.report,
        std::time::SystemTime::now(),
    ));
    let mut contents = pulled.json.clone();
    contents.push('\n');
    std::fs::write(&path, contents)?;
    let log_path = path.with_extension("log");
    std::fs::write(&log_path, pulled.log.join("\n") + "\n")?;
    print!("{}", transport::summary(&pulled.report));
    eprintln!("saved {} and {}", path.display(), log_path.display());
    Ok(ExitCode::SUCCESS)
}

fn run_package(args: &[String]) -> Result<ExitCode> {
    let (mut app, mut target, mut profile) = (None, None, None);
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let mut value = || {
            iter.next()
                .cloned()
                .with_context(|| format!("`{arg}` needs a value"))
        };
        match arg.as_str() {
            "--app" => app = Some(App::find(&value()?)?),
            "--target" => target = Some(Target::parse(&value()?)?),
            "--sign" => profile = Some(value()?),
            other => bail!("unknown `cargo xtask package` argument `{other}`"),
        }
    }
    let (Some(app), Some(target)) = (app, target) else {
        bail!(
            "usage: cargo xtask package --app <diag|counter|long_list> --target <armv7|x86_64|aarch64> [--sign <profile>]"
        );
    };
    let root = workspace_root();

    // The app's release library, through the cross driver (with the symbol check).
    let mut cross_args = vec![
        "--target".to_owned(),
        target.spec_name().to_owned(),
        "--release".to_owned(),
    ];
    cross_args.extend([
        "--check-symbols".to_owned(),
        "-p".to_owned(),
        app.package.to_owned(),
    ]);
    // Stage exactly the library the cross build produced and the symbol check passed.
    let Some(mut libraries) = cross_build(&Options::parse(&cross_args)?)? else {
        bail!("building {} for {} failed", app.package, target.spec_name());
    };
    let library = libraries
        .pop()
        .context("the cross build produced no library")?;

    // The staged tree.
    let manifest_toml: toml::Value = std::fs::read_to_string(root.join("Cargo.toml"))?.parse()?;
    let version = manifest_toml["workspace"]["package"]["version"]
        .as_str()
        .context("[workspace.package] has no version")?;
    let template = std::fs::read_to_string(root.join("host/tizen-manifest.template.xml"))?;
    let manifest = package::render_manifest(&template, &ManifestValues::for_app(app, version))?;
    let staging = package::staging_dir(&root, app, target);
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let rendered_manifest = staging.join("tizen-manifest.xml");
    std::fs::write(&rendered_manifest, &manifest)?;

    // The host, given the staged library and the rendered manifest (the SDK's own TPK step stays
    // off; with `-p:TizenCreateTpkOnBuild=true` it would package the same inputs).
    let csproj = root.join("host/Dxn.GpuiTizen.Host.csproj");
    let status = Command::new("dotnet")
        .args(["build", "-c", "Release"])
        .arg(&csproj)
        .arg(format!("-p:DxnAppLibrary={}", library.display()))
        .arg(format!(
            "-p:TizenManifestFile={}",
            rendered_manifest.display()
        ))
        .status()
        .context("running dotnet build (install the .NET SDK)")?;
    if !status.success() {
        bail!("dotnet build of the host failed");
    }
    let entries = package::layout(
        &package::host_files(&root.join("host/bin/Release/tizen80")),
        &library,
        &root.join("host/shared/res/icon.png"),
        manifest.clone(),
    );
    for entry in entries {
        match entry {
            Entry::Copy { from, to } => {
                let to = staging.join(to);
                std::fs::create_dir_all(to.parent().unwrap_or(&staging))?;
                std::fs::copy(&from, &to).with_context(|| format!("copying {}", from.display()))?;
            }
            Entry::Dir(dir) => std::fs::create_dir_all(staging.join(dir))?,
            Entry::Write { to, contents } => {
                let to = staging.join(to);
                std::fs::create_dir_all(to.parent().unwrap_or(&staging))?;
                std::fs::write(to, contents)?;
            }
        }
    }
    eprintln!("staged {}", staging.display());

    match profile {
        Some(profile) => {
            let command = package::sign_command(&staging, &profile);
            let status = Command::new(&command[0])
                .args(&command[1..])
                .status()
                .context("running `tz` (the Tizen SDK CLI)")?;
            Ok(if status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
        None => {
            eprintln!("{}", package::UNSIGNED_NOTE);
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Runs the pinned toolchain's `llvm-nm` on a library and reports every policy violation.
fn check_symbols(rustlib_bin: &Path, library: &Path, entry: &str) -> Result<bool> {
    let nm = rustlib_bin.join("llvm-nm");
    let run = |only: &str| -> Result<String> {
        let output = Command::new(&nm)
            .args(["-D", only])
            .arg(library)
            .output()
            .with_context(|| {
                format!(
                    "running {} (install the `llvm-tools` component)",
                    nm.display()
                )
            })?;
        if !output.status.success() {
            bail!(
                "llvm-nm failed on {}: {}",
                library.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    };
    let readobj = rustlib_bin.join("llvm-readobj");
    let needed = Command::new(&readobj)
        .arg("--needed-libs")
        .arg(library)
        .output()
        .with_context(|| format!("running {}", readobj.display()))?;
    if !needed.status.success() {
        bail!(
            "llvm-readobj failed on {}: {}",
            library.display(),
            String::from_utf8_lossy(&needed.stderr)
        );
    }
    let violations = symcheck::check_library(
        &run("--defined-only")?,
        &run("--undefined-only")?,
        &String::from_utf8_lossy(&needed.stdout),
        entry,
    );
    for violation in &violations {
        eprintln!("{}: {violation}", library.display());
    }
    if violations.is_empty() {
        eprintln!("{}: symbols ok (exports only `{entry}`)", library.display());
    }
    Ok(violations.is_empty())
}

fn execute(invocation: &Invocation) -> Result<bool> {
    eprintln!("+ {} {}", invocation.program, invocation.args.join(" "));
    let mut command = Command::new(&invocation.program);
    for key in &invocation.env_remove {
        command.env_remove(key);
    }
    let status = command
        .args(&invocation.args)
        .envs(invocation.env.iter().map(|(k, v)| (k, v)))
        .status()
        .with_context(|| format!("running {}", invocation.program))?;
    Ok(status.success())
}

fn cargo_metadata(
    toolchain: &str,
    manifest: Option<&Path>,
    lockfile: Option<&Path>,
) -> Result<serde_json::Value> {
    let mut command = Command::new("cargo");
    command.env("RUSTUP_TOOLCHAIN", toolchain).args([
        "metadata",
        "--no-deps",
        "--format-version",
        "1",
    ]);
    if let Some(manifest) = manifest {
        command.arg("--manifest-path").arg(manifest);
    } else {
        command
            .arg("--manifest-path")
            .arg(workspace_root().join("Cargo.toml"));
    }
    if let Some(lockfile) = lockfile {
        command.args(cross::lockfile_config(lockfile));
    }
    let output = command.output().context("running cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

/// The pinned toolchain's `lib/rustlib/<host>/bin`.
fn rustlib_bin(toolchain: &str) -> Result<PathBuf> {
    let rustc = |args: &[&str]| -> Result<String> {
        let output = Command::new("rustc")
            .env("RUSTUP_TOOLCHAIN", toolchain)
            .args(args)
            .output()
            .context("running rustc")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    let sysroot = rustc(&["--print", "sysroot"])?;
    let host = rustc(&["-vV"])?
        .lines()
        .find_map(|l| l.strip_prefix("host: ").map(str::to_owned))
        .context("rustc -vV has no host line")?;
    Ok(Path::new(&sysroot)
        .join("lib/rustlib")
        .join(host)
        .join("bin"))
}
