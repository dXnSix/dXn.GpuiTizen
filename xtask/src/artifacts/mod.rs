//! The libraries a Cargo build produced, from its `--message-format=json-render-diagnostics`
//! output. Cargo uplifts every `cdylib` to `<target>/<profile>/lib<crate>.so`, and two builds of
//! the same package with different features (a workspace build that unifies them, a single
//! package's build) overwrite that one path, so a fresh build can leave the other build's copy
//! there. Each unit's own library sits next to its `.rmeta` in the unit's output directory; that
//! is the library this build produced, and the one `--check-symbols` checks and `package` stages.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// The package name in a Cargo package id: `path+file:///x/nsh#nsh-platform-tizen@0.4.0`,
/// `registry+https://...#serde@1.0.0` (after `#`, before `@`), or `path+file:///x/nsh-core#0.4.0`
/// (the last path segment when the fragment is only a version).
pub fn package_name(id: &str) -> Option<&str> {
    let (url, fragment) = id.rsplit_once('#')?;
    match fragment.split_once('@') {
        Some((name, _)) => Some(name),
        None => url.trim_end_matches('/').rsplit('/').next(),
    }
}

/// The selected packages' `cdylib`s from Cargo's JSON messages, matched by package (a library may
/// be named apart from its package: `nsh-platform-tizen` builds `libnsh.so`): each unit's own
/// `.so` when it `exists`, else the path Cargo reported. In message order, without duplicates.
pub fn cdylibs(stdout: &str, packages: &[String], exists: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let crates: Vec<String> = packages.iter().map(|p| p.replace('-', "_")).collect();
    let mut found = Vec::new();
    for line in stdout.lines() {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if message["reason"] != "compiler-artifact" {
            continue;
        }
        let target = &message["target"];
        let is_cdylib = target["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|k| k == "cdylib"));
        let selected = message["package_id"]
            .as_str()
            .and_then(package_name)
            .map(|name| crates.iter().any(|c| *c == name.replace('-', "_")))
            .unwrap_or_else(|| {
                target["name"]
                    .as_str()
                    .is_some_and(|name| crates.iter().any(|c| c == name))
            });
        if !is_cdylib || !selected {
            continue;
        }
        let filenames: Vec<PathBuf> = message["filenames"]
            .as_array()
            .map(|names| {
                names
                    .iter()
                    .filter_map(Value::as_str)
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default();
        let Some(uplifted) = filenames
            .iter()
            .find(|f| f.extension().is_some_and(|e| e == "so"))
        else {
            continue;
        };
        let unit_dir = filenames
            .iter()
            .filter(|f| f.extension().is_some_and(|e| e == "rmeta" || e == "rlib"))
            .filter_map(|f| f.parent())
            .find(|dir| Some(*dir) != uplifted.parent());
        let own = unit_dir
            .zip(uplifted.file_name())
            .map(|(dir, name)| dir.join(name))
            .filter(|own| exists(own));
        let library = own.unwrap_or_else(|| uplifted.clone());
        if !found.contains(&library) {
            found.push(library);
        }
    }
    found
}

#[cfg(test)]
mod tests;
