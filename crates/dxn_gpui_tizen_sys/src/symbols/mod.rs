//! The symbol policy: `symbols.allow` lists every Tizen symbol a TV library may import, with the
//! public Tizen 8.0 native API module that documents it. `xtask check-symbols` reads it from here.

/// The allowlist file, embedded.
pub const ALLOWLIST: &str = include_str!("../../symbols.allow");

/// Prefixes no declared or allowed symbol may have: undocumented display internals (`tdm_`,
/// `tbm_`), DALi and its C# binder (`dali_`, `CSharp_`), and Elementary (`elm_`).
pub const FORBIDDEN_PREFIXES: &[&str] = &["tdm_", "tbm_", "dali_", "CSharp_", "elm_"];

/// One allowlist entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allowed<'a> {
    pub symbol: &'a str,
    pub module: &'a str,
}

/// Parses an allowlist: one `<symbol> <module>` per line; blank lines and `#` comments are skipped.
/// A line without a module yields an entry with an empty module (reported by the tests).
pub fn parse(text: &str) -> impl Iterator<Item = Allowed<'_>> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| match line.split_once(char::is_whitespace) {
            Some((symbol, module)) => Allowed {
                symbol,
                module: module.trim(),
            },
            None => Allowed {
                symbol: line,
                module: "",
            },
        })
}

/// The embedded allowlist's entries.
pub fn allowed() -> impl Iterator<Item = Allowed<'static>> {
    parse(ALLOWLIST)
}

/// Whether `symbol` is in the embedded allowlist.
pub fn is_allowed(symbol: &str) -> bool {
    allowed().any(|entry| entry.symbol == symbol)
}

/// The forbidden prefix `symbol` starts with, if any.
pub fn forbidden_prefix(symbol: &str) -> Option<&'static str> {
    FORBIDDEN_PREFIXES
        .iter()
        .copied()
        .find(|prefix| symbol.starts_with(prefix))
}

#[cfg(test)]
mod tests;
