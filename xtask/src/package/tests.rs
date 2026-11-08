use std::path::{Path, PathBuf};

use super::*;
use crate::workspace_root;

fn template() -> String {
    std::fs::read_to_string(workspace_root().join("host/tizen-manifest.template.xml"))
        .expect("the host's manifest template")
}

#[test]
fn app_table_and_package_ids() {
    assert_eq!(App::find("diag").unwrap().package, "dxn_gpui_tizen_diag");
    assert_eq!(App::find("counter").unwrap().package_id, "DxnGpuiTzC");
    assert_eq!(
        App::find("long_list").unwrap().app_id(),
        "DxnGpuiTzL.LongList"
    );
    assert!(App::find("video").is_err());
    for app in APPS {
        validate_package_id(app.package_id).unwrap();
    }
    for bad in ["DxnGpuiTz", "DxnGpuiTzDD", "DxnGpui-zD", "DxnGpuiTž", ""] {
        assert!(
            validate_package_id(bad).is_err(),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn rendered_manifest_has_the_required_elements() {
    let app = App::find("diag").unwrap();
    let manifest = render_manifest(&template(), &ManifestValues::for_app(app, "0.1.0")).unwrap();
    for expected in [
        r#"package="DxnGpuiTzD""#,
        r#"version="0.1.0""#,
        r#"api-version="8.0""#,
        r#"<profile name="tv-samsung" />"#,
        r#"appid="DxnGpuiTzD.Diag""#,
        r#"exec="Dxn.GpuiTizen.Host.dll""#,
        r#"type="dotnet""#,
        r#"multiple="false""#,
        r#"<metadata key="http://tizen.org/metadata/prefer_dotnet_aot" value="true" />"#,
        "<privilege>http://tizen.org/privilege/internet</privilege>",
        "<label>GpuiTizen Diag</label>",
    ] {
        assert!(
            manifest.contains(expected),
            "{expected} missing from\n{manifest}"
        );
    }
    assert!(!manifest.contains("{{"), "{manifest}");
    assert!(
        !manifest.contains("<!--"),
        "the template's comment is dropped"
    );
}

#[test]
fn manifest_rendering_rejects_bad_input() {
    let app = App::find("counter").unwrap();
    let mut values = ManifestValues::for_app(app, "0.1.0");
    values.package_id = "TooShort".into();
    assert!(render_manifest(&template(), &values).is_err());
    let values = ManifestValues::for_app(app, "0.1");
    assert!(render_manifest(&template(), &values).is_err());
    let values = ManifestValues::for_app(app, "1.2.3");
    let err = render_manifest("<x>{{UNKNOWN}}</x>", &values)
        .unwrap_err()
        .to_string();
    assert!(err.contains("{{UNKNOWN}}"), "{err}");
    let mut values = ManifestValues::for_app(app, "1.2.3");
    values.label = "A & <B>".into();
    let manifest = render_manifest(&template(), &values).unwrap();
    assert!(
        manifest.contains("<label>A &amp; &lt;B&gt;</label>"),
        "{manifest}"
    );
}

#[test]
fn staged_layout_follows_the_tpk_tree() {
    let root = Path::new("/repo");
    let app = App::find("counter").unwrap();
    let staging = staging_dir(root, app, Target::Armv7);
    assert_eq!(staging, PathBuf::from("/repo/target/tpk/counter-armv7"));
    assert_eq!(
        staging_dir(root, App::find("diag").unwrap(), Target::X86_64),
        PathBuf::from("/repo/target/tpk/diag-x86_64")
    );
    let host = host_files(Path::new("/repo/host/bin/Release/tizen80"));
    let entries = layout(
        &host,
        Path::new("/repo/target/armv7-tizen-linux-gnueabi/release/libexample_counter.so"),
        Path::new("/repo/host/shared/res/icon.png"),
        "<manifest/>".into(),
    );
    assert_eq!(
        entries,
        [
            Entry::Copy {
                from: "/repo/host/bin/Release/tizen80/Dxn.GpuiTizen.Host.dll".into(),
                to: "bin/Dxn.GpuiTizen.Host.dll".into()
            },
            Entry::Copy {
                from: "/repo/host/bin/Release/tizen80/Dxn.GpuiTizen.Host.deps.json".into(),
                to: "bin/Dxn.GpuiTizen.Host.deps.json".into()
            },
            Entry::Copy {
                from: "/repo/target/armv7-tizen-linux-gnueabi/release/libexample_counter.so".into(),
                to: "lib/libdxn_app.so".into()
            },
            Entry::Dir("res".into()),
            Entry::Copy {
                from: "/repo/host/shared/res/icon.png".into(),
                to: "shared/res/icon.png".into()
            },
            Entry::Write {
                to: "tizen-manifest.xml".into(),
                contents: "<manifest/>".into()
            },
        ]
    );
}

#[test]
fn signing_runs_tz_pack_with_the_profile_only() {
    assert_eq!(
        sign_command(Path::new("/repo/target/tpk/diag-armv7"), "owner"),
        [
            "tz",
            "pack",
            "-t",
            "tpk",
            "-s",
            "owner",
            "/repo/target/tpk/diag-armv7"
        ]
    );
    assert!(UNSIGNED_NOTE.contains("certificate profile"));
}

#[test]
fn the_host_files_exist_in_the_repository() {
    let host = workspace_root().join("host");
    for file in [
        "Program.cs",
        "Dxn.GpuiTizen.Host.csproj",
        "tizen-manifest.template.xml",
        "shared/res/icon.png",
    ] {
        assert!(host.join(file).is_file(), "host/{file} is missing");
    }
    let program = std::fs::read_to_string(host.join("Program.cs")).unwrap();
    assert_eq!(program.matches("DllImport(").count(), 1, "one P/Invoke");
    assert!(program.contains(r#"DllImport("libdxn_app.so", ExactSpelling = true)"#));
    assert!(program.contains("dxn_gpui_tizen_main(int argc, IntPtr* argv)"));
}
