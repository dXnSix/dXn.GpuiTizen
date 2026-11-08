use std::path::{Path, PathBuf};

use super::*;
use crate::cross::{Context, Options, plan};

#[test]
fn discovery_prefers_the_environment_then_the_sdk() {
    assert_eq!(
        candidates(
            Target::Armv7,
            Some("/custom/gcc"),
            Some("/sdk"),
            Some(Path::new("/home/me"))
        ),
        [PathBuf::from("/custom/gcc")]
    );
    assert_eq!(
        candidates(
            Target::Aarch64,
            None,
            Some("/sdk"),
            Some(Path::new("/home/me"))
        ),
        [PathBuf::from("/sdk/tools/aarch64-linux-gnu-gcc-9.2")]
    );
    assert_eq!(
        candidates(Target::X86_64, Some(""), None, Some(Path::new("/home/me"))),
        [PathBuf::from(
            "/home/me/tizen-studio/tools/x86_64-linux-gnu-gcc-9.2"
        )]
    );
    assert!(candidates(Target::Armv7, None, None, None).is_empty());
    assert_eq!(env(Target::Armv7), "TIZEN_GCC_TOOLCHAIN_ARMV7");
}

#[test]
fn link_args_follow_the_sdk_layout() {
    assert_eq!(
        link_args(Target::Armv7, Path::new("/t")),
        [
            "--ld-path=/t/arm-tizen-linux-gnueabi/bin/ld.bfd",
            "-B/t/lib/gcc/arm-tizen-linux-gnueabi/9.2.0",
            "-L/t/lib/gcc/arm-tizen-linux-gnueabi/9.2.0",
            "-L/t/arm-tizen-linux-gnueabi/lib",
        ]
    );
    assert_eq!(
        link_args(Target::Aarch64, Path::new("/t")),
        [
            "--ld-path=/t/aarch64-tizen-linux-gnu/bin/ld.bfd",
            "-B/t/lib/gcc/aarch64-tizen-linux-gnu/9.2.0",
            "-L/t/lib/gcc/aarch64-tizen-linux-gnu/9.2.0",
            "-L/t/aarch64-tizen-linux-gnu/lib64",
        ]
    );
    assert_eq!(
        link_args(Target::X86_64, Path::new("/t"))[3],
        "-L/t/x86_64-tizen-linux-gnu/lib64"
    );
}

fn ctx(gcc_toolchain: Option<&str>, needs_link: bool) -> Context {
    Context {
        root: PathBuf::from("/repo"),
        toolchain: "nightly".into(),
        rustlib_bin: PathBuf::from("/tc/bin"),
        clang: "clang".into(),
        rootstrap: Some(PathBuf::from("/rs")),
        rootstrap_searched: Vec::new(),
        gcc_toolchain: gcc_toolchain.map(PathBuf::from),
        gcc_toolchain_searched: vec![PathBuf::from(
            "/home/me/tizen-studio/tools/arm-linux-gnueabi-gcc-9.2",
        )],
        needs_link,
        inherited_rustflags: Vec::new(),
        lockfile: None,
    }
}

fn options(list: &[&str]) -> Options {
    Options::parse(&list.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()).unwrap()
}

#[test]
fn linking_builds_pass_the_toolchain_to_the_linker() {
    let inv = plan(
        &options(&["--target", "armv7", "-p", "example_counter"]),
        &ctx(Some("/gcc"), true),
        &[],
    )
    .unwrap();
    let flags = inv.env_value("CARGO_ENCODED_RUSTFLAGS").unwrap();
    let flags: Vec<&str> = flags.split('\x1f').collect();
    assert!(
        !flags.contains(&"-Clink-arg=-fuse-ld=lld"),
        "lld rejects the rootstrap's libraries: {flags:?}"
    );
    for expected in [
        "-Clink-arg=--ld-path=/gcc/arm-tizen-linux-gnueabi/bin/ld.bfd",
        "-Clink-arg=-B/gcc/lib/gcc/arm-tizen-linux-gnueabi/9.2.0",
        "-Clink-arg=-L/gcc/arm-tizen-linux-gnueabi/lib",
    ] {
        assert!(
            flags.contains(&expected),
            "{expected} missing from {flags:?}"
        );
    }
    let cflags = inv.env_value("CFLAGS_armv7_tizen_linux_gnueabi").unwrap();
    assert!(
        !cflags.contains("/gcc"),
        "C compilation stays on clang: {cflags}"
    );
}

#[test]
fn missing_toolchain_fails_only_linking_builds() {
    let err = plan(
        &options(&["--target", "armv7", "-p", "example_counter"]),
        &ctx(None, true),
        &[],
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("arm-linux-gnueabi-gcc-9.2"), "{err}");
    assert!(err.contains("TIZEN_GCC_TOOLCHAIN_ARMV7"), "{err}");
    assert!(err.contains("NativeToolchain-Gcc-9.2"), "{err}");
    assert!(err.contains("/home/me/tizen-studio/tools"), "{err}");

    assert!(
        plan(
            &options(&["--target", "armv7", "-p", "dxn_gpui_tizen"]),
            &ctx(None, false),
            &[]
        )
        .is_ok()
    );
    assert!(
        plan(
            &options(&["--target", "armv7", "--clippy", "-p", "example_counter"]),
            &ctx(None, true),
            &[]
        )
        .is_ok()
    );
}

/// Every target: the full link arguments, the environment variable, the SDK candidate and the
/// missing-toolchain error.
#[test]
fn every_target_has_link_args_lookup_and_failure() {
    let table = [
        (
            Target::Armv7,
            "arm-linux-gnueabi-gcc-9.2",
            "arm-tizen-linux-gnueabi",
            "lib",
            "TIZEN_GCC_TOOLCHAIN_ARMV7",
        ),
        (
            Target::X86_64,
            "x86_64-linux-gnu-gcc-9.2",
            "x86_64-tizen-linux-gnu",
            "lib64",
            "TIZEN_GCC_TOOLCHAIN_X86_64",
        ),
        (
            Target::Aarch64,
            "aarch64-linux-gnu-gcc-9.2",
            "aarch64-tizen-linux-gnu",
            "lib64",
            "TIZEN_GCC_TOOLCHAIN_AARCH64",
        ),
    ];
    assert_eq!(table.len(), Target::ALL.len());
    for (target, dir, triple, runtime, variable) in table {
        assert_eq!(
            link_args(target, Path::new("/t")),
            [
                format!("--ld-path=/t/{triple}/bin/ld.bfd"),
                format!("-B/t/lib/gcc/{triple}/9.2.0"),
                format!("-L/t/lib/gcc/{triple}/9.2.0"),
                format!("-L/t/{triple}/{runtime}"),
            ],
            "{target:?}"
        );
        assert_eq!(env(target), variable);
        assert_eq!(
            candidates(target, None, Some("/sdk"), None),
            [PathBuf::from(format!("/sdk/tools/{dir}"))]
        );
        let searched = candidates(target, None, None, Some(Path::new("/home/me")));
        let err = missing(target, &searched);
        for needle in [
            dir,
            variable,
            "NativeToolchain-Gcc-9.2",
            "/home/me/tizen-studio/tools",
        ] {
            assert!(
                err.contains(needle),
                "{target:?}: {needle} missing from {err}"
            );
        }
    }
}

/// Without an SDK toolchain, a non-linking build keeps the pinned toolchain's lld.
#[test]
fn non_linking_builds_without_the_toolchain_use_lld() {
    let inv = plan(
        &options(&["--target", "armv7", "-p", "dxn_gpui_tizen"]),
        &ctx(None, false),
        &[],
    )
    .unwrap();
    let flags = inv.env_value("CARGO_ENCODED_RUSTFLAGS").unwrap();
    let flags: Vec<&str> = flags.split('\x1f').collect();
    assert!(flags.contains(&"-Clink-arg=-fuse-ld=lld"), "{flags:?}");
    assert!(flags.contains(&"-Clink-arg=-B/tc/bin/gcc-ld"), "{flags:?}");
    assert!(!flags.iter().any(|f| f.contains("--ld-path")), "{flags:?}");

    // With the toolchain installed, a non-linking build uses the same flags as a linking one.
    let gate = plan(
        &options(&["--target", "armv7", "-p", "dxn_gpui_tizen"]),
        &ctx(Some("/gcc"), false),
        &[],
    )
    .unwrap();
    let link = plan(
        &options(&["--target", "armv7", "-p", "dxn_gpui_tizen"]),
        &ctx(Some("/gcc"), true),
        &[],
    )
    .unwrap();
    assert_eq!(
        gate.env_value("CARGO_ENCODED_RUSTFLAGS"),
        link.env_value("CARGO_ENCODED_RUSTFLAGS")
    );
}
