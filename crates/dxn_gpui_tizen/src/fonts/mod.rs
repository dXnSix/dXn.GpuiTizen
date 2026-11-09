//! Fonts from fontconfig's directories (design `d17-settings-fonts`), read in Rust without linking
//! fontconfig. The text system starts without system fonts and receives only the font files whose
//! families are allowed ([`RunOptions::font_families`](crate::RunOptions::font_families), by
//! default the TV's UI families and their fallbacks), so fontdb never scans every font on the TV.
//!
//! - Directories: the `<dir>` entries of `/etc/fonts/fonts.conf` and of the files its `<include>`
//!   entries name (a directory includes its `NN*.conf` files, in name order, as fontconfig does);
//!   a missing include is skipped. With no directory found, `/usr/share/fonts`.
//! - Families: every face's family names (typographic and legacy) from its `name` table, read with
//!   `ttf-parser`; collections (`.ttc`) count every face.
//! - Allowed: a file is loaded when one of its families equals an allowed family or starts with
//!   one (`BreezeSans` allows `BreezeSansKorean`), ignoring case and spaces.
//!
//! Paths are the device's; `sysroot` (`/` on the TV, a fixture tree in tests) prefixes them on
//! reads.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// fontconfig's configuration.
pub const CONFIG: &str = "/etc/fonts/fonts.conf";
/// The directory used when the configuration names none.
pub const FALLBACK_DIR: &str = "/usr/share/fonts";
/// The families loaded by default: Samsung's TV UI family, then Tizen's (BreezeSans since Tizen
/// 7, TizenSans before), each with its script fallbacks.
pub const DEFAULT_FAMILIES: &[&str] = &["SamsungOneUI", "BreezeSans", "TizenSans"];
/// Font file extensions.
pub const EXTENSIONS: &[&str] = &["ttf", "otf", "ttc", "otc"];
/// Includes followed at most this deep (a guard against include loops through symlinks).
const MAX_INCLUDE_DEPTH: usize = 16;

/// `path` (a device path) under `sysroot`.
fn on(sysroot: &Path, path: &Path) -> PathBuf {
    sysroot.join(path.strip_prefix("/").unwrap_or(path))
}

/// The `<dir>` and `<include>` entries of one configuration file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigEntries {
    pub dirs: Vec<PathBuf>,
    /// With whether a missing target is ignored (`ignore_missing="yes"`).
    pub includes: Vec<(PathBuf, bool)>,
}

fn unescape(text: &str) -> String {
    text.trim()
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// The value of attribute `name` in a start tag's attribute text.
fn attribute<'a>(attributes: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = attributes;
    while let Some(at) = rest.find(name) {
        let after = rest[at + name.len()..].trim_start();
        let boundary = at == 0 || rest[..at].ends_with(char::is_whitespace);
        if boundary && let Some(value) = after.strip_prefix('=') {
            let value = value.trim_start();
            let quote = value.chars().next()?;
            if quote == '"' || quote == '\'' {
                let value = &value[1..];
                return value.find(quote).map(|end| &value[..end]);
            }
        }
        rest = &rest[at + name.len()..];
    }
    None
}

/// Parses a configuration file's `<dir>` and `<include>` entries; relative paths resolve against
/// `base` (the file's directory). `~` and `xdg` paths are skipped: TV apps have no home fonts.
pub fn parse_config(text: &str, base: &Path) -> ConfigEntries {
    let mut entries = ConfigEntries::default();
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        rest = &rest[open..];
        if let Some(comment) = rest.strip_prefix("<!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
            continue;
        }
        let Some(close) = rest.find('>') else { break };
        let tag = &rest[1..close];
        rest = &rest[close + 1..];
        if tag.starts_with(['/', '?', '!']) || tag.ends_with('/') {
            continue;
        }
        let (name, attributes) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
        if name != "dir" && name != "include" {
            continue;
        }
        let end = rest.find('<').unwrap_or(rest.len());
        let value = unescape(&rest[..end]);
        rest = &rest[end..];
        if value.is_empty()
            || value.starts_with('~')
            || attribute(attributes, "prefix") == Some("xdg")
        {
            continue;
        }
        let path = base.join(&value);
        if name == "dir" {
            entries.dirs.push(path);
        } else {
            let ignore_missing = attribute(attributes, "ignore_missing") == Some("yes");
            entries.includes.push((path, ignore_missing));
        }
    }
    entries
}

/// The configuration files a directory include loads: `NN*.conf`, in name order.
fn conf_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with(|c: char| c.is_ascii_digit()) && name.ends_with(".conf")
                })
        })
        .collect();
    files.sort();
    files
}

/// Why the directories came out as they did (for the log).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirsReport {
    pub dirs: Vec<PathBuf>,
    /// Includes that were missing and not marked `ignore_missing`.
    pub missing: Vec<PathBuf>,
    /// Whether [`FALLBACK_DIR`] was used.
    pub fallback: bool,
}

/// The font directories of the configuration at `config` (device paths), in order and without
/// duplicates.
pub fn font_dirs(sysroot: &Path, config: &Path) -> DirsReport {
    let mut report = DirsReport::default();
    let mut seen_files = BTreeSet::new();
    let mut seen_dirs = BTreeSet::new();
    load(
        sysroot,
        config,
        false,
        0,
        &mut seen_files,
        &mut seen_dirs,
        &mut report,
    );
    if report.dirs.is_empty() {
        report.dirs.push(PathBuf::from(FALLBACK_DIR));
        report.fallback = true;
    }
    report
}

fn load(
    sysroot: &Path,
    path: &Path,
    ignore_missing: bool,
    depth: usize,
    seen_files: &mut BTreeSet<PathBuf>,
    seen_dirs: &mut BTreeSet<PathBuf>,
    report: &mut DirsReport,
) {
    if depth > MAX_INCLUDE_DEPTH {
        return;
    }
    let real = on(sysroot, path);
    if real.is_dir() {
        for file in conf_files(&real) {
            let device = Path::new("/").join(file.strip_prefix(sysroot).unwrap_or(&file));
            load(
                sysroot,
                &device,
                true,
                depth + 1,
                seen_files,
                seen_dirs,
                report,
            );
        }
        return;
    }
    if !seen_files.insert(path.to_path_buf()) {
        return;
    }
    let Ok(text) = std::fs::read_to_string(&real) else {
        if !ignore_missing {
            report.missing.push(path.to_path_buf());
        }
        return;
    };
    let base = path.parent().unwrap_or(Path::new("/"));
    let entries = parse_config(&text, base);
    for dir in entries.dirs {
        if seen_dirs.insert(dir.clone()) {
            report.dirs.push(dir);
        }
    }
    for (include, ignore_missing) in entries.includes {
        load(
            sysroot,
            &include,
            ignore_missing,
            depth + 1,
            seen_files,
            seen_dirs,
            report,
        );
    }
}

/// The font files under `dirs` (device paths), recursively, in path order.
pub fn font_files(sysroot: &Path, dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = BTreeSet::new();
    let mut stack: Vec<PathBuf> = dirs.to_vec();
    let mut visited = BTreeSet::new();
    while let Some(dir) = stack.pop() {
        let real = on(sysroot, &dir);
        let canonical = std::fs::canonicalize(&real).unwrap_or(real.clone());
        if !visited.insert(canonical) {
            continue;
        }
        let Ok(read) = std::fs::read_dir(&real) else {
            continue;
        };
        for entry in read.filter_map(Result::ok) {
            let name = entry.file_name();
            let device = dir.join(&name);
            let path = entry.path();
            if path.is_dir() {
                stack.push(device);
            } else if Path::new(&name)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
            {
                files.insert(device);
            }
        }
    }
    files.into_iter().collect()
}

/// Every face's family names in a font file (typographic family first, then the legacy family),
/// without duplicates; empty for data that is not a font.
pub fn families(data: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let count = ttf_parser::fonts_in_collection(data).unwrap_or(1);
    for index in 0..count {
        let Ok(face) = ttf_parser::Face::parse(data, index) else {
            continue;
        };
        for id in [
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
            ttf_parser::name_id::FAMILY,
        ] {
            for name in face.names().into_iter().filter(|name| name.name_id == id) {
                if let Some(text) = name.to_string()
                    && !text.trim().is_empty()
                    && !names.contains(&text)
                {
                    names.push(text);
                }
            }
        }
    }
    names
}

fn normalized(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether a file with `families` is loaded for the `allowed` families.
pub fn allowed(families: &[String], allowed: &[String]) -> bool {
    families.iter().any(|family| {
        let family = normalized(family);
        allowed
            .iter()
            .map(|a| normalized(a))
            .any(|a| !a.is_empty() && family.starts_with(&a))
    })
}

/// The allowed families: the options' list, else [`DEFAULT_FAMILIES`].
pub fn allowed_families(requested: Option<&[String]>) -> Vec<String> {
    requested.map_or_else(
        || DEFAULT_FAMILIES.iter().map(|f| (*f).to_owned()).collect(),
        <[String]>::to_vec,
    )
}

/// A font file to load, with its families.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub path: PathBuf,
    pub families: Vec<String>,
}

/// The files to load from `files` for `allowed`, reading each through `open` (the shell maps
/// them; tests read them).
pub fn select<B: AsRef<[u8]>>(
    files: &[PathBuf],
    allowed_families: &[String],
    mut open: impl FnMut(&Path) -> Option<B>,
) -> Vec<Selected> {
    files
        .iter()
        .filter_map(|path| {
            let data = open(path)?;
            let families = families(data.as_ref());
            allowed(&families, allowed_families).then(|| Selected {
                path: path.clone(),
                families,
            })
        })
        .collect()
}

/// The family the text system falls back to: the first allowed family a selected file has by
/// that exact name, else the first family of the first selected file, else the first allowed
/// family.
pub fn ui_family(selected: &[Selected], allowed_families: &[String]) -> Option<String> {
    allowed_families
        .iter()
        .find_map(|a| {
            let a = normalized(a);
            selected
                .iter()
                .flat_map(|s| &s.families)
                .find(|family| normalized(family) == a)
                .cloned()
        })
        .or_else(|| {
            selected
                .iter()
                .flat_map(|s| s.families.first())
                .next()
                .cloned()
        })
        .or_else(|| allowed_families.first().cloned())
}

#[cfg(test)]
mod tests;
