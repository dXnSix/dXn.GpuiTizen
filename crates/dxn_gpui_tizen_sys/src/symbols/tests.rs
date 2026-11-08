extern crate std;

use std::collections::BTreeSet;
use std::vec::Vec;

use super::*;
use crate::DECLARED_SYMBOLS;

fn declared() -> BTreeSet<&'static str> {
    DECLARED_SYMBOLS.iter().map(|(symbol, _)| *symbol).collect()
}

fn allowed_set() -> BTreeSet<&'static str> {
    allowed().map(|entry| entry.symbol).collect()
}

#[test]
fn every_declared_symbol_is_allowed() {
    let missing: Vec<_> = declared().difference(&allowed_set()).copied().collect();
    assert!(
        missing.is_empty(),
        "declared but not in symbols.allow: {missing:?}"
    );
}

#[test]
fn every_allowed_symbol_is_declared() {
    let stale: Vec<_> = allowed_set().difference(&declared()).copied().collect();
    assert!(
        stale.is_empty(),
        "in symbols.allow but no longer declared: {stale:?}"
    );
}

#[test]
fn no_forbidden_prefixes() {
    let bad: Vec<_> = declared()
        .union(&allowed_set())
        .filter(|s| forbidden_prefix(s).is_some())
        .copied()
        .collect();
    assert!(bad.is_empty(), "forbidden prefixes: {bad:?}");
    for (symbol, prefix) in [
        ("tdm_client_create", "tdm_"),
        ("tbm_surface_create", "tbm_"),
        ("dali_window_new", "dali_"),
        ("CSharp_Dali_Window_New", "CSharp_"),
        ("elm_init", "elm_"),
    ] {
        assert_eq!(forbidden_prefix(symbol), Some(prefix));
    }
    assert_eq!(forbidden_prefix("ecore_wl2_window_new"), None);
}

#[test]
fn every_entry_names_its_module_once() {
    let entries: Vec<_> = allowed().collect();
    assert!(!entries.is_empty());
    for entry in &entries {
        assert!(
            !entry.module.is_empty(),
            "{} has no API module",
            entry.symbol
        );
    }
    assert_eq!(
        entries.len(),
        allowed_set().len(),
        "duplicate allowlist entries"
    );
    assert_eq!(
        DECLARED_SYMBOLS.len(),
        declared().len(),
        "a symbol is declared twice"
    );
}

#[test]
fn parser_skips_comments_and_reports_missing_modules() {
    let parsed: Vec<_> =
        parse("# c\n\nui_app_main Application (appfw/app.h)\n  lonely  \n").collect();
    assert_eq!(
        parsed,
        [
            Allowed {
                symbol: "ui_app_main",
                module: "Application (appfw/app.h)"
            },
            Allowed {
                symbol: "lonely",
                module: ""
            },
        ]
    );
    assert!(is_allowed("ui_app_main"));
    assert!(!is_allowed("tdm_client_create"));
}

#[test]
fn declarations_record_their_library() {
    let library = |symbol: &str| {
        DECLARED_SYMBOLS
            .iter()
            .find(|(s, _)| *s == symbol)
            .map(|(_, l)| *l)
    };
    assert_eq!(library("ui_app_main"), Some("capi-appfw-application"));
    assert_eq!(library("ECORE_EVENT_KEY_DOWN"), Some("ecore_input"));
    assert_eq!(library("dlog_print"), Some("dlog"));
    assert_eq!(
        library("ecore_wl2_window_native_surface_get"),
        Some("ecore_wl2")
    );
}
