//! `cargo xtask <command>`; see `cargo xtask help`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context as _, Result, bail};
use xtask::cross::{self, Context, Invocation, Options};
use xtask::{pinned_toolchain, policy, workspace_root};

const HELP: &str = "\
cargo xtask <command>

Commands:
  cross     cross-build for a Tizen target (see `cargo xtask cross --help`)
  policy    check the workspace pins, lints, release profile, alias and toolchain
  help      show this text
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
    let root = workspace_root();
    let toolchain = pinned_toolchain(&root)?;
    let own_metadata = cargo_metadata(&toolchain, None)?;
    let tv_libs = cross::cdylib_packages(&own_metadata);
    let metadata = match options.manifest_path() {
        Some(path) => cargo_metadata(&toolchain, Some(Path::new(path)))?,
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
    let ctx = Context {
        rustlib_bin: rustlib_bin(&toolchain)?,
        clang: std::env::var("TIZEN_CLANG").unwrap_or_else(|_| "clang".into()),
        rootstrap: searched.iter().find(|p| p.is_dir()).cloned(),
        rootstrap_searched: searched,
        needs_link: cross::selection_needs_link(&metadata, options, &tv_libs),
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
        return Ok(ExitCode::FAILURE);
    }
    Ok(ExitCode::SUCCESS)
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

fn cargo_metadata(toolchain: &str, manifest: Option<&Path>) -> Result<serde_json::Value> {
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
