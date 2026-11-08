use std::process::Command;

use serde_json::Value;

use super::*;
use crate::workspace_root;

fn spec(target: Target) -> Value {
    let path = target.spec_path(&workspace_root());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?}: {e}"))
}

fn features(spec: &Value) -> Vec<String> {
    spec["features"]
        .as_str()
        .unwrap_or_default()
        .split(',')
        .map(str::to_owned)
        .collect()
}

fn pointer_width(spec: &Value) -> Option<u64> {
    let width = &spec["target-pointer-width"];
    width.as_u64().or_else(|| width.as_str()?.parse().ok())
}

#[test]
fn armv7_is_softfp_with_neon_and_tizen_vendor() {
    let spec = spec(Target::Armv7);
    assert_eq!(spec["arch"], "arm");
    assert_eq!(spec["llvm-target"], "armv7-unknown-linux-gnueabi");
    assert_eq!(spec["llvm-floatabi"], "soft", "the soft-float ABI");
    assert_eq!(spec["os"], "linux");
    assert_eq!(spec["env"], "gnu");
    assert_eq!(spec["vendor"], "tizen");
    assert_eq!(pointer_width(&spec), Some(32));
    let features = features(&spec);
    for required in ["+v7", "+thumb2", "+vfp3", "+neon"] {
        assert!(
            features.iter().any(|f| f == required),
            "missing {required}: {features:?}"
        );
    }
    for forbidden in ["+soft-float", "-neon", "-vfp3"] {
        assert!(
            !features.iter().any(|f| f == forbidden),
            "has {forbidden}: {features:?}"
        );
    }
}

#[test]
fn x86_64_is_the_emulator_with_tizen_vendor() {
    let spec = spec(Target::X86_64);
    assert_eq!(spec["arch"], "x86_64");
    assert_eq!(spec["llvm-target"], "x86_64-unknown-linux-gnu");
    assert_eq!(spec["os"], "linux");
    assert_eq!(spec["env"], "gnu");
    assert_eq!(spec["vendor"], "tizen");
    assert_eq!(pointer_width(&spec), Some(64));
}

#[test]
fn aarch64_is_stock_aarch64_with_tizen_vendor() {
    let spec = spec(Target::Aarch64);
    assert_eq!(spec["arch"], "aarch64");
    assert_eq!(spec["llvm-target"], "aarch64-unknown-linux-gnu");
    assert_eq!(spec["os"], "linux");
    assert_eq!(spec["env"], "gnu");
    assert_eq!(spec["vendor"], "tizen");
    assert_eq!(pointer_width(&spec), Some(64));
}

/// The pinned nightly (this test runs on it through `rust-toolchain.toml`) accepts every spec and
/// derives the cfgs the platform code depends on.
#[test]
fn pinned_nightly_accepts_every_spec() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    for &target in Target::ALL {
        let output = Command::new(&rustc)
            .current_dir(workspace_root())
            .args(["-Zunstable-options", "--print", "cfg", "--target"])
            .arg(target.spec_path(&workspace_root()))
            .output()
            .expect("rustc runs");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{target:?}: {stderr}");
        let cfg = String::from_utf8_lossy(&output.stdout);
        assert!(
            cfg.contains(r#"target_vendor="tizen""#),
            "{target:?}: {cfg}"
        );
        assert!(cfg.contains(r#"target_os="linux""#), "{target:?}: {cfg}");
        if target == Target::Armv7 {
            assert!(cfg.contains(r#"target_feature="neon""#), "{cfg}");
            assert!(cfg.contains(r#"target_feature="vfp3""#), "{cfg}");
        }
    }
}

#[test]
fn names_and_environment_keys() {
    assert_eq!(Target::parse("armv7").unwrap(), Target::Armv7);
    assert_eq!(Target::parse("x86_64").unwrap(), Target::X86_64);
    assert_eq!(Target::parse("aarch64").unwrap(), Target::Aarch64);
    assert!(Target::parse("mips").is_err());
    assert_eq!(Target::Aarch64.rootstrap_env(), "TIZEN_ROOTSTRAP_AARCH64");
    assert_eq!(Target::Aarch64.clang_triple(), "aarch64-tizen-linux-gnu");
    assert!(
        Target::Aarch64
            .sdk_rootstrap(Path::new("/sdk"))
            .ends_with("rootstraps/tizen-8.0-device64.core")
    );
    assert_eq!(Target::Armv7.env_suffix(), "armv7_tizen_linux_gnueabi");
    assert_eq!(
        Target::X86_64.cargo_env("LINKER"),
        "CARGO_TARGET_X86_64_TIZEN_LINUX_GNU_LINKER"
    );
    assert!(
        Target::Armv7
            .sdk_rootstrap(Path::new("/sdk"))
            .ends_with("platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-device.core")
    );
}
