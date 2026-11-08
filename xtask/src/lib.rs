//! Build tooling for dXn.GpuiTizen. Each subcommand's logic is a module of pure functions with its
//! own `<module>::tests` group; `main.rs` only parses arguments and runs processes.

use std::path::PathBuf;

pub mod cross;
pub mod gcc;
pub mod package;
pub mod policy;
pub mod symcheck;
pub mod targets;
pub mod tizen_deps;

/// The pinned toolchain channel from `rust-toolchain.toml`.
pub fn pinned_toolchain(root: &std::path::Path) -> anyhow::Result<String> {
    let text = std::fs::read_to_string(root.join("rust-toolchain.toml"))?;
    let value: toml::Value = text.parse()?;
    value
        .get("toolchain")
        .and_then(|t| t.get("channel"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("rust-toolchain.toml has no `toolchain.channel`"))
}

/// The workspace root (the parent of this crate's directory).
pub fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(PathBuf::from)
        .unwrap_or(manifest_dir)
}
