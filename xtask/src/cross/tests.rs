use std::path::{Path, PathBuf};

use serde_json::json;

use super::*;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn ctx(rootstrap: Option<&str>, needs_link: bool) -> Context {
    Context {
        root: PathBuf::from("/repo"),
        toolchain: "nightly-2026-10-18".into(),
        rustlib_bin: PathBuf::from("/tc/lib/rustlib/aarch64-apple-darwin/bin"),
        clang: "clang".into(),
        rootstrap: rootstrap.map(PathBuf::from),
        rootstrap_searched: vec![PathBuf::from(
            "/home/me/tizen-studio/platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-device.core",
        )],
        gcc_toolchain: rootstrap.map(|_| PathBuf::from("/gcc")),
        gcc_toolchain_searched: Vec::new(),
        needs_link,
        inherited_rustflags: Vec::new(),
        lockfile: None,
    }
}

fn plan_for(list: &[&str], ctx: &Context) -> Invocation {
    let options = Options::parse(&args(list)).expect("options parse");
    plan(
        &options,
        ctx,
        &["dxn_gpui_tizen_diag".into(), "example_counter".into()],
    )
    .expect("plan")
}

#[test]
fn armv7_build_command_and_environment() {
    let inv = plan_for(
        &["--target", "armv7", "-p", "dxn_gpui_tizen"],
        &ctx(Some("/rs/armv7"), true),
    );
    assert_eq!(inv.program, "cargo");
    assert_eq!(
        inv.args,
        args(&[
            "build",
            "-Zbuild-std=std,panic_abort",
            "-Zjson-target-spec",
            "--target",
            "/repo/targets/armv7-tizen-linux-gnueabi.json",
            "-p",
            "dxn_gpui_tizen",
        ])
    );
    assert_eq!(
        inv.env_value("RUSTUP_TOOLCHAIN"),
        Some("nightly-2026-10-18")
    );
    assert_eq!(inv.env_value("CC_armv7_tizen_linux_gnueabi"), Some("clang"));
    assert_eq!(
        inv.env_value("CFLAGS_armv7_tizen_linux_gnueabi"),
        Some(
            "--target=armv7l-tizen-linux-gnueabi --sysroot=/rs/armv7 -march=armv7-a -mfloat-abi=softfp -mfpu=neon"
        )
    );
    assert_eq!(
        inv.env_value("AR_armv7_tizen_linux_gnueabi"),
        Some("/tc/lib/rustlib/aarch64-apple-darwin/bin/llvm-ar")
    );
    assert_eq!(
        inv.env_value("CARGO_TARGET_ARMV7_TIZEN_LINUX_GNUEABI_LINKER"),
        Some("clang")
    );
    assert_eq!(inv.env_remove, ["RUSTFLAGS"]);
    let rustflags = inv
        .env_value("CARGO_ENCODED_RUSTFLAGS")
        .unwrap()
        .replace('\x1f', " ");
    for expected in [
        "-Clink-arg=--target=armv7l-tizen-linux-gnueabi",
        "-Clink-arg=--sysroot=/rs/armv7",
        "-Clink-arg=--ld-path=/gcc/arm-tizen-linux-gnueabi/bin/ld.bfd",
        "-Clink-arg=-mfloat-abi=softfp",
    ] {
        assert!(
            rustflags.contains(expected),
            "{expected} missing from {rustflags}"
        );
    }
}

#[test]
fn x86_64_release_command_and_environment() {
    let inv = plan_for(
        &[
            "--release",
            "--target",
            "x86_64",
            "--features",
            "diag",
            "--no-default-features",
            "-p",
            "example_counter",
        ],
        &ctx(Some("/rs/x86"), true),
    );
    assert_eq!(
        inv.args,
        args(&[
            "build",
            "-Zbuild-std=std,panic_abort",
            "-Zjson-target-spec",
            "--target",
            "/repo/targets/x86_64-tizen-linux-gnu.json",
            "--release",
            "--features",
            "diag",
            "--no-default-features",
            "-p",
            "example_counter",
        ])
    );
    assert_eq!(
        inv.env_value("CFLAGS_x86_64_tizen_linux_gnu"),
        Some("--target=x86_64-tizen-linux-gnu --sysroot=/rs/x86")
    );
    assert_eq!(
        inv.env_value("CARGO_TARGET_X86_64_TIZEN_LINUX_GNU_LINKER"),
        Some("clang")
    );
    let rustflags = inv
        .env_value("CARGO_ENCODED_RUSTFLAGS")
        .unwrap()
        .replace('\x1f', " ");
    assert!(
        rustflags.contains("-Clink-arg=--sysroot=/rs/x86"),
        "{rustflags}"
    );
    assert!(!rustflags.contains("softfp"), "{rustflags}");
}

#[test]
fn clippy_denies_warnings_and_keeps_trailing_args() {
    let inv = plan_for(
        &[
            "--target",
            "armv7",
            "--clippy",
            "-p",
            "xtask",
            "--",
            "-W",
            "clippy::pedantic",
        ],
        &ctx(None, true),
    );
    assert_eq!(inv.args[0], "clippy");
    let tail: Vec<&str> = inv
        .args
        .iter()
        .skip_while(|a| *a != "--")
        .map(String::as_str)
        .collect();
    assert_eq!(tail, ["--", "-W", "clippy::pedantic", "-D", "warnings"]);
}

#[test]
fn missing_rootstrap_fails_before_cargo_for_linking_builds() {
    let options =
        Options::parse(&args(&["--target", "armv7", "-p", "dxn_gpui_tizen_diag"])).unwrap();
    let err = plan(&options, &ctx(None, true), &[])
        .unwrap_err()
        .to_string();
    assert!(err.contains("tizen-8.0-device.core"), "{err}");
    assert!(err.contains("TIZEN_ROOTSTRAP_ARMV7"), "{err}");
    assert!(err.contains("package-manager-cli"), "{err}");
    assert!(err.contains("/home/me/tizen-studio"), "{err}");

    let options = Options::parse(&args(&["--target", "x86_64", "--all-tv-libs"])).unwrap();
    let err = plan(&options, &ctx(None, true), &[])
        .unwrap_err()
        .to_string();
    assert!(err.contains("tizen-8.0-emulator64.core"), "{err}");
    assert!(err.contains("TIZEN_ROOTSTRAP_X86_64"), "{err}");
}

#[test]
fn rlib_builds_and_clippy_need_no_rootstrap() {
    let inv = plan_for(
        &["--target", "armv7", "-p", "dxn_gpui_tizen"],
        &ctx(None, false),
    );
    assert_eq!(
        inv.env_value("CFLAGS_armv7_tizen_linux_gnueabi"),
        Some("--target=armv7l-tizen-linux-gnueabi -march=armv7-a -mfloat-abi=softfp -mfpu=neon")
    );
    let inv = plan_for(
        &["--target", "armv7", "--clippy", "--all-tv-libs"],
        &ctx(None, true),
    );
    assert_eq!(inv.args[0], "clippy");
}

#[test]
fn all_tv_libs_selects_every_cdylib_and_manifest_path_passes_through() {
    let inv = plan_for(
        &["--target", "armv7", "--all-tv-libs"],
        &ctx(Some("/rs"), true),
    );
    let tail: Vec<&str> = inv.args.iter().skip(5).map(String::as_str).collect();
    assert_eq!(tail, ["-p", "dxn_gpui_tizen_diag", "-p", "example_counter"]);

    let inv = plan_for(
        &[
            "--target",
            "armv7",
            "--manifest-path",
            "../nsh/crates/gpui/Cargo.toml",
            "-p",
            "nsh-platform-gpui",
        ],
        &ctx(None, false),
    );
    assert!(
        inv.args
            .windows(2)
            .any(|w| w == ["--manifest-path", "../nsh/crates/gpui/Cargo.toml"])
    );
    assert_eq!(
        inv.env_value("RUSTUP_TOOLCHAIN"),
        Some("nightly-2026-10-18"),
        "always the pinned nightly"
    );

    let options = Options::parse(&args(&[
        "--target",
        "armv7",
        "--all-tv-libs",
        "--manifest-path",
        "x/Cargo.toml",
    ]))
    .unwrap();
    assert!(plan(&options, &ctx(Some("/rs"), true), &[]).is_err());
}

#[test]
fn option_parsing() {
    assert!(
        Options::parse(&args(&["-p", "x"])).is_err(),
        "--target is required"
    );
    assert!(Options::parse(&args(&["--target", "sparc"])).is_err());
    let options = Options::parse(&args(&[
        "--target=x86_64",
        "--check-symbols",
        "-pa",
        "--package",
        "b",
        "--package=c",
        "--manifest-path=m/Cargo.toml",
    ]))
    .unwrap();
    assert_eq!(options.target, Target::X86_64);
    assert!(options.check_symbols);
    assert_eq!(options.packages(), ["a", "b", "c"]);
    assert_eq!(options.manifest_path(), Some("m/Cargo.toml"));
}

#[test]
fn rootstrap_discovery() {
    assert_eq!(
        rootstrap_candidates(
            Target::Armv7,
            Some("/custom"),
            Some("/sdk"),
            Some(Path::new("/home/me"))
        ),
        [PathBuf::from("/custom")]
    );
    assert_eq!(
        rootstrap_candidates(
            Target::X86_64,
            None,
            Some("/sdk"),
            Some(Path::new("/home/me"))
        ),
        [PathBuf::from(
            "/sdk/platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-emulator64.core"
        )]
    );
    assert_eq!(
        rootstrap_candidates(Target::Armv7, Some(""), None, Some(Path::new("/home/me"))),
        [PathBuf::from(
            "/home/me/tizen-studio/platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-device.core"
        )]
    );
    assert!(rootstrap_candidates(Target::Armv7, None, None, None).is_empty());
}

#[test]
fn link_detection_from_cargo_metadata() {
    let metadata = json!({
        "packages": [
            {"id": "sys", "name": "dxn_gpui_tizen_sys", "targets": [{"crate_types": ["lib"]}]},
            {"id": "diag", "name": "dxn_gpui_tizen_diag", "targets": [{"crate_types": ["cdylib", "rlib"]}]},
            {"id": "xt", "name": "xtask", "targets": [{"crate_types": ["lib"]}, {"crate_types": ["bin"]}]}
        ],
        "workspace_members": ["sys", "diag", "xt"],
        "workspace_default_members": ["sys"]
    });
    assert_eq!(cdylib_packages(&metadata), ["dxn_gpui_tizen_diag"]);
    let opts = |list: &[&str]| Options::parse(&args(list)).unwrap();
    assert!(!selection_needs_link(
        &metadata,
        &opts(&["--target", "armv7"]),
        &[]
    ));
    assert!(!selection_needs_link(
        &metadata,
        &opts(&["--target", "armv7", "-p", "dxn_gpui_tizen_sys"]),
        &[]
    ));
    assert!(selection_needs_link(
        &metadata,
        &opts(&["--target", "armv7", "-p", "dxn_gpui_tizen_diag"]),
        &[]
    ));
    assert!(selection_needs_link(
        &metadata,
        &opts(&["--target", "armv7", "--workspace"]),
        &[]
    ));
    assert!(selection_needs_link(
        &metadata,
        &opts(&["--target", "armv7", "--all-tv-libs"]),
        &cdylib_packages(&metadata)
    ));
}

#[test]
fn link_arguments_survive_inherited_rustflags() {
    assert_eq!(
        inherited_rustflags(None, Some("-D warnings")),
        ["-D", "warnings"]
    );
    assert_eq!(
        inherited_rustflags(Some("-Cdebuginfo=0\x1f--cfg=ci"), Some("-D warnings")),
        ["-Cdebuginfo=0", "--cfg=ci"],
        "CARGO_ENCODED_RUSTFLAGS wins, as in Cargo"
    );
    assert!(inherited_rustflags(Some(""), None).is_empty());

    let mut ctx = ctx(Some("/rs/armv7"), true);
    ctx.inherited_rustflags = inherited_rustflags(None, Some("-D warnings"));
    let inv = plan_for(&["--target", "armv7", "-p", "example_counter"], &ctx);
    let flags: Vec<&str> = inv
        .env_value("CARGO_ENCODED_RUSTFLAGS")
        .unwrap()
        .split('\x1f')
        .collect();
    assert_eq!(&flags[..2], ["-D", "warnings"]);
    assert!(
        flags.contains(&"-Clink-arg=--sysroot=/rs/armv7"),
        "{flags:?}"
    );
    assert!(
        flags.contains(&"-Clink-arg=--ld-path=/gcc/arm-tizen-linux-gnueabi/bin/ld.bfd"),
        "{flags:?}"
    );
    assert_eq!(
        inv.env_remove,
        ["RUSTFLAGS"],
        "RUSTFLAGS must not shadow the merged flags"
    );
}

#[test]
fn check_symbols_fails_before_cargo_while_unwired() {
    let options = Options::parse(&args(&[
        "--target",
        "armv7",
        "--check-symbols",
        "-p",
        "example_counter",
    ]))
    .unwrap();
    let err = plan(&options, &ctx(Some("/rs"), true), &[])
        .unwrap_err()
        .to_string();
    assert!(err.contains("check-symbols"), "{err}");
}

#[test]
fn aarch64_uses_its_own_rootstrap_and_no_arm32_flags() {
    let mut ctx = ctx(Some("/rs/aarch64"), true);
    ctx.rootstrap_searched = rootstrap_candidates(Target::Aarch64, None, Some("/sdk"), None);
    let inv = plan_for(&["--target", "aarch64", "-p", "example_counter"], &ctx);
    assert!(
        inv.args
            .contains(&"/repo/targets/aarch64-tizen-linux-gnu.json".to_owned())
    );
    assert_eq!(
        inv.env_value("CFLAGS_aarch64_tizen_linux_gnu"),
        Some("--target=aarch64-tizen-linux-gnu --sysroot=/rs/aarch64")
    );
    assert_eq!(
        inv.env_value("CARGO_TARGET_AARCH64_TIZEN_LINUX_GNU_LINKER"),
        Some("clang")
    );

    let options = Options::parse(&args(&["--target", "aarch64", "-p", "example_counter"])).unwrap();
    let mut missing = ctx.clone();
    missing.rootstrap = None;
    let err = plan(&options, &missing, &[]).unwrap_err().to_string();
    assert!(err.contains("TIZEN_ROOTSTRAP_AARCH64"), "{err}");
    assert!(err.contains("tizen-8.0-device64.core"), "{err}");
}

#[test]
fn foreign_workspaces_keep_their_lockfile_here() {
    let root = Path::new("/work/dXn.GpuiTizen");
    assert_eq!(
        foreign_lockfile(root, Path::new("/work/dXn.GpuiTizen")),
        None
    );
    assert_eq!(
        foreign_lockfile(root, Path::new("/work/nsh/nsh-wt/tizen/crates/gpui")),
        Some(PathBuf::from(
            "/work/dXn.GpuiTizen/artifacts-temp/locks/work_nsh_nsh-wt_tizen_crates_gpui/Cargo.lock"
        ))
    );

    let files = |path: &Path| -> Option<String> {
        match path.to_str()? {
            "/nsh/crates/gpui/Cargo.toml" => {
                Some("[workspace]\nmembers = [\"nsh-platform-gpui\"]\n".into())
            }
            "/nsh/crates/gpui/nsh-platform-gpui/Cargo.toml" => {
                Some("[package]\nname = \"nsh-platform-gpui\"\n".into())
            }
            _ => None,
        }
    };
    assert_eq!(
        workspace_root_of(
            Path::new("/nsh/crates/gpui/nsh-platform-gpui/Cargo.toml"),
            files
        ),
        PathBuf::from("/nsh/crates/gpui")
    );
    assert_eq!(
        workspace_root_of(Path::new("/nsh/crates/gpui/Cargo.toml"), files),
        PathBuf::from("/nsh/crates/gpui")
    );

    let mut ctx = ctx(None, false);
    ctx.lockfile = Some(PathBuf::from("/repo/artifacts-temp/locks/nsh/Cargo.lock"));
    let inv = plan_for(
        &[
            "--target",
            "armv7",
            "--manifest-path",
            "/nsh/crates/gpui/Cargo.toml",
        ],
        &ctx,
    );
    assert!(
        inv.args.windows(2).any(|w| w
            == [
                "--config",
                "resolver.lockfile-path='/repo/artifacts-temp/locks/nsh/Cargo.lock'"
            ]),
        "{:?}",
        inv.args
    );
}
