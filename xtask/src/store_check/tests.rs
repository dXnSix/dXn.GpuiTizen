use super::*;
use crate::package::{App, ManifestValues, render_manifest};
use crate::targets::Target;
use crate::workspace_root;

const DESCRIPTION: &str = "# GpuiTizen Video\n\n## What the app does\n\nPlays a clip.\n\n\
## Privileges\n\n- `http://tizen.org/privilege/internet`: streams.\n";

fn manifest(values: &ManifestValues) -> String {
    let template =
        std::fs::read_to_string(workspace_root().join("host/tizen-manifest.template.xml")).unwrap();
    render_manifest(&template, values).unwrap()
}

/// A package as `cargo xtask package` stages it.
fn package() -> Files {
    let app = App::find("video").unwrap();
    let mut files = Files::new();
    files.insert(
        "tizen-manifest.xml".into(),
        manifest(&ManifestValues::for_app(app, "0.1.0")).into_bytes(),
    );
    files.insert("bin/Dxn.GpuiTizen.Host.dll".into(), vec![0; 64]);
    files.insert("lib/libdxn_app.so".into(), vec![0x7f, b'E', b'L', b'F']);
    files.insert("res/clip.mp4".into(), vec![0; 1024]);
    files.insert(
        "shared/res/icon.png".into(),
        std::fs::read(workspace_root().join("host/shared/res/icon.png")).unwrap(),
    );
    files
}

fn run(files: &Files, description: Option<&str>) -> Report {
    check(
        &Inputs {
            files,
            feature_description: description,
            feature_description_path: "store/feature-description.video.md",
        },
        |_, _| Ok(Vec::new()),
    )
}

fn set_manifest(files: &mut Files, edit: impl FnOnce(String) -> String) {
    let text = String::from_utf8(files["tizen-manifest.xml"].clone()).unwrap();
    files.insert("tizen-manifest.xml".into(), edit(text).into_bytes());
}

#[test]
fn a_staged_package_passes() {
    let report = run(&package(), Some(DESCRIPTION));
    assert!(report.passed(), "{report}");
    for check in [
        "package id",
        "version",
        "api version",
        "profile",
        "app id",
        "exec",
        "icon",
        "feature description",
        "privileges",
        "symbols",
        "size",
    ] {
        assert_eq!(
            report.outcome(check),
            Some(Outcome::Ok),
            "{check}: {report}"
        );
    }
    assert!(
        report
            .to_string()
            .ends_with("store check passed (0 warning(s))\n")
    );
}

#[test]
fn a_multi_architecture_package_passes_too() {
    let mut files = package();
    let app = App::find("video").unwrap();
    files.insert(
        "tizen-manifest.xml".into(),
        manifest(&ManifestValues::for_architecture(
            app,
            "0.1.0",
            Target::Aarch64,
        ))
        .into_bytes(),
    );
    let report = run(&files, Some(DESCRIPTION));
    assert!(report.passed(), "{report}");
}

#[test]
fn each_manifest_field_fails_on_its_own() {
    type Edit = fn(String) -> String;
    let cases: [(&str, Edit); 6] = [
        ("package id", |m| m.replace("DxnGpuiTzV", "Dxn-Video")),
        ("version", |m| {
            m.replace(r#"version="0.1.0""#, r#"version="0.1""#)
        }),
        ("api version", |m| {
            m.replace(r#"api-version="8.0""#, r#"api-version="6.0""#)
        }),
        ("profile", |m| m.replace("tv-samsung", "mobile")),
        ("app id", |m| m.replace("DxnGpuiTzV.Video", "Other.Video")),
        ("exec", |m| {
            m.replace("Dxn.GpuiTizen.Host.dll", "Missing.dll")
        }),
    ];
    for (check, edit) in cases {
        let mut files = package();
        set_manifest(&mut files, edit);
        let report = run(&files, Some(DESCRIPTION));
        assert!(!report.passed(), "{check}");
        assert_eq!(
            report.outcome(check),
            Some(Outcome::Fail),
            "{check}: {report}"
        );
    }
    let mut files = package();
    files.remove("tizen-manifest.xml");
    let report = run(&files, Some(DESCRIPTION));
    assert_eq!(report.outcome("manifest"), Some(Outcome::Fail));
}

#[test]
fn the_icon_must_be_a_512_by_423_png() {
    let mut files = package();
    files.remove("shared/res/icon.png");
    assert_eq!(
        run(&files, Some(DESCRIPTION)).outcome("icon"),
        Some(Outcome::Fail)
    );

    let mut small = files["res/clip.mp4"].clone();
    small.clear();
    small.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    small.extend_from_slice(&[0, 0, 0, 13]);
    small.extend_from_slice(b"IHDR");
    small.extend_from_slice(&117u32.to_be_bytes());
    small.extend_from_slice(&117u32.to_be_bytes());
    assert_eq!(png_size(&small), Some((117, 117)));
    files.insert("shared/res/icon.png".into(), small);
    let report = run(&files, Some(DESCRIPTION));
    assert_eq!(report.outcome("icon"), Some(Outcome::Fail));
    assert!(
        report.to_string().contains("117×117 (512×423 required)"),
        "{report}"
    );

    files.insert("shared/res/icon.png".into(), b"GIF89a".to_vec());
    assert_eq!(
        run(&files, Some(DESCRIPTION)).outcome("icon"),
        Some(Outcome::Fail)
    );
}

#[test]
fn privileges_must_be_declared_in_the_feature_description() {
    let undeclared = "# App\n\n## Privileges\n\nNone.\n";
    let report = run(&package(), Some(undeclared));
    assert_eq!(
        report.outcome("privileges"),
        Some(Outcome::Fail),
        "{report}"
    );
    assert!(report.to_string().contains("internet is not declared"));

    let mut files = package();
    set_manifest(&mut files, |m| {
        m.replace(
            "</privileges>",
            "  <privilege>http://tizen.org/privilege/mediastorage</privilege>\n  </privileges>",
        )
    });
    let report = run(&files, Some(DESCRIPTION));
    assert_eq!(report.outcome("privileges"), Some(Outcome::Fail));

    assert_eq!(
        declared_privileges(
            "## Other\n- http://tizen.org/privilege/camera\n## Privileges\n\
             - `http://tizen.org/privilege/internet`, http://tizen.org/privilege/tv.inputdevice\n"
        ),
        [
            "http://tizen.org/privilege/internet",
            "http://tizen.org/privilege/tv.inputdevice"
        ],
        "only the Privileges section"
    );
}

#[test]
fn the_feature_description_is_required() {
    let report = run(&package(), None);
    assert_eq!(
        report.outcome("feature description"),
        Some(Outcome::Fail),
        "{report}"
    );
    assert!(
        report
            .to_string()
            .contains("store/feature-description.video.md is missing")
    );
    assert_eq!(
        run(&package(), Some("  \n")).outcome("feature description"),
        Some(Outcome::Fail)
    );
    assert_eq!(
        feature_description_for("DxnGpuiTzV").as_deref(),
        Some("store/feature-description.video.md")
    );
    assert_eq!(feature_description_for("NshSomeApp"), None);
}

#[test]
fn symbol_violations_and_missing_libraries_fail() {
    let files = package();
    let report = check(
        &Inputs {
            files: &files,
            feature_description: Some(DESCRIPTION),
            feature_description_path: "store/feature-description.video.md",
        },
        |name, _| {
            assert_eq!(name, "lib/libdxn_app.so");
            Ok(vec![
                "`helper` is exported; only the entry symbol may be".into(),
            ])
        },
    );
    assert_eq!(report.outcome("symbols"), Some(Outcome::Fail));
    assert!(
        report
            .to_string()
            .contains("lib/libdxn_app.so: `helper` is exported")
    );

    let mut files = package();
    files.remove("lib/libdxn_app.so");
    assert_eq!(
        run(&files, Some(DESCRIPTION)).outcome("symbols"),
        Some(Outcome::Fail)
    );
}

#[test]
fn sizes_are_reported_and_large_packages_warned_about() {
    let mut files = package();
    files.insert(
        "res/large.bin".into(),
        vec![0; (PACKAGE_BUDGET + 1) as usize],
    );
    let report = run(&files, Some(DESCRIPTION));
    assert!(report.passed(), "a warning, not a failure");
    assert_eq!(report.outcome("size"), Some(Outcome::Warn));
    assert!(report.to_string().contains("1 warning(s)"));
    assert!(
        report
            .items("size")
            .any(|i| i.detail.starts_with("lib/libdxn_app.so: 0.0 MiB"))
    );
}

#[test]
fn every_app_has_a_feature_description() {
    for app in crate::package::APPS {
        let path = workspace_root().join(format!("store/feature-description.{}.md", app.name));
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} is missing", path.display()));
        assert!(
            declared_privileges(&text) == ["http://tizen.org/privilege/internet"],
            "{}: the host's manifest asks for internet",
            path.display()
        );
    }
}

#[test]
fn manifests_are_read_past_comments_quotes_and_spaces() {
    let mut files = package();
    set_manifest(&mut files, |m| {
        m.replace(
            r#"<profile name="tv-samsung" />"#,
            "<!-- <profile name=\"mobile\" /> -->\n  <profile name = 'tv-samsung' />",
        )
        .replace(
            "<privilege>http://tizen.org/privilege/internet</privilege>",
            "<privilege >http://tizen.org/privilege/internet</privilege>\n    \
             <!-- <privilege>http://tizen.org/privilege/camera</privilege> -->",
        )
    });
    let report = run(&files, Some(DESCRIPTION));
    assert!(report.passed(), "{report}");
    assert_eq!(report.outcome("profile"), Some(Outcome::Ok));

    // A privilege written with a space before `>` is still found: undeclared, it fails.
    let mut files = package();
    set_manifest(&mut files, |m| {
        m.replace(
            "</privileges>",
            "  <privilege >http://tizen.org/privilege/camera</privilege>\n  </privileges>",
        )
    });
    assert_eq!(
        run(&files, Some(DESCRIPTION)).outcome("privileges"),
        Some(Outcome::Fail)
    );
}

#[test]
fn samsung_privileges_count_when_declared() {
    let tv = "http://developer.samsung.com/privilege/productinfo";
    let mut files = package();
    set_manifest(&mut files, |m| {
        m.replace(
            "</privileges>",
            &format!("  <privilege>{tv}</privilege>\n  </privileges>"),
        )
    });
    let declared = format!("{DESCRIPTION}- `{tv}`: the model name.\n");
    let report = run(&files, Some(&declared));
    assert!(report.passed(), "{report}");
    assert_eq!(
        run(&files, Some(DESCRIPTION)).outcome("privileges"),
        Some(Outcome::Fail)
    );
}

#[test]
fn an_unfilled_template_fails() {
    let template =
        std::fs::read_to_string(workspace_root().join("store/feature-description.counter.md"))
            .unwrap();
    assert!(
        !placeholders(&template).is_empty(),
        "the templates have placeholders"
    );
    let report = run(&package(), Some(&template));
    assert_eq!(report.outcome("feature description"), Some(Outcome::Fail));
    assert!(
        report
            .to_string()
            .contains("still holds the template's placeholders")
    );
    assert!(
        placeholders("See [the guide](https://example.com), [x] done, [ ] open.").is_empty(),
        "links and checkboxes are not placeholders"
    );
}

/// The start of a little-endian ELF library for `machine`.
fn elf(machine: u16) -> Vec<u8> {
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"\x7fELF");
    bytes[4] = 1;
    bytes[5] = 1;
    bytes[18..20].copy_from_slice(&machine.to_le_bytes());
    bytes
}

fn arch_package(target: Target, machine: u16) -> Files {
    let app = App::find("video").unwrap();
    let mut files = package();
    files.insert(
        "tizen-manifest.xml".into(),
        manifest(&ManifestValues::for_architecture(app, "0.1.0", target)).into_bytes(),
    );
    files.insert("lib/libdxn_app.so".into(), elf(machine));
    files
}

fn upload(packages: &[(&str, Files)]) -> Report {
    let packages: Vec<UploadPackage<'_>> = packages
        .iter()
        .map(|(name, files)| UploadPackage { name, files })
        .collect();
    check_upload(&packages, |files| run(files, Some(DESCRIPTION)))
}

#[test]
fn a_multi_architecture_upload_is_checked_as_one() {
    let armv7 = arch_package(Target::Armv7, 40);
    let aarch64 = arch_package(Target::Aarch64, 183);
    let report = upload(&[
        ("v-armv7.tpk", armv7.clone()),
        ("v-aarch64.tpk", aarch64.clone()),
    ]);
    assert!(report.passed(), "{report}");
    assert!(
        report
            .to_string()
            .contains("v-aarch64.tpk: `DxnGpuiTzV` (10 letters and digits)")
    );

    // A library of the wrong architecture for its CPU feature.
    let wrong = arch_package(Target::Aarch64, 40);
    assert!(!upload(&[("a.tpk", armv7.clone()), ("b.tpk", wrong)]).passed());

    // Two packages with the same fourth number.
    let report = upload(&[("a.tpk", armv7.clone()), ("b.tpk", armv7.clone())]);
    assert!(
        report
            .to_string()
            .contains("FAIL upload: fourth version numbers unique"),
        "{report}"
    );

    // Different versions in one upload.
    let mut other = aarch64.clone();
    let text = String::from_utf8(other["tizen-manifest.xml"].clone()).unwrap();
    other.insert(
        "tizen-manifest.xml".into(),
        text.replace("0.1.0.2", "0.2.0.2").into_bytes(),
    );
    assert!(!upload(&[("a.tpk", armv7.clone()), ("b.tpk", other)]).passed());

    // A package without the fourth number, and an empty or oversized upload.
    assert!(!upload(&[("a.tpk", package())]).passed());
    assert!(!upload(&[]).passed());
    let eleven: Vec<(&str, Files)> = (0..11).map(|_| ("a.tpk", armv7.clone())).collect();
    assert!(!upload(&eleven).passed());

    assert_eq!(elf_machine(&elf(183)), Some(183));
    assert_eq!(elf_machine(b"not elf"), None);
}
