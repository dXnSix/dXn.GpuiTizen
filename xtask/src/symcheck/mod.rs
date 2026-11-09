//! `cargo xtask check-symbols`: a TV library must export exactly one global symbol (its entry),
//! export nothing weak, and import only documented Tizen symbols (`symbols.allow` in
//! `dxn_gpui_tizen_sys`), glibc (`libc`, `libm`, `libdl`, `libpthread`, `librt`, `ld-linux`),
//! `libgcc_s` and the run-time loader.
//!
//! Input is `llvm-nm -D --defined-only`, `llvm-nm -D --undefined-only` and
//! `llvm-readobj --needed-libs` output from the pinned toolchain's `llvm-tools`; parsing and checking
//! are pure functions.

use std::fmt;

/// The entry point of this repository's TV libraries.
pub const DEFAULT_ENTRY: &str = "dxn_gpui_tizen_main";

/// Symbol-version prefixes of the system libraries a TV library may import from: every glibc
/// library and the loader version their symbols `GLIBC_*`; `libgcc_s` versions its own `GCC_*`.
pub const SYSTEM_VERSION_PREFIXES: &[&str] = &["GLIBC_", "GCC_"];

/// Unversioned weak imports emitted by the C runtime start files and Rust's `std` that the
/// run-time loader resolves (or leaves null) without a library. The last three are glibc
/// functions newer than the TVs' glibc 2.30 that `std::process` binds weakly (unversioned, since
/// the rootstrap's glibc lacks them) and falls back from when null; `zbus` (under
/// `accesskit_unix`) links process spawning in.
pub const LOADER_WEAK_IMPORTS: &[&str] = &[
    "__gmon_start__",
    "_ITM_deregisterTMCloneTable",
    "_ITM_registerTMCloneTable",
    "__clock_gettime64",
    "__pthread_get_minstack",
    "pidfd_getpid",
    "pidfd_spawnp",
    "posix_spawn_file_actions_addchdir",
];

/// The system libraries a TV library may need besides the Tizen ones of `DECLARED_SYMBOLS`: glibc's
/// (`libc`, `libm`, `libdl`, `libpthread`, `librt`), the loader (`ld-linux*`) and `libgcc_s`.
pub const SYSTEM_LIBRARIES: &[&str] = &[
    "libc.so.6",
    "libm.so.6",
    "libdl.so.2",
    "libpthread.so.0",
    "librt.so.1",
    "libgcc_s.so.1",
];

/// Parses `llvm-readobj --needed-libs` output: the names inside `NeededLibraries [ ... ]`.
pub fn parse_needed(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut needed = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with("NeededLibraries") {
            inside = true;
        } else if inside && line == "]" {
            break;
        } else if inside && !line.is_empty() {
            needed.push(line.to_owned());
        }
    }
    needed
}

/// Whether a `DT_NEEDED` entry is allowed: a system library, the loader, or `lib<name>.so[.*]` of a
/// Tizen library in `tizen_libraries` (the link names `DECLARED_SYMBOLS` records).
pub fn library_allowed(needed: &str, tizen_libraries: &[&str]) -> bool {
    if SYSTEM_LIBRARIES.contains(&needed)
        || (needed.starts_with("ld-linux") && needed.contains(".so."))
    {
        return true;
    }
    tizen_libraries.iter().any(|name| {
        needed
            .strip_prefix("lib")
            .and_then(|rest| rest.strip_prefix(name))
            .is_some_and(|rest| rest == ".so" || rest.starts_with(".so."))
    })
}

/// One line of `llvm-nm -D` output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    /// The symbol version (`GLIBC_2.4` of `memcpy@GLIBC_2.4`), if any.
    pub version: Option<String>,
    /// nm's type letter: `T`, `D`, `B`, `R`, `W`, `V`, `U`, `w`, ...
    pub kind: char,
}

impl Symbol {
    /// A weak symbol (`W`/`V` defined, `w`/`v` undefined).
    pub fn is_weak(&self) -> bool {
        matches!(self.kind, 'W' | 'V' | 'w' | 'v')
    }
}

/// Parses `llvm-nm -D` output (`[address] <type> <name>[@[@]version]`).
pub fn parse_nm(text: &str) -> Vec<Symbol> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace().rev();
            let name = fields.next()?;
            let kind = fields.next()?.chars().next()?;
            let (name, version) = match name.split_once('@') {
                Some((name, version)) => (name, Some(version.trim_start_matches('@').to_owned())),
                None => (name, None),
            };
            Some(Symbol {
                name: name.to_owned(),
                version,
                kind,
            })
        })
        .collect()
}

/// A policy violation, reported by symbol name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    /// The entry symbol is not exported.
    MissingEntry(String),
    /// A global symbol other than the entry is exported.
    ExtraExport(String),
    /// A weak symbol is exported.
    WeakExport(String),
    /// An import that is neither in `symbols.allow` nor a system library's.
    UndocumentedImport(String),
    /// A `DT_NEEDED` library that is neither a system library nor a declared Tizen one.
    UnexpectedLibrary(String),
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::MissingEntry(name) => write!(f, "the entry symbol `{name}` is not exported"),
            Violation::ExtraExport(name) => {
                write!(f, "`{name}` is exported; only the entry symbol may be")
            }
            Violation::WeakExport(name) => write!(f, "`{name}` is a weak exported symbol"),
            Violation::UnexpectedLibrary(name) => write!(
                f,
                "`{name}` is a needed library but neither glibc, libgcc_s, the loader nor a Tizen library of dxn_gpui_tizen_sys"
            ),
            Violation::UndocumentedImport(name) => write!(
                f,
                "`{name}` is imported but is neither in symbols.allow nor a glibc/libgcc_s/loader symbol"
            ),
        }
    }
}

/// Checks a library's dynamic symbols against the policy.
pub fn check(
    defined: &[Symbol],
    undefined: &[Symbol],
    entry: &str,
    is_allowed: impl Fn(&str) -> bool,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    // `.dynsym` holds no local symbols, so every defined entry is an export, whatever its letter
    // (`llvm-nm` prints IFUNC exports as `i` and GNU-unique ones as `u`).
    let exports: Vec<&Symbol> = defined.iter().collect();
    if !exports.iter().any(|s| s.name == entry && !s.is_weak()) {
        violations.push(Violation::MissingEntry(entry.to_owned()));
    }
    for symbol in exports {
        if symbol.is_weak() {
            violations.push(Violation::WeakExport(symbol.name.clone()));
        } else if symbol.name != entry {
            violations.push(Violation::ExtraExport(symbol.name.clone()));
        }
    }
    for symbol in undefined {
        let system = symbol
            .version
            .as_deref()
            .is_some_and(|v| SYSTEM_VERSION_PREFIXES.iter().any(|p| v.starts_with(p)));
        let loader = symbol.is_weak()
            && symbol.version.is_none()
            && LOADER_WEAK_IMPORTS.contains(&symbol.name.as_str());
        if !system && !loader && !is_allowed(&symbol.name) {
            violations.push(Violation::UndocumentedImport(symbol.name.clone()));
        }
    }
    violations
}

/// Checks `DT_NEEDED` entries.
pub fn check_needed(needed: &[String], tizen_libraries: &[&str]) -> Vec<Violation> {
    needed
        .iter()
        .filter(|n| !library_allowed(n, tizen_libraries))
        .map(|n| Violation::UnexpectedLibrary(n.clone()))
        .collect()
}

/// The Tizen link names `dxn_gpui_tizen_sys` declares symbols from.
pub fn tizen_libraries() -> Vec<&'static str> {
    let mut names: Vec<&str> = dxn_gpui_tizen_sys::DECLARED_SYMBOLS
        .iter()
        .map(|(_, l)| *l)
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Checks `llvm-nm` and `llvm-readobj --needed-libs` output against the policy and
/// `dxn_gpui_tizen_sys`'s allowlist.
pub fn check_library(defined: &str, undefined: &str, needed: &str, entry: &str) -> Vec<Violation> {
    let mut violations = check(
        &parse_nm(defined),
        &parse_nm(undefined),
        entry,
        dxn_gpui_tizen_sys::symbols::is_allowed,
    );
    violations.extend(check_needed(&parse_needed(needed), &tizen_libraries()));
    violations
}

#[cfg(test)]
mod tests;
