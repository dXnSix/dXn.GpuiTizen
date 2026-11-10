use std::path::Path;

use super::*;

const WS: &str = "/work/gpui-fast/target/release-workspace";
const WGPU: &str = "/work/wgpu";

#[test]
fn nine_patch_entries_into_the_release_workspace() {
    let config = patch_config(Path::new(WS), Path::new(WGPU));
    let value: toml::Value = config.parse().expect("the patch config is valid TOML");
    let patch = value["patch"]["crates-io"].as_table().unwrap();
    assert_eq!(
        patch.len(),
        9 + 11,
        "all nine gpui-fast crates, never a subset, and wgpu"
    );
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

[[package]]
name = "wgpu-hal"
version = "30.0.1"

[[package]]
name = "log"
version = "0.4.28"
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

#[test]
fn the_wgpu_stack_is_patched_from_the_fork_checkout() {
    let config = patch_config(Path::new(WS), Path::new(WGPU));
    let value: toml::Value = config.parse().unwrap();
    let patch = value["patch"]["crates-io"].as_table().unwrap();
    assert_eq!(
        patch["wgpu-hal"]["path"].as_str(),
        Some("/work/wgpu/wgpu-hal"),
        "the TV changes"
    );
    // wgpu-hal's path dependencies, so the graph holds one copy of each.
    for (package, dir) in [
        ("naga", "naga"),
        ("naga-types", "naga-types"),
        ("wgpu-types", "wgpu-types"),
        ("wgpu-core", "wgpu-core"),
        ("wgpu", "wgpu"),
        (
            "wgpu-core-deps-windows-linux-android",
            "wgpu-core/platform-deps/windows-linux-android",
        ),
    ] {
        assert_eq!(
            patch[package]["path"].as_str(),
            Some(format!("{WGPU}/{dir}").as_str()),
            "{package}"
        );
    }
    assert_eq!(WGPU_BRANCH, "v30-tizen");
    assert_eq!(WGPU_CHECKOUT, "../wgpu");
}

#[test]
fn wgpu_hal_must_resolve_to_the_fork_checkout() {
    let patched = "wgpu-hal v30.0.1 (/work/wgpu/wgpu-hal)\n\u{2514}\u{2500}\u{2500} wgpu-core v30.0.1 (/work/wgpu/wgpu-core)\n";
    assert_eq!(verify_wgpu_hal_tree(patched, Path::new(WGPU)), Ok(()));
    let registry = "wgpu-hal v30.0.1\n\u{2514}\u{2500}\u{2500} wgpu-core v30.0.1\n";
    assert!(verify_wgpu_hal_tree(registry, Path::new(WGPU)).is_err());
    let elsewhere = "wgpu-hal v30.0.1 (/elsewhere/wgpu-hal)\n";
    assert!(verify_wgpu_hal_tree(elsewhere, Path::new(WGPU)).is_err());
    assert!(verify_wgpu_hal_tree("", Path::new(WGPU)).is_err());
}

#[test]
fn unused_wgpu_patches_are_reported_but_desktop_gpui_crates_are_not() {
    let stderr = "\
warning: Patch `gpui-fast-linux v0.1.13 (/work/gpui-fast/target/release-workspace/gpui_linux)` was not used in the crate graph.
warning: Patch `wgpu-core-deps-apple v30.0.1 (/work/wgpu/wgpu-core/platform-deps/apple)` was not used in the crate graph.
help: Check that the patched package version and available features are compatible
";
    assert_eq!(unused_wgpu_patches(stderr), ["wgpu-core-deps-apple"]);
    assert!(unused_wgpu_patches("").is_empty());
}

#[test]
fn wgpu_hal_from_the_registry_or_twice_fails_the_lock() {
    let registry = PATCHED_LOCK.replace(
        "name = \"wgpu-hal\"\nversion = \"30.0.1\"\n",
        "name = \"wgpu-hal\"\nversion = \"30.0.1\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
    );
    assert!(
        verify_lock(&registry)
            .unwrap_err()
            .contains("wgpu-hal still comes from")
    );
    let twice = format!("{PATCHED_LOCK}\n[[package]]\nname = \"wgpu-hal\"\nversion = \"30.0.0\"\n");
    assert!(verify_lock(&twice).unwrap_err().contains("more than once"));
    let missing = PATCHED_LOCK.replace("name = \"wgpu-hal\"", "name = \"wgpu-hal-x\"");
    assert!(
        verify_lock(&missing)
            .unwrap_err()
            .contains("wgpu-hal is not in the graph")
    );
}

#[test]
fn tv_builds_take_the_patch_config_once() {
    let config = Path::new("/repo/artifacts-temp/gpui-fast-patch.toml");
    let args = |list: &[&str]| -> Vec<String> { list.iter().map(|s| (*s).to_owned()).collect() };
    assert_eq!(
        patched_cargo_args(&args(&["-p", "example_counter"]), config),
        args(&[
            "--config",
            "/repo/artifacts-temp/gpui-fast-patch.toml",
            "-p",
            "example_counter"
        ])
    );
    let already = args(&["--config", "artifacts-temp/gpui-fast-patch.toml", "-p", "x"]);
    assert_eq!(patched_cargo_args(&already, config), already);
    assert!(uses_patch_config(&patched_cargo_args(&[], config)));
}

/// `--from`: CI writes the patch config for the `tizen-release` and `v30-tizen` checkouts it made,
/// without Bun and without generating a release.
mod from_checkout {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn the_three_modes_parse() {
        assert_eq!(parse_mode(&[]), Ok(Mode::Commit));
        assert_eq!(parse_mode(&args(&["--verify"])), Ok(Mode::Verify));
        assert_eq!(
            parse_mode(&args(&["--from", "forks/gpui-fast"])),
            Ok(Mode::From {
                release_workspace: "forks/gpui-fast".into(),
                wgpu: None
            })
        );
        assert_eq!(
            parse_mode(&args(&[
                "--from",
                "forks/gpui-fast",
                "--wgpu",
                "forks/wgpu"
            ])),
            Ok(Mode::From {
                release_workspace: "forks/gpui-fast".into(),
                wgpu: Some("forks/wgpu".into())
            })
        );
    }

    #[test]
    fn anything_else_is_the_usage() {
        for bad in [
            &["--from"][..],
            &["--from", "--wgpu", "x"],
            &["--from", "a", "--wgpu"],
            &["--from", "a", "--wgpu", "--verify"],
            &["--verify", "--from", "a"],
            &["--wgpu", "x"],
            &["release"],
        ] {
            assert_eq!(parse_mode(&args(bad)), Err(USAGE.to_owned()), "{bad:?}");
        }
    }

    #[test]
    fn a_checked_out_release_patches_its_crate_directories() {
        let config = patch_config(
            Path::new("/ci/forks/gpui-fast"),
            Path::new("/ci/forks/wgpu"),
        );
        let value: toml::Value = config.parse().unwrap();
        let patch = value["patch"]["crates-io"].as_table().unwrap();
        assert_eq!(
            patch["gpui-fast"]["path"].as_str(),
            Some("/ci/forks/gpui-fast/gpui")
        );
        assert_eq!(
            patch["wgpu-hal"]["path"].as_str(),
            Some("/ci/forks/wgpu/wgpu-hal")
        );
    }
}
