//! Workspace policy: the pins, lints, release profile, alias and toolchain components that every
//! build of this repository relies on. `cargo xtask policy` and the `policy::tests` group run it.

use std::path::Path;

use anyhow::{Context, Result};
use toml::Value;

/// The exact gpui-fast version NSH's `nsh-platform-gpui` uses; it moves only together with NSH's pin.
pub const GPUI_FAST_VERSION: &str = "=0.1.13";

/// Workspace members every checkout must have.
pub const REQUIRED_MEMBERS: &[&str] = &[
    "crates/dxn_gpui_tizen_sys",
    "crates/dxn_gpui_tizen",
    "crates/dxn_gpui_tizen_diag",
    "examples/counter",
    "examples/long_list",
    "xtask",
];

/// Toolchain components the cross builds and the lint gates need.
pub const REQUIRED_COMPONENTS: &[&str] = &["rust-src", "rustfmt", "clippy", "llvm-tools"];

/// The three files the policy reads, as text.
pub struct Inputs {
    pub manifest: String,
    pub toolchain: String,
    pub cargo_config: String,
}

impl Inputs {
    /// Reads the policy inputs from a workspace root.
    pub fn read(root: &Path) -> Result<Self> {
        let read = |rel: &str| {
            std::fs::read_to_string(root.join(rel)).with_context(|| format!("reading {rel}"))
        };
        Ok(Self {
            manifest: read("Cargo.toml")?,
            toolchain: read("rust-toolchain.toml")?,
            cargo_config: read(".cargo/config.toml")?,
        })
    }
}

/// Checks the policy and returns every violation (empty when the workspace complies).
pub fn check(inputs: &Inputs) -> Vec<String> {
    let mut errors = Vec::new();
    match inputs.manifest.parse::<Value>() {
        Ok(manifest) => check_manifest(&manifest, &mut errors),
        Err(err) => errors.push(format!("Cargo.toml does not parse: {err}")),
    }
    match inputs.toolchain.parse::<Value>() {
        Ok(toolchain) => check_toolchain(&toolchain, &mut errors),
        Err(err) => errors.push(format!("rust-toolchain.toml does not parse: {err}")),
    }
    match inputs.cargo_config.parse::<Value>() {
        Ok(config) => check_cargo_config(&config, &mut errors),
        Err(err) => errors.push(format!(".cargo/config.toml does not parse: {err}")),
    }
    errors
}

fn get<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |v, key| v.get(key))
}

fn expect_str(value: &Value, path: &[&str], expected: &str, file: &str, errors: &mut Vec<String>) {
    match get(value, path).and_then(Value::as_str) {
        Some(actual) if actual == expected => {}
        actual => errors.push(format!(
            "{file}: `{}` must be {expected:?}, found {actual:?}",
            path.join(".")
        )),
    }
}

fn check_manifest(manifest: &Value, errors: &mut Vec<String>) {
    const FILE: &str = "Cargo.toml";
    expect_str(manifest, &["workspace", "resolver"], "2", FILE, errors);
    expect_str(
        manifest,
        &["workspace", "package", "edition"],
        "2024",
        FILE,
        errors,
    );
    expect_str(
        manifest,
        &["workspace", "package", "license"],
        "MIT OR Apache-2.0",
        FILE,
        errors,
    );
    expect_str(
        manifest,
        &["workspace", "package", "repository"],
        "https://github.com/dXnSix/dXn.GpuiTizen",
        FILE,
        errors,
    );

    let members: Vec<&str> = get(manifest, &["workspace", "members"])
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    for required in REQUIRED_MEMBERS {
        if !members.contains(required) {
            errors.push(format!("{FILE}: workspace member `{required}` is missing"));
        }
    }

    for (key, package) in [("gpui", "gpui-fast"), ("gpui_wgpu", "gpui-fast-wgpu")] {
        let dep = get(manifest, &["workspace", "dependencies", key]);
        let pkg = dep.and_then(|d| d.get("package")).and_then(Value::as_str);
        let version = dep.and_then(|d| d.get("version")).and_then(Value::as_str);
        if pkg != Some(package) || version != Some(GPUI_FAST_VERSION) {
            errors.push(format!(
                "{FILE}: workspace dependency `{key}` must be package {package:?} version \
                 {GPUI_FAST_VERSION:?}, found package {pkg:?} version {version:?}"
            ));
        }
    }
    if let Some(dep) = get(manifest, &["workspace", "dependencies", "gpui"])
        && dep.get("default-features").and_then(Value::as_bool) != Some(false)
    {
        errors.push(format!(
            "{FILE}: `gpui` must set `default-features = false`"
        ));
    }

    check_patches(manifest, errors);

    let lint = get(manifest, &["workspace", "lints", "rust", "unexpected_cfgs"]);
    let check_cfg: Vec<&str> = lint
        .and_then(|l| l.get("check-cfg"))
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if !check_cfg.contains(&r#"cfg(target_vendor, values("tizen"))"#) {
        errors.push(format!(
            "{FILE}: `unexpected_cfgs` must declare check-cfg `cfg(target_vendor, values(\"tizen\"))`"
        ));
    }
    expect_lint(manifest, "rust", "unsafe_op_in_unsafe_fn", errors);
    for lint in ["undocumented_unsafe_blocks", "dbg_macro", "todo"] {
        expect_lint(manifest, "clippy", lint, errors);
    }

    let profile = get(manifest, &["profile", "release"]);
    let field = |key: &str| profile.and_then(|p| p.get(key));
    if field("panic").and_then(Value::as_str) != Some("abort") {
        errors.push(format!(
            "{FILE}: release profile must set `panic = \"abort\"`"
        ));
    }
    if field("opt-level").and_then(Value::as_integer) != Some(3) {
        errors.push(format!("{FILE}: release profile must set `opt-level = 3`"));
    }
    if field("lto").and_then(Value::as_str) != Some("fat") {
        errors.push(format!("{FILE}: release profile must set `lto = \"fat\"`"));
    }
    if field("codegen-units").and_then(Value::as_integer) != Some(1) {
        errors.push(format!(
            "{FILE}: release profile must set `codegen-units = 1`"
        ));
    }
    let strip = field("strip");
    if strip.and_then(Value::as_str) != Some("symbols")
        && strip.and_then(Value::as_bool) != Some(true)
    {
        errors.push(format!("{FILE}: release profile must strip symbols"));
    }
}

fn expect_lint(manifest: &Value, tool: &str, lint: &str, errors: &mut Vec<String>) {
    let entry = get(manifest, &["workspace", "lints", tool, lint]);
    let level = entry
        .and_then(Value::as_str)
        .or_else(|| entry.and_then(|e| e.get("level")).and_then(Value::as_str));
    if level != Some("deny") && level != Some("forbid") {
        errors.push(format!(
            "Cargo.toml: `[workspace.lints.{tool}] {lint}` must be denied, found {level:?}"
        ));
    }
}

/// No `[patch]` entry may point at a local path: a local patch only exists on one machine. The
/// committed git patch is all or nothing: every `gpui-fast-*` crate and the whole wgpu stack, as
/// `cargo xtask gpui-fast-release` patches them (`release::CRATES`, `release::WGPU_CRATES`), so no
/// crate of either comes from crates.io beside the forks'.
fn check_patches(manifest: &Value, errors: &mut Vec<String>) {
    let Some(patch) = manifest.get("patch").and_then(Value::as_table) else {
        return;
    };
    if let Some(crates_io) = patch.get("crates-io").and_then(Value::as_table) {
        let names = crate::release::CRATES
            .iter()
            .chain(crate::release::WGPU_CRATES.iter())
            .map(|(_, package)| *package);
        let all: Vec<&str> = names.collect();
        let present: Vec<&&str> = all
            .iter()
            .filter(|name| crates_io.contains_key(**name))
            .collect();
        if !present.is_empty() && present.len() != all.len() {
            let missing: Vec<&str> = all
                .iter()
                .filter(|name| !crates_io.contains_key(**name))
                .copied()
                .collect();
            errors.push(format!(
                "Cargo.toml: `[patch.crates-io]` patches gpui-fast and wgpu only in part; missing {}",
                missing.join(", ")
            ));
        }
    }
    for (registry, entries) in patch {
        for (name, entry) in entries.as_table().into_iter().flatten() {
            if entry.get("path").is_some() {
                errors.push(format!(
                    "Cargo.toml: `[patch.{registry}] {name}` points at a local path; use \
                     `--config artifacts-temp/gpui-fast-patch.toml` instead"
                ));
            }
        }
    }
}

fn check_toolchain(toolchain: &Value, errors: &mut Vec<String>) {
    const FILE: &str = "rust-toolchain.toml";
    let channel = get(toolchain, &["toolchain", "channel"]).and_then(Value::as_str);
    if !channel.is_some_and(is_dated_nightly) {
        errors.push(format!(
            "{FILE}: `toolchain.channel` must be a dated nightly (nightly-YYYY-MM-DD), found {channel:?}"
        ));
    }
    let components: Vec<&str> = get(toolchain, &["toolchain", "components"])
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    for required in REQUIRED_COMPONENTS {
        if !components.contains(required) {
            errors.push(format!("{FILE}: component `{required}` is missing"));
        }
    }
}

/// `nightly-YYYY-MM-DD`.
pub fn is_dated_nightly(channel: &str) -> bool {
    let Some(date) = channel.strip_prefix("nightly-") else {
        return false;
    };
    let parts: Vec<&str> = date.split('-').collect();
    parts.len() == 3
        && [4, 2, 2]
            .iter()
            .zip(&parts)
            .all(|(len, part)| part.len() == *len && part.bytes().all(|b| b.is_ascii_digit()))
}

fn check_cargo_config(config: &Value, errors: &mut Vec<String>) {
    let alias = get(config, &["alias", "xtask"]);
    let words: Vec<String> = match alias {
        Some(Value::String(s)) => s.split_whitespace().map(str::to_owned).collect(),
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    };
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    if !matches!(words.as_slice(), ["run", "--package" | "-p", "xtask", ..]) {
        errors.push(format!(
            ".cargo/config.toml: alias `xtask` must run the xtask package, found {words:?}"
        ));
    }
}

#[cfg(test)]
mod tests;
