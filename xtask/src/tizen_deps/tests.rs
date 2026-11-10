use super::*;

const TIZEN_TREE: &str = "\
dxn_gpui_tizen v0.1.0 (/repo/crates/dxn_gpui_tizen)
gpui-fast v0.1.13
gpui-fast-wgpu v0.1.13
wgpu v30.0.1
wgpu-hal v30.0.1
wayland-sys v0.31.11
cosmic-text v0.19.0
";

const DIAG_TREE: &str = "\
dxn_gpui_tizen_diag v0.1.0 (/repo/crates/dxn_gpui_tizen_diag)
dxn_gpui_tizen_sys v0.1.0 (/repo/crates/dxn_gpui_tizen_sys)
wgpu v30.0.1
wgpu-core v30.0.1
libloading v0.8.9
";

#[test]
fn a_clean_tizen_graph_passes() {
    let packages = parse_tree(TIZEN_TREE);
    assert!(packages.contains(&("wgpu".into(), "30.0.1".into())));
    assert!(desktop_crates(&packages).is_empty());
    assert!(diag_violations(&parse_tree(DIAG_TREE), Some("30.0.1")).is_empty());
    assert_eq!(single_wgpu(&packages), Ok("30.0.1".into()));
}

#[test]
fn desktop_platform_crates_are_reported() {
    let tree = format!(
        "{TIZEN_TREE}gpui-fast-platform v0.1.13\ngpui-fast-linux v0.1.13\nfreetype-sys v0.20.1\nyeslogic-fontconfig-sys v6.0.1\n"
    );
    assert_eq!(
        desktop_crates(&parse_tree(&tree)),
        [
            "freetype-sys 0.20.1",
            "gpui-fast-linux 0.1.13",
            "gpui-fast-platform 0.1.13",
            "yeslogic-fontconfig-sys 6.0.1"
        ]
    );
}

#[test]
fn the_diag_rejects_gpui_and_a_second_or_other_wgpu() {
    let tree = format!("{DIAG_TREE}gpui-fast v0.1.13\n");
    assert_eq!(
        diag_violations(&parse_tree(&tree), Some("30.0.1")),
        ["GPUI crate gpui-fast 0.1.13 in the diagnostic app"]
    );
    let tree = format!("{DIAG_TREE}wgpu v29.0.0\n");
    assert_eq!(
        diag_violations(&parse_tree(&tree), Some("30.0.1")),
        ["2 wgpu versions: 29.0.0, 30.0.1"]
    );
    let tree = DIAG_TREE.replace("wgpu v30.0.1", "wgpu v31.0.0");
    assert_eq!(
        diag_violations(&parse_tree(&tree), Some("30.0.1")),
        ["the diagnostic app's wgpu 31.0.0 is not the workspace's wgpu 30.0.1"]
    );
    let tree = DIAG_TREE.replace("wgpu v30.0.1\n", "");
    assert_eq!(
        diag_violations(&parse_tree(&tree), Some("30.0.1")),
        ["no wgpu in the diagnostic app"]
    );
}

#[test]
fn the_workspace_graph_holds_one_wgpu() {
    let tree = format!("{TIZEN_TREE}wgpu v31.0.0\n");
    assert_eq!(
        single_wgpu(&parse_tree(&tree)),
        Err("2 wgpu versions in the Tizen graph: 30.0.1, 31.0.0".into())
    );
    assert!(single_wgpu(&parse_tree("gpui-fast v0.1.13\n")).is_err());
}

#[test]
fn duplicate_lines_collapse() {
    let packages = parse_tree("wgpu v30.0.1\nwgpu v30.0.1 (*)\n\n  \n");
    assert_eq!(packages.len(), 1);
}

/// `--manifest-path`, `-p`, the features and `--single`: NSH's GPUI workspace checked from here.
mod foreign {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    const LOCK: &str = r#"
version = 4

[[package]]
name = "wgpu"
version = "30.0.1"

[[package]]
name = "wgpu-hal"
version = "30.0.1"

[[package]]
name = "naga"
version = "29.0.0"

[[package]]
name = "naga"
version = "30.0.0"
"#;

    #[test]
    fn this_workspace_takes_only_a_target() {
        assert_eq!(
            parse_check(&args(&["--target", "armv7"])),
            Ok(Check::Graph {
                target: "armv7".into(),
                manifest_path: None,
                package: None,
                no_default_features: false,
                features: None,
            })
        );
        for narrowed in [
            &["--target", "armv7", "-p", "x"][..],
            &["--target", "armv7", "--no-default-features"],
            &["--target", "armv7", "--features", "js"],
        ] {
            assert_eq!(
                parse_check(&args(narrowed)),
                Err(USAGE.to_owned()),
                "{narrowed:?}"
            );
        }
    }

    #[test]
    fn another_workspace_narrows_its_graph() {
        assert_eq!(
            parse_check(&args(&[
                "--target",
                "armv7",
                "--manifest-path",
                "../nsh/crates/gpui/Cargo.toml",
                "-p",
                "nsh-platform-gpui",
                "--no-default-features",
                "--features",
                "js",
            ])),
            Ok(Check::Graph {
                target: "armv7".into(),
                manifest_path: Some("../nsh/crates/gpui/Cargo.toml".into()),
                package: Some("nsh-platform-gpui".into()),
                no_default_features: true,
                features: Some("js".into()),
            })
        );
    }

    #[test]
    fn single_reads_a_lockfile() {
        assert_eq!(
            parse_check(&args(&[
                "--manifest-path",
                "m/Cargo.toml",
                "--single",
                "wgpu"
            ])),
            Ok(Check::Single {
                crate_name: "wgpu".into(),
                manifest_path: Some("m/Cargo.toml".into()),
            })
        );
        for bad in [
            &["--single", "wgpu", "--target", "armv7"][..],
            &["--single", "wgpu", "--features", "js"],
            &["--single"],
            &["--single", "--target"],
            &[],
            &["--target"],
            &["--bogus"],
        ] {
            assert_eq!(parse_check(&args(bad)), Err(USAGE.to_owned()), "{bad:?}");
        }
    }

    #[test]
    fn one_version_in_the_lock_passes_and_two_fail() {
        assert_eq!(single_in_lock(LOCK, "wgpu"), Ok("30.0.1".to_owned()));
        assert_eq!(lock_versions(LOCK, "naga").unwrap(), ["29.0.0", "30.0.0"]);
        let err = single_in_lock(LOCK, "naga").unwrap_err();
        assert!(
            err.contains("2 naga versions") && err.contains("29.0.0, 30.0.0"),
            "{err}"
        );
        assert_eq!(
            single_in_lock(LOCK, "ash"),
            Err("no ash in the lockfile".to_owned())
        );
        assert!(single_in_lock("not = [toml", "wgpu").is_err());
    }

    #[test]
    fn a_foreign_graph_allows_no_wgpu_but_not_two() {
        let one = parse_tree("nsh-platform-gpui v0.4.0\nwgpu v30.0.1\n");
        assert!(foreign_violations(&one).is_empty());
        let none = parse_tree("nsh-core v0.4.0\n");
        assert!(foreign_violations(&none).is_empty());
        let two = parse_tree("wgpu v29.0.0\nwgpu v30.0.1\ngpui-fast-linux v0.1.13\n");
        let violations = foreign_violations(&two);
        assert_eq!(violations.len(), 2, "{violations:?}");
        assert!(violations.iter().any(|v| v.contains("gpui-fast-linux")));
    }
}

/// GTK, WebKitGTK, GStreamer and wry (NSH's Linux toolkit) are desktop crates too.
mod gui_toolkits {
    use super::*;

    #[test]
    fn toolkit_crates_are_desktop_crates() {
        for name in [
            "gtk4",
            "gtk4-sys",
            "gdk4",
            "gdk-pixbuf",
            "gio",
            "glib-sys",
            "gobject-sys",
            "pango",
            "cairo-rs",
            "webkit6",
            "webkit2gtk",
            "javascriptcore6",
            "soup3",
            "gstreamer",
            "gstreamer-video",
            "wry",
            "x11rb",
            "gpui-fast-linux",
        ] {
            assert!(is_desktop_crate(name), "{name}");
        }
    }

    #[test]
    fn crates_that_only_share_a_prefix_are_not() {
        for name in [
            "glob",
            "gimli",
            "gif",
            "gioxyz",
            "wgpu",
            "gpui-fast",
            "cairo_like",
            "wrynot",
            "softbuffer",
            "pangocairo_x",
        ] {
            assert!(!is_desktop_crate(name), "{name}");
        }
    }

    #[test]
    fn a_graph_with_gtk_reports_it() {
        let graph =
            parse_tree("nsh-platform-gpui v0.4.0\ngtk4 v0.10.0\ngstreamer v0.24.0\nwgpu v30.0.1\n");
        assert_eq!(
            foreign_violations(&graph),
            ["gstreamer 0.24.0", "gtk4 0.10.0"]
        );
    }
}
