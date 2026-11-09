//! `cargo xtask cross`: Cargo on the pinned nightly with `-Zbuild-std`, a Tizen target spec, clang
//! for C dependencies, and clang driving the Tizen SDK's GNU `ld.bfd` for linking against the
//! Tizen rootstrap (see [`crate::gcc`]; lld only when no SDK toolchain is installed, for builds that
//! never link).
//!
//! Everything here is a pure function of the parsed options and a [`Context`]; `main.rs` gathers
//! the context (rootstrap, toolchain paths, `cargo metadata`) and runs the [`Invocation`].

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde_json::Value;

use crate::targets::Target;

/// Parsed `cargo xtask cross` options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub target: Target,
    pub release: bool,
    pub clippy: bool,
    pub check_symbols: bool,
    /// The entry symbol `--check-symbols` expects (`--entry`, default `dxn_gpui_tizen_main`).
    pub entry: Option<String>,
    pub all_tv_libs: bool,
    /// Cargo selection and feature flags, passed through unchanged (`-p`, `--manifest-path`,
    /// `--features`, `--no-default-features`, ...).
    pub cargo_args: Vec<String>,
}

pub const USAGE: &str = "\
cargo xtask cross --target <armv7|x86_64|aarch64> [--release] [--clippy] [--check-symbols [--entry <symbol>]]
                  [--all-tv-libs]
                  [-p <package>]... [--manifest-path <Cargo.toml>] [--features <f>] [--no-default-features]
                  [other cargo flags]";

impl Options {
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut target = None;
        let mut options = Options {
            target: Target::Armv7,
            release: false,
            clippy: false,
            check_symbols: false,
            entry: None,
            all_tv_libs: false,
            cargo_args: Vec::new(),
        };
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--target" => {
                    let Some(name) = args.next() else {
                        bail!("`--target` needs a value\n{USAGE}");
                    };
                    target = Some(Target::parse(name)?);
                }
                s if s.starts_with("--target=") => target = Some(Target::parse(&s[9..])?),
                "--release" => options.release = true,
                "--clippy" => options.clippy = true,
                "--check-symbols" => options.check_symbols = true,
                "--entry" => {
                    let Some(name) = args.next() else {
                        bail!("`--entry` needs a value\n{USAGE}");
                    };
                    options.entry = Some(name.clone());
                }
                "--all-tv-libs" => options.all_tv_libs = true,
                _ => options.cargo_args.push(arg.clone()),
            }
        }
        let Some(target) = target else {
            bail!("missing `--target`\n{USAGE}");
        };
        options.target = target;
        Ok(options)
    }

    /// The `--manifest-path` passed through, if any.
    pub fn manifest_path(&self) -> Option<&str> {
        flag_values(&self.cargo_args, None, "--manifest-path")
            .into_iter()
            .next()
    }

    /// Packages selected with `-p` / `--package`.
    pub fn packages(&self) -> Vec<&str> {
        flag_values(&self.cargo_args, Some("-p"), "--package")
    }

    pub fn workspace(&self) -> bool {
        self.cargo_args
            .iter()
            .any(|a| a == "--workspace" || a == "--all")
    }
}

/// Values of a flag in its `-p x`, `-px`, `--package x` and `--package=x` spellings.
fn flag_values<'a>(args: &'a [String], short: Option<&str>, long: &str) -> Vec<&'a str> {
    let mut values = Vec::new();
    let mut iter = args.iter().take_while(|a| *a != "--");
    while let Some(arg) = iter.next() {
        if Some(arg.as_str()) == short || arg == long {
            if let Some(value) = iter.next() {
                values.push(value.as_str());
            }
        } else if let Some(value) = arg.strip_prefix(long).and_then(|r| r.strip_prefix('=')) {
            values.push(value);
        } else if let Some(value) = short
            .and_then(|s| arg.strip_prefix(s))
            .filter(|v| !v.is_empty())
        {
            values.push(value);
        }
    }
    values
}

/// What `main.rs` found on this machine.
#[derive(Clone, Debug)]
pub struct Context {
    /// This repository's root (target specs live here).
    pub root: PathBuf,
    /// The pinned nightly from `rust-toolchain.toml`, used whatever workspace is built.
    pub toolchain: String,
    /// The pinned toolchain's `lib/rustlib/<host>/bin` (llvm-ar, rust-lld and `gcc-ld/`).
    pub rustlib_bin: PathBuf,
    /// The C compiler (`clang`, or `TIZEN_CLANG`).
    pub clang: String,
    /// The target's rootstrap, when one exists.
    pub rootstrap: Option<PathBuf>,
    /// Where the rootstrap was looked for, for the error message.
    pub rootstrap_searched: Vec<PathBuf>,
    /// The Tizen SDK's GCC toolchain for the target (start files and `libgcc_s`), when one exists.
    pub gcc_toolchain: Option<PathBuf>,
    /// Where the GCC toolchain was looked for, for the error message.
    pub gcc_toolchain_searched: Vec<PathBuf>,
    /// Whether the selected packages produce a linked artifact (a `cdylib`).
    pub needs_link: bool,
    /// Flags the caller's environment already passes to rustc (`CARGO_ENCODED_RUSTFLAGS`, else
    /// `RUSTFLAGS`). Cargo reads only one flag source, so they are merged with the link arguments.
    pub inherited_rustflags: Vec<String>,
    /// For a workspace outside this repository: where its lockfile is kept, so Cargo never
    /// rewrites a lockfile in another checkout (see [`foreign_lockfile`]).
    pub lockfile: Option<PathBuf>,
}

/// The workspace root that owns `manifest`: the nearest ancestor directory (the manifest's own
/// included) whose `Cargo.toml` declares `[workspace]`, else the manifest's directory.
pub fn workspace_root_of(manifest: &Path, read: impl Fn(&Path) -> Option<String>) -> PathBuf {
    let start = manifest.parent().unwrap_or(Path::new("."));
    start
        .ancestors()
        .find(|dir| {
            read(&dir.join("Cargo.toml"))
                .is_some_and(|text| text.lines().any(|line| line.trim() == "[workspace]"))
        })
        .unwrap_or(start)
        .to_path_buf()
}

/// Where a foreign workspace's lockfile is kept: `artifacts-temp/locks/<workspace path>/Cargo.lock`
/// under this repository. Returns `None` for this repository's own workspace. The caller seeds it
/// from the workspace's committed `Cargo.lock` before each build.
pub fn foreign_lockfile(root: &Path, workspace: &Path) -> Option<PathBuf> {
    if workspace.starts_with(root) {
        return None;
    }
    let name: String = workspace
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("_");
    Some(
        root.join("artifacts-temp/locks")
            .join(name)
            .join("Cargo.lock"),
    )
}

/// `--config resolver.lockfile-path=...` for a kept lockfile.
pub fn lockfile_config(lockfile: &Path) -> [String; 2] {
    [
        "--config".to_owned(),
        format!("resolver.lockfile-path='{}'", lockfile.display()),
    ]
}

/// The caller's rustc flags from `CARGO_ENCODED_RUSTFLAGS` (unit-separated) or `RUSTFLAGS`
/// (whitespace-separated), in Cargo's precedence order.
pub fn inherited_rustflags(encoded: Option<&str>, plain: Option<&str>) -> Vec<String> {
    match (encoded, plain) {
        (Some(encoded), _) if !encoded.is_empty() => {
            encoded.split('\x1f').map(str::to_owned).collect()
        }
        (_, Some(plain)) => plain.split_whitespace().map(str::to_owned).collect(),
        _ => Vec::new(),
    }
}

/// A process to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    /// Variables removed from the inherited environment.
    pub env_remove: Vec<String>,
}

impl Invocation {
    pub fn env_value(&self, key: &str) -> Option<&str> {
        self.env
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// The rootstrap candidates for a target, in order: the environment variable, then the Tizen
/// SDK's default location (`TIZEN_SDK`, else `~/tizen-studio`).
pub fn rootstrap_candidates(
    target: Target,
    env_value: Option<&str>,
    tizen_sdk: Option<&str>,
    home: Option<&Path>,
) -> Vec<PathBuf> {
    if let Some(value) = env_value.filter(|v| !v.is_empty()) {
        return vec![PathBuf::from(value)];
    }
    crate::targets::sdk_root(tizen_sdk, home)
        .map(|sdk| vec![target.sdk_rootstrap(&sdk)])
        .unwrap_or_default()
}

/// The error for a linking build without a rootstrap.
pub fn missing_rootstrap(target: Target, searched: &[PathBuf]) -> String {
    let searched: Vec<String> = searched.iter().map(|p| p.display().to_string()).collect();
    format!(
        "linking for {} needs the Tizen 8.0 common rootstrap `{}`, which was not found (looked in: {}).\n\
         Install it with the Tizen SDK's package manager (`package-manager-cli install \
         TIZEN-8.0-NativeAppDevelopment-CLI`), or point `{}` at an extracted rootstrap.",
        target.spec_name(),
        target.sdk_rootstrap_name(),
        if searched.is_empty() {
            "nowhere".to_owned()
        } else {
            searched.join(", ")
        },
        target.rootstrap_env(),
    )
}

/// Builds the Cargo invocation, or fails early when linking is needed and no rootstrap exists.
pub fn plan(options: &Options, ctx: &Context, tv_libs: &[String]) -> Result<Invocation> {
    let target = options.target;
    if options.all_tv_libs && options.manifest_path().is_some() {
        bail!(
            "`--all-tv-libs` selects this workspace's TV libraries; it cannot be combined with `--manifest-path`"
        );
    }
    if options.check_symbols && options.clippy {
        bail!("`--check-symbols` checks built libraries; it cannot be combined with `--clippy`");
    }
    let links = ctx.needs_link && !options.clippy;
    if links && ctx.rootstrap.is_none() {
        bail!(missing_rootstrap(target, &ctx.rootstrap_searched));
    }
    if links && ctx.gcc_toolchain.is_none() {
        bail!(crate::gcc::missing(target, &ctx.gcc_toolchain_searched));
    }

    let mut args = vec![
        if options.clippy { "clippy" } else { "build" }.to_owned(),
        "-Zbuild-std=std,panic_abort".to_owned(),
        "-Zjson-target-spec".to_owned(),
        "--target".to_owned(),
        target.spec_path(&ctx.root).display().to_string(),
    ];
    if options.release {
        args.push("--release".to_owned());
    }
    if let Some(lockfile) = &ctx.lockfile {
        args.extend(lockfile_config(lockfile));
    }
    let (cargo_args, trailing) = match options.cargo_args.iter().position(|a| a == "--") {
        Some(i) => (&options.cargo_args[..i], &options.cargo_args[i + 1..]),
        None => (&options.cargo_args[..], &[][..]),
    };
    args.extend(cargo_args.iter().cloned());
    if options.all_tv_libs {
        for package in tv_libs {
            args.push("-p".to_owned());
            args.push(package.clone());
        }
    }
    // The libraries to check are read from Cargo's messages (`crate::artifacts`).
    if options.check_symbols {
        args.push("--message-format=json-render-diagnostics".to_owned());
    }
    if options.clippy || !trailing.is_empty() {
        args.push("--".to_owned());
        args.extend(trailing.iter().cloned());
        if options.clippy {
            args.extend(["-D".to_owned(), "warnings".to_owned()]);
        }
    }

    let suffix = target.env_suffix();
    let triple = target.clang_triple();
    let sysroot = ctx
        .rootstrap
        .as_ref()
        .map(|p| format!("--sysroot={}", p.display()));
    let mut c_flags = vec![format!("--target={triple}")];
    c_flags.extend(sysroot.clone());
    c_flags.extend(target.c_flags().iter().map(|f| (*f).to_owned()));

    let mut env = vec![
        ("RUSTUP_TOOLCHAIN".to_owned(), ctx.toolchain.clone()),
        (format!("CC_{suffix}"), ctx.clang.clone()),
        (format!("CFLAGS_{suffix}"), c_flags.join(" ")),
        (
            format!("AR_{suffix}"),
            ctx.rustlib_bin.join("llvm-ar").display().to_string(),
        ),
        (
            format!("BINDGEN_EXTRA_CLANG_ARGS_{suffix}"),
            c_flags.join(" "),
        ),
        (target.cargo_env("LINKER"), ctx.clang.clone()),
    ];
    let mut link_args = vec![format!("--target={triple}")];
    link_args.extend(sysroot);
    match &ctx.gcc_toolchain {
        // Linking against the rootstrap: the SDK's GNU ld (see `gcc::linker` for why not lld).
        Some(gcc_toolchain) => link_args.extend(crate::gcc::link_args(target, gcc_toolchain)),
        // Every build uses the SDK toolchain when it is installed, so gate, clippy and linking builds
        // share the same rustflags (no rebuilds when switching). Without it only non-linking builds
        // get here (a linking build failed above); they never reach the linker, and the pinned
        // toolchain's lld stands in.
        None => {
            link_args.push("-fuse-ld=lld".to_owned());
            link_args.push(format!("-B{}", ctx.rustlib_bin.join("gcc-ld").display()));
        }
    }
    link_args.extend(target.c_flags().iter().map(|f| (*f).to_owned()));
    // Cargo takes rustc flags from exactly one source (`CARGO_ENCODED_RUSTFLAGS`, then `RUSTFLAGS`,
    // then the per-target setting), so the caller's flags and the link arguments go together into
    // `CARGO_ENCODED_RUSTFLAGS`. With `--target`, they apply to target artifacts only.
    let mut rustflags = ctx.inherited_rustflags.clone();
    rustflags.extend(link_args.iter().map(|a| format!("-Clink-arg={a}")));
    env.push(("CARGO_ENCODED_RUSTFLAGS".to_owned(), rustflags.join("\x1f")));

    Ok(Invocation {
        program: "cargo".to_owned(),
        args,
        env,
        env_remove: vec!["RUSTFLAGS".to_owned()],
    })
}

/// The `cdylib`s a build produced: `<target dir>/<spec>/<debug|release>/lib<crate>.so` for each
/// selected `cdylib` package (`-` in package names becomes `_`).
pub fn built_libraries(
    target_dir: &Path,
    target: Target,
    release: bool,
    packages: &[String],
) -> Vec<PathBuf> {
    let profile = if release { "release" } else { "debug" };
    packages
        .iter()
        .map(|package| {
            target_dir
                .join(target.spec_name())
                .join(profile)
                .join(format!("lib{}.so", package.replace('-', "_")))
        })
        .collect()
}

/// The selected packages that build a `cdylib` (the same selection as [`selection_needs_link`]).
pub fn selected_cdylibs(metadata: &Value, options: &Options, tv_libs: &[String]) -> Vec<String> {
    if options.all_tv_libs {
        return tv_libs.to_vec();
    }
    let named = options.packages();
    let all = cdylib_packages(metadata);
    if !named.is_empty() {
        return all
            .into_iter()
            .filter(|p| named.contains(&p.as_str()))
            .collect();
    }
    let ids = |key: &str| -> Vec<&str> {
        metadata[key]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    };
    let members = if options.workspace() {
        ids("workspace_members")
    } else {
        let defaults = ids("workspace_default_members");
        if defaults.is_empty() {
            ids("workspace_members")
        } else {
            defaults
        }
    };
    packages(metadata)
        .filter(|p| is_cdylib(p))
        .filter(|p| p["id"].as_str().is_some_and(|id| members.contains(&id)))
        .filter_map(|p| p["name"].as_str().map(str::to_owned))
        .collect()
}

/// Packages of a `cargo metadata --no-deps` document that build a `cdylib`.
pub fn cdylib_packages(metadata: &Value) -> Vec<String> {
    let mut names: Vec<String> = packages(metadata)
        .filter(|p| is_cdylib(p))
        .filter_map(|p| p["name"].as_str().map(str::to_owned))
        .collect();
    names.sort();
    names
}

/// Whether the packages a build selects include a `cdylib`: the named `-p` packages, every member
/// with `--workspace`, otherwise the workspace's default members.
pub fn selection_needs_link(metadata: &Value, options: &Options, tv_libs: &[String]) -> bool {
    if options.all_tv_libs && !tv_libs.is_empty() {
        return true;
    }
    let named = options.packages();
    let member_ids = |key: &str| -> Vec<&str> {
        metadata[key]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    };
    let selected: Vec<&Value> = if !named.is_empty() {
        packages(metadata)
            .filter(|p| p["name"].as_str().is_some_and(|n| named.contains(&n)))
            .collect()
    } else {
        let ids = if options.workspace() {
            member_ids("workspace_members")
        } else {
            let defaults = member_ids("workspace_default_members");
            if defaults.is_empty() {
                member_ids("workspace_members")
            } else {
                defaults
            }
        };
        packages(metadata)
            .filter(|p| p["id"].as_str().is_some_and(|id| ids.contains(&id)))
            .collect()
    };
    selected.into_iter().any(is_cdylib)
}

fn packages(metadata: &Value) -> impl Iterator<Item = &Value> {
    metadata["packages"].as_array().into_iter().flatten()
}

fn is_cdylib(package: &Value) -> bool {
    package["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|t| {
            t["crate_types"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|k| k == "cdylib")
        })
}

#[cfg(test)]
mod tests;
