//! `cargo xtask tizen-deps --target <t>`: the Tizen dependency graph must hold no desktop platform
//! crate (GPUI's desktop platforms, X11, xkbcommon, portals, FreeType, fontconfig, font-kit) and
//! exactly one wgpu, and the diagnostic app's graph must hold no GPUI crate and that same wgpu
//! (the one gpui-fast-wgpu resolves to). Input is
//! `cargo tree -e normal --prefix none --format '{p}'` output.

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
        .filter(|(name, _)| DESKTOP_CRATES.contains(&name.as_str()))
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

#[cfg(test)]
mod tests;
