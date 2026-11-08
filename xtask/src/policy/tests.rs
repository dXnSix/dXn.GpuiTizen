use super::*;
use crate::workspace_root;

fn inputs() -> Inputs {
    Inputs::read(&workspace_root()).expect("policy inputs are readable")
}

fn with_manifest(edit: impl FnOnce(&str) -> String) -> Vec<String> {
    let mut inputs = inputs();
    inputs.manifest = edit(&inputs.manifest);
    check(&inputs)
}

fn replace(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture text {from:?} not found");
    text.replacen(from, to, 1)
}

#[test]
fn workspace_complies() {
    assert_eq!(check(&inputs()), Vec::<String>::new());
}

#[test]
fn gpui_fast_pin_drift_fails() {
    let errors = with_manifest(|m| {
        replace(
            m,
            r#"version = "=0.1.13", default"#,
            r#"version = "0.1.13", default"#,
        )
    });
    assert!(errors.iter().any(|e| e.contains("`gpui`")), "{errors:?}");
    let errors = with_manifest(|m| {
        replace(
            m,
            r#"gpui-fast-wgpu", version = "=0.1.13""#,
            r#"gpui-fast-wgpu", version = "=0.1.14""#,
        )
    });
    assert!(
        errors.iter().any(|e| e.contains("`gpui_wgpu`")),
        "{errors:?}"
    );
}

#[test]
fn local_path_patch_fails() {
    let errors = with_manifest(|m| {
        format!("{m}\n[patch.crates-io]\ngpui-fast = {{ path = \"../gpui-fast/crates/gpui\" }}\n")
    });
    assert!(
        errors.iter().any(|e| e.contains("local path")),
        "{errors:?}"
    );
    let errors = with_manifest(|m| {
        format!(
            "{m}\n[patch.crates-io]\ngpui-fast = {{ git = \"https://github.com/dXnSix/gpui-fast\", branch = \"tizen-release\" }}\n"
        )
    });
    assert!(
        !errors.iter().any(|e| e.contains("local path")),
        "{errors:?}"
    );
}

#[test]
fn lint_drift_fails() {
    let errors = with_manifest(|m| {
        replace(
            m,
            r#"unsafe_op_in_unsafe_fn = "deny""#,
            r#"unsafe_op_in_unsafe_fn = "warn""#,
        )
    });
    assert!(
        errors.iter().any(|e| e.contains("unsafe_op_in_unsafe_fn")),
        "{errors:?}"
    );
    let errors = with_manifest(|m| replace(m, r#"undocumented_unsafe_blocks = "deny""#, ""));
    assert!(
        errors
            .iter()
            .any(|e| e.contains("undocumented_unsafe_blocks")),
        "{errors:?}"
    );
    let errors = with_manifest(|m| replace(m, r#"dbg_macro = "deny""#, r#"dbg_macro = "allow""#));
    assert!(errors.iter().any(|e| e.contains("dbg_macro")), "{errors:?}");
    let errors = with_manifest(|m| replace(m, r#"todo = "deny""#, ""));
    assert!(errors.iter().any(|e| e.contains("todo")), "{errors:?}");
    let errors = with_manifest(|m| replace(m, r#"values("tizen")"#, r#"values("linux")"#));
    assert!(errors.iter().any(|e| e.contains("check-cfg")), "{errors:?}");
}

#[test]
fn release_profile_drift_fails() {
    for (from, to, needle) in [
        (
            r#"panic = "abort"
opt-level"#,
            r#"panic = "unwind"
opt-level"#,
            "panic",
        ),
        ("opt-level = 3", "opt-level = 2", "opt-level"),
        (r#"lto = "fat""#, r#"lto = "thin""#, "lto"),
        ("codegen-units = 1", "codegen-units = 16", "codegen-units"),
        (r#"strip = "symbols""#, r#"strip = "none""#, "strip"),
    ] {
        let errors = with_manifest(|m| replace(m, from, to));
        assert!(
            errors.iter().any(|e| e.contains(needle)),
            "{needle}: {errors:?}"
        );
    }
}

#[test]
fn member_and_workspace_package_drift_fails() {
    let errors = with_manifest(|m| {
        replace(
            m,
            "    \"examples/long_list\",\n    \"xtask\",",
            "    \"xtask\",",
        )
    });
    assert!(
        errors.iter().any(|e| e.contains("examples/long_list")),
        "{errors:?}"
    );
    let errors = with_manifest(|m| replace(m, r#"edition = "2024""#, r#"edition = "2021""#));
    assert!(errors.iter().any(|e| e.contains("edition")), "{errors:?}");
    let errors =
        with_manifest(|m| replace(m, r#"license = "MIT OR Apache-2.0""#, r#"license = "MIT""#));
    assert!(errors.iter().any(|e| e.contains("license")), "{errors:?}");
}

#[test]
fn toolchain_drift_fails() {
    let mut drifted = inputs();
    drifted.toolchain = replace(&drifted.toolchain, r#""llvm-tools""#, r#""rust-docs""#);
    let errors = check(&drifted);
    assert!(
        errors.iter().any(|e| e.contains("llvm-tools")),
        "{errors:?}"
    );

    let mut drifted = inputs();
    drifted.toolchain = "[toolchain]\nchannel = \"nightly\"\ncomponents = [\"rust-src\", \"rustfmt\", \"clippy\", \"llvm-tools\"]\n".into();
    let errors = check(&drifted);
    assert!(
        errors.iter().any(|e| e.contains("dated nightly")),
        "{errors:?}"
    );
    assert!(is_dated_nightly("nightly-2026-10-18"));
    assert!(!is_dated_nightly("nightly-2026-10"));
    assert!(!is_dated_nightly("1.99.0"));
}

#[test]
fn alias_drift_fails() {
    let mut drifted = inputs();
    drifted.cargo_config = "[alias]\nxt = \"run --package xtask --\"\n".into();
    let errors = check(&drifted);
    assert!(
        errors.iter().any(|e| e.contains("alias `xtask`")),
        "{errors:?}"
    );
}
