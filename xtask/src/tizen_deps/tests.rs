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
