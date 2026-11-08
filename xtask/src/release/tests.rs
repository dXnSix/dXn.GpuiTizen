use std::path::Path;

use super::*;

const WS: &str = "/work/gpui-fast/target/release-workspace";

#[test]
fn nine_patch_entries_into_the_release_workspace() {
    let config = patch_config(Path::new(WS));
    let value: toml::Value = config.parse().expect("the patch config is valid TOML");
    let patch = value["patch"]["crates-io"].as_table().unwrap();
    assert_eq!(patch.len(), 9, "all nine gpui-fast crates, never a subset");
    for (dir, package) in CRATES {
        assert_eq!(
            patch[package]["path"].as_str(),
            Some(format!("{WS}/{dir}").as_str()),
            "{package}"
        );
    }
    assert_eq!(
        patch["gpui-fast"]["path"].as_str(),
        Some("/work/gpui-fast/target/release-workspace/gpui")
    );
    assert_eq!(
        patch["gpui-fast-wgpu"]["path"].as_str(),
        Some("/work/gpui-fast/target/release-workspace/gpui_wgpu")
    );
    let packages: std::collections::BTreeSet<&str> = CRATES.iter().map(|(_, p)| *p).collect();
    assert_eq!(packages.len(), 9);
    assert!(packages.iter().all(|p| p.starts_with("gpui-fast")));
}

const PATCHED_LOCK: &str = r#"
version = 4

[[package]]
name = "dxn_gpui_tizen"
version = "0.1.0"

[[package]]
name = "gpui-fast"
version = "0.1.13"

[[package]]
name = "gpui-fast-macros"
version = "0.1.13"

[[package]]
name = "gpui-fast-wgpu"
version = "0.1.13"

[[package]]
name = "wgpu"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[patch.unused]]
name = "gpui-fast-linux"
version = "0.1.13"

[[patch.unused]]
name = "gpui-fast-platform"
version = "0.1.13"
"#;

#[test]
fn tree_resolving_to_the_release_workspace_passes() {
    let stdout = format!(
        "gpui-fast v0.1.13 ({WS}/gpui)\n└── gpui-fast-wgpu v0.1.13 ({WS}/gpui_wgpu)\n    └── dxn_gpui_tizen v0.1.0 (/work/dXn.GpuiTizen/crates/dxn_gpui_tizen)\n"
    );
    assert_eq!(verify_tree(&stdout, Path::new(WS)), Ok(()));
    let from_registry = "gpui-fast v0.1.13\n└── dxn_gpui_tizen v0.1.0 (/work/dXn.GpuiTizen/crates/dxn_gpui_tizen)\n";
    assert!(
        verify_tree(from_registry, Path::new(WS))
            .unwrap_err()
            .contains("does not resolve")
    );
    assert!(
        verify_tree("", Path::new(WS))
            .unwrap_err()
            .contains("does not resolve")
    );
}

#[test]
fn the_patched_lock_passes_with_only_desktop_patches_unused() {
    assert_eq!(verify_lock(PATCHED_LOCK), Ok(()));
}

#[test]
fn an_unused_patch_of_a_crate_the_graph_holds_fails() {
    let lock = format!(
        "{PATCHED_LOCK}\n[[patch.unused]]\nname = \"gpui-fast-wgpu\"\nversion = \"0.1.13\"\n"
    );
    let err = verify_lock(&lock).unwrap_err();
    assert!(
        err.contains("gpui-fast-wgpu") && err.contains("crates.io"),
        "{err}"
    );
}

#[test]
fn gpui_fast_from_the_registry_or_an_empty_graph_fails() {
    let lock = PATCHED_LOCK.replace(
        "name = \"gpui-fast\"\nversion = \"0.1.13\"\n",
        "name = \"gpui-fast\"\nversion = \"0.1.13\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
    );
    assert!(verify_lock(&lock).unwrap_err().contains("still comes from"));
    assert!(
        verify_lock("version = 4\n")
            .unwrap_err()
            .contains("no packages")
    );
    assert!(
        verify_lock("version = 4\n[[package]]\nname = \"x\"\nversion = \"1.0.0\"\n")
            .unwrap_err()
            .contains("not in the graph")
    );
    assert!(verify_lock("not toml [").is_err());
}

#[test]
fn the_release_branch_keeps_only_generated_files() {
    for name in ["target", "Cargo.lock", ".git"] {
        assert!(NOT_RELEASED.contains(&name), "{name}");
    }
}

#[test]
fn the_commit_names_its_source() {
    let message = commit_message("babf63e");
    assert!(
        message.starts_with("release: gpui-fast 0.1.13 for Tizen"),
        "{message}"
    );
    assert!(
        message.contains("--tag v0.1.13") && message.contains("`tizen` at babf63e"),
        "{message}"
    );
    assert!(!message.to_lowercase().contains("claude"));
}

#[test]
fn patched_builds_are_recognized() {
    let args = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert!(uses_patch_config(&args(&[
        "-p",
        "x",
        "--config",
        "artifacts-temp/gpui-fast-patch.toml"
    ])));
    assert!(uses_patch_config(&args(&[
        "--config=/repo/artifacts-temp/gpui-fast-patch.toml"
    ])));
    assert!(!uses_patch_config(&args(&[
        "--config",
        "net.offline=true",
        "-p",
        "gpui-fast-patch.toml"
    ])));
    assert!(!uses_patch_config(&args(&["-p", "dxn_gpui_tizen"])));
}
