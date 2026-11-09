use std::path::{Path, PathBuf};

use super::*;

fn artifact(name: &str, kinds: &[&str], filenames: &[&str]) -> String {
    serde_json::json!({
        "reason": "compiler-artifact",
        "package_id": format!("path+file:///repo#{name}@0.1.0"),
        "target": { "name": name, "kind": kinds },
        "filenames": filenames,
        "fresh": true,
    })
    .to_string()
}

const RELEASE: &str = "/repo/target/armv7-tizen-linux-gnueabi/release";

fn output() -> String {
    [
        artifact(
            "dxn_gpui_tizen_diag",
            &["cdylib", "rlib"],
            &[
                &format!("{RELEASE}/libdxn_gpui_tizen_diag.so"),
                &format!("{RELEASE}/libdxn_gpui_tizen_diag.rlib"),
                &format!(
                    "{RELEASE}/build/dxn_gpui_tizen_diag/ddfa4c78/out/libdxn_gpui_tizen_diag.rmeta"
                ),
            ],
        ),
        artifact(
            "example_counter",
            &["cdylib", "rlib"],
            &[
                &format!("{RELEASE}/libexample_counter.so"),
                &format!("{RELEASE}/build/example_counter/1234/out/libexample_counter.rmeta"),
            ],
        ),
        artifact(
            "gpui",
            &["lib"],
            &[&format!("{RELEASE}/build/gpui/9/out/libgpui.rlib")],
        ),
        r#"{"reason":"build-finished","success":true}"#.to_owned(),
        "not json".to_owned(),
    ]
    .join("\n")
}

#[test]
fn each_unit_s_own_library_is_found_next_to_its_rmeta() {
    let found = cdylibs(
        &output(),
        &["dxn_gpui_tizen_diag".into(), "example-counter".into()],
        |_| true,
    );
    assert_eq!(
        found,
        [
            PathBuf::from(format!(
                "{RELEASE}/build/dxn_gpui_tizen_diag/ddfa4c78/out/libdxn_gpui_tizen_diag.so"
            )),
            PathBuf::from(format!(
                "{RELEASE}/build/example_counter/1234/out/libexample_counter.so"
            )),
        ],
        "not the uplifted copies another build may have written"
    );
}

#[test]
fn only_selected_cdylib_packages_count() {
    let found = cdylibs(
        &output(),
        &["example_counter".into(), "gpui".into()],
        |_| true,
    );
    assert_eq!(found.len(), 1);
    assert!(found[0].ends_with("libexample_counter.so"));
    assert!(cdylibs(&output(), &[], |_| true).is_empty());
    assert!(cdylibs("", &["example_counter".into()], |_| true).is_empty());
}

#[test]
fn without_the_unit_s_copy_the_reported_path_is_used() {
    let found = cdylibs(&output(), &["example_counter".into()], |_: &Path| false);
    assert_eq!(
        found,
        [PathBuf::from(format!("{RELEASE}/libexample_counter.so"))]
    );
    // An old layout with the rmeta beside the uplifted library.
    let flat = artifact(
        "example_counter",
        &["cdylib"],
        &[
            &format!("{RELEASE}/libexample_counter.so"),
            &format!("{RELEASE}/libexample_counter.rmeta"),
        ],
    );
    assert_eq!(
        cdylibs(&flat, &["example_counter".into()], |_| true),
        [PathBuf::from(format!("{RELEASE}/libexample_counter.so"))]
    );
}

#[test]
fn a_package_built_twice_is_listed_once() {
    let twice = format!("{}\n{}", output(), output());
    assert_eq!(
        cdylibs(&twice, &["example_counter".into()], |_| true).len(),
        1
    );
}
