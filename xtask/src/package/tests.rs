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
    assert!(App::find("nope").is_err());
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
        &[],
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

#[test]
fn the_video_app_ships_its_clip_in_res() {
    let app = App::find("video").unwrap();
    assert_eq!(app.package, "example_video");
    assert_eq!(app.app_id(), "DxnGpuiTzV.Video");
    validate_package_id(app.package_id).unwrap();
    let root = workspace_root();
    let resources: Vec<PathBuf> = app.resources.iter().map(|file| root.join(file)).collect();
    for file in &resources {
        assert!(file.is_file(), "{} is missing", file.display());
    }
    let entries = layout(
        &[],
        Path::new("/lib/libexample_video.so"),
        &resources,
        Path::new("/icon.png"),
        String::new(),
    );
    assert!(entries.contains(&Entry::Copy {
        from: root.join("examples/video/res/clip.mp4"),
        to: "res/clip.mp4".into()
    }));
    let others = APPS.iter().filter(|a| a.name != "video");
    assert!(others.clone().all(|a| a.resources.is_empty()));
}

/// Seller Office's multi-architecture upload (launch checklist): one package per architecture,
/// a fourth version number unique to each, its architecture in the manifest, at most ten
/// packages of one type in the `.zip`.
#[test]
fn multi_arch() {
    let app = App::find("video").unwrap();
    assert_eq!(STORE_TARGETS, [Target::Armv7, Target::Aarch64]);
    let numbers: std::collections::BTreeSet<u32> =
        Target::ALL.iter().map(|t| arch_number(*t)).collect();
    assert_eq!(numbers.len(), Target::ALL.len(), "unique per architecture");

    let manifests: Vec<String> = STORE_TARGETS
        .iter()
        .map(|target| {
            render_manifest(
                &template(),
                &ManifestValues::for_architecture(app, "0.1.0", *target),
            )
            .unwrap()
        })
        .collect();
    assert!(
        manifests[0].contains(r#"version="0.1.0.1""#),
        "{}",
        manifests[0]
    );
    assert!(manifests[1].contains(r#"version="0.1.0.2""#));
    assert!(manifests[0].contains(
        r#"<feature name="http://tizen.org/feature/platform.core.cpu.arch.armv7">true</feature>"#
    ));
    assert!(manifests[1].contains(
        r#"<feature name="http://tizen.org/feature/platform.core.cpu.arch.armv8">true</feature>"#
    ));
    assert!(manifests[1].trim_end().ends_with("</manifest>"));
    let single = render_manifest(&template(), &ManifestValues::for_app(app, "0.1.0")).unwrap();
    assert!(single.contains(r#"version="0.1.0""#) && !single.contains("<feature"));

    let packages = [
        (Target::Armv7, PathBuf::from("/t/video-armv7.tpk")),
        (Target::Aarch64, PathBuf::from("/t/video-aarch64.tpk")),
    ];
    assert_eq!(
        multi_arch_entries(app, "0.1.0", &packages).unwrap(),
        [
            (
                PathBuf::from("/t/video-armv7.tpk"),
                "DxnGpuiTzV-0.1.0.1-armv7.tpk".to_owned()
            ),
            (
                PathBuf::from("/t/video-aarch64.tpk"),
                "DxnGpuiTzV-0.1.0.2-aarch64.tpk".to_owned()
            ),
        ]
    );
    assert_eq!(multi_arch_zip_name(app, "0.1.0"), "DxnGpuiTzV-0.1.0.zip");
    assert!(multi_arch_entries(app, "0.1.0", &[]).is_err());
    let twice = [packages[0].clone(), packages[0].clone()];
    assert!(multi_arch_entries(app, "0.1.0", &twice).is_err());
    let eleven: Vec<_> = (0..11).map(|_| packages[0].clone()).collect();
    assert!(multi_arch_entries(app, "0.1.0", &eleven).is_err());

    validate_version("255.255.65535", false).unwrap();
    validate_version("1.2.3.99999", true).unwrap();
    for bad in ["256.0.0", "1.2.65536", "1.2", "1.2.3.4"] {
        assert!(validate_version(bad, false).is_err(), "{bad}");
    }
    assert!(validate_version("1.2.3.100000", true).is_err());
    assert!(validate_version("1.2.3", true).is_err());
}
