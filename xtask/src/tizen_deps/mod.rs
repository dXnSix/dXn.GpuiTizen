//! `cargo xtask tizen-deps --target <t>`: the Tizen dependency graph must hold no desktop platform
//! crate (GPUI's desktop platforms, X11, xkbcommon, portals, FreeType, fontconfig, font-kit) and
//! exactly one wgpu, and the diagnostic app's graph must hold no GPUI crate and that same wgpu
//! (the one gpui-fast-wgpu resolves to). Input is
//! `cargo tree -e normal --prefix none --format '{p}'` output.
//!
//! Another workspace (NSH's `crates/gpui`) is checked with `--manifest-path`: its Tizen graph
//! (`-p`, `--no-default-features`, `--features`) holds no desktop platform crate and one wgpu,
//! resolved into a lockfile kept under `artifacts-temp/locks/`. `--single <crate>` reads a
//! workspace's committed `Cargo.lock` and requires exactly one version of the crate across every
//! target.

use std::collections::BTreeSet;

/// Crates that must never reach a Tizen build.
pub const DESKTOP_CRATES: &[&str] = &[
    "gpui-fast-platform",
    "gpui-fast-linux",
    "gpui-fast-macos",
    "gpui-fast-windows",
    "gpui-fast-web",
    "x11rb",
    "x11-dl",
    "x11",
    "xkbcommon",
    "xkbcommon-dl",
    "ashpd",
    "freetype-sys",
    "yeslogic-fontconfig-sys",
    "font-kit",
    "zed-font-kit",
    "winit",
];

/// Name prefixes of desktop toolkit crates that must never reach a Tizen build: GTK and its
/// libraries, WebKitGTK, GStreamer (NSH's Linux web view and video), wry.
pub const DESKTOP_PREFIXES: &[&str] = &[
    "gtk",
    "gdk",
    "gio",
    "glib",
    "gobject",
    "pango",
    "cairo",
    "webkit",
    "javascriptcore",
    "soup",
    "gstreamer",
    "wry",
];

/// Whether `name` is a desktop platform or toolkit crate: one of [`DESKTOP_CRATES`], or a
/// toolkit prefix followed by nothing, `-` or a version digit (`gtk4`, `gtk4-sys`, `soup3`).
pub fn is_desktop_crate(name: &str) -> bool {
    DESKTOP_CRATES.contains(&name)
        || DESKTOP_PREFIXES.iter().any(|prefix| {
            name.strip_prefix(prefix).is_some_and(|rest| {
                rest.is_empty()
                    || rest.starts_with('-')
                    || rest.starts_with(|c: char| c.is_ascii_digit())
            })
        })
}

/// The wgpu versions in a graph.
pub fn wgpu_versions(packages: &BTreeSet<(String, String)>) -> Vec<String> {
    packages
        .iter()
        .filter(|(n, _)| n == "wgpu")
        .map(|(_, v)| v.clone())
        .collect()
}

/// The workspace's Tizen graph holds exactly one wgpu; returns it, or the violation.
pub fn single_wgpu(packages: &BTreeSet<(String, String)>) -> Result<String, String> {
    match wgpu_versions(packages).as_slice() {
        [version] => Ok(version.clone()),
        [] => Err("no wgpu in the Tizen graph".to_owned()),
        many => Err(format!(
            "{} wgpu versions in the Tizen graph: {}",
            many.len(),
            many.join(", ")
        )),
    }
}

/// The line the check prints on success.
pub const OK_LINE: &str = "no desktop platform crates";

/// Parses `cargo tree --prefix none --format '{p}'` lines into `(name, version)` pairs.
pub fn parse_tree(text: &str) -> BTreeSet<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?;
            let version = words.next()?.strip_prefix('v')?;
            Some((name.to_owned(), version.to_owned()))
        })
        .collect()
}

/// Desktop platform crates in a graph.
pub fn desktop_crates(packages: &BTreeSet<(String, String)>) -> Vec<String> {
    packages
        .iter()
        .filter(|(name, _)| is_desktop_crate(name))
        .map(|(name, version)| format!("{name} {version}"))
        .collect()
}

/// The diagnostic app's own rules: no GPUI crate, and exactly the workspace's wgpu (`expected`).
pub fn diag_violations(
    packages: &BTreeSet<(String, String)>,
    expected: Option<&str>,
) -> Vec<String> {
    let mut violations: Vec<String> = packages
        .iter()
        .filter(|(name, _)| {
            name == "gpui" || name.starts_with("gpui-fast") || name.starts_with("gpui_")
        })
        .map(|(name, version)| format!("GPUI crate {name} {version} in the diagnostic app"))
        .collect();
    match (wgpu_versions(packages).as_slice(), expected) {
        ([version], Some(expected)) if version == expected => {}
        ([version], Some(expected)) => violations.push(format!(
            "the diagnostic app's wgpu {version} is not the workspace's wgpu {expected}"
        )),
        ([_], None) => {}
        ([], _) => violations.push("no wgpu in the diagnostic app".to_owned()),
        (many, _) => violations.push(format!("{} wgpu versions: {}", many.len(), many.join(", "))),
    }
    violations
}

/// The usage line of `cargo xtask tizen-deps`.
pub const USAGE: &str = "usage: cargo xtask tizen-deps --target <armv7|x86_64|aarch64> \
[--manifest-path <Cargo.toml> [-p <package>] [--no-default-features] [--features <f>]]
       cargo xtask tizen-deps --single <crate> [--manifest-path <Cargo.toml>]";

/// What `cargo xtask tizen-deps` checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// The Tizen graph of a target: this workspace (with the diagnostic app's rules) when
    /// `manifest_path` is `None`, else another workspace's.
    Graph {
        target: String,
        manifest_path: Option<String>,
        package: Option<String>,
        no_default_features: bool,
        features: Option<String>,
    },
    /// One version of `crate_name` in a workspace's `Cargo.lock`.
    Single {
        crate_name: String,
        manifest_path: Option<String>,
    },
}

/// Parses the arguments of `cargo xtask tizen-deps`.
pub fn parse_check(args: &[String]) -> Result<Check, String> {
    let usage = || USAGE.to_owned();
    let mut target = None;
    let mut manifest_path = None;
    let mut package = None;
    let mut no_default_features = false;
    let mut features = None;
    let mut single = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let mut value = || match rest.next() {
            Some(v) if !v.starts_with('-') => Ok(v.clone()),
            _ => Err(usage()),
        };
        match arg.as_str() {
            "--target" => target = Some(value()?),
            "--manifest-path" => manifest_path = Some(value()?),
            "-p" | "--package" => package = Some(value()?),
            "--features" => features = Some(value()?),
            "--single" => single = Some(value()?),
            "--no-default-features" => no_default_features = true,
            _ => return Err(usage()),
        }
    }
    match (single, target) {
        (Some(crate_name), None)
            if package.is_none() && !no_default_features && features.is_none() =>
        {
            Ok(Check::Single {
                crate_name,
                manifest_path,
            })
        }
        (None, Some(target)) => {
            let narrows = package.is_some() || no_default_features || features.is_some();
            if narrows && manifest_path.is_none() {
                return Err(usage());
            }
            Ok(Check::Graph {
                target,
                manifest_path,
                package,
                no_default_features,
                features,
            })
        }
        _ => Err(usage()),
    }
}

/// The versions of `crate_name` a `Cargo.lock` holds, sorted.
pub fn lock_versions(lock: &str, crate_name: &str) -> Result<Vec<String>, String> {
    let value: toml::Value = lock
        .parse()
        .map_err(|e| format!("the lockfile does not parse: {e}"))?;
    let mut versions: Vec<String> = value
        .get("package")
        .and_then(toml::Value::as_array)
        .map(|packages| {
            packages
                .iter()
                .filter(|p| p.get("name").and_then(toml::Value::as_str) == Some(crate_name))
                .filter_map(|p| p.get("version").and_then(toml::Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    versions.sort();
    versions.dedup();
    Ok(versions)
}

/// Exactly one version of `crate_name` in a lockfile; returns it, or the violation.
pub fn single_in_lock(lock: &str, crate_name: &str) -> Result<String, String> {
    match lock_versions(lock, crate_name)?.as_slice() {
        [version] => Ok(version.clone()),
        [] => Err(format!("no {crate_name} in the lockfile")),
        many => Err(format!(
            "{} {crate_name} versions in the lockfile: {}",
            many.len(),
            many.join(", ")
        )),
    }
}

/// The violations of another workspace's Tizen graph: desktop platform crates, and not exactly
/// one wgpu (a graph with no wgpu passes: not every package renders).
pub fn foreign_violations(packages: &BTreeSet<(String, String)>) -> Vec<String> {
    let mut violations = desktop_crates(packages);
    let versions = wgpu_versions(packages);
    if versions.len() > 1 {
        violations.push(format!(
            "{} wgpu versions in the Tizen graph: {}",
            versions.len(),
            versions.join(", ")
        ));
    }
    violations
}

#[cfg(test)]
mod tests;
