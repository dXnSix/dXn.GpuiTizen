//! The Tizen targets: their custom spec files, the clang triple and flags for C code, and where
//! their rootstrap (the sysroot) is found.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

/// A Tizen target this repository builds for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// 32-bit ARM TVs: the soft-float ABI with VFPv3 and NEON hardware ("softfp").
    Armv7,
    /// The Tizen emulator.
    X86_64,
    /// 64-bit ARM devices (no retail TV is confirmed 64-bit yet).
    Aarch64,
}

impl Target {
    pub const ALL: &[Target] = &[Target::Armv7, Target::X86_64, Target::Aarch64];

    /// Parses the `--target` value (`armv7`, `x86_64`, or a full spec name).
    pub fn parse(name: &str) -> Result<Self> {
        Ok(match name {
            "armv7" | "arm" | "armv7-tizen-linux-gnueabi" => Target::Armv7,
            "x86_64" | "x86-64" | "emulator" | "x86_64-tizen-linux-gnu" => Target::X86_64,
            "aarch64" | "arm64" | "aarch64-tizen-linux-gnu" => Target::Aarch64,
            other => bail!("unknown target `{other}`; expected one of armv7, x86_64, aarch64"),
        })
    }

    /// The spec's name: `targets/<name>.json`, and the target directory under `target/`.
    pub fn spec_name(self) -> &'static str {
        match self {
            Target::Armv7 => "armv7-tizen-linux-gnueabi",
            Target::X86_64 => "x86_64-tizen-linux-gnu",
            Target::Aarch64 => "aarch64-tizen-linux-gnu",
        }
    }

    /// The spec file, relative to the workspace root.
    pub fn spec_path(self, root: &Path) -> PathBuf {
        root.join("targets")
            .join(format!("{}.json", self.spec_name()))
    }

    /// The triple clang compiles and links C code for; it matches the rootstrap's GCC directory.
    pub fn clang_triple(self) -> &'static str {
        match self {
            Target::Armv7 => "armv7l-tizen-linux-gnueabi",
            Target::X86_64 => "x86_64-tizen-linux-gnu",
            Target::Aarch64 => "aarch64-tizen-linux-gnu",
        }
    }

    /// Extra C flags (flutter-tizen's for armv7).
    pub fn c_flags(self) -> &'static [&'static str] {
        match self {
            Target::Armv7 => &["-march=armv7-a", "-mfloat-abi=softfp", "-mfpu=neon"],
            Target::X86_64 | Target::Aarch64 => &[],
        }
    }

    /// The environment variable that names this target's rootstrap.
    pub fn rootstrap_env(self) -> &'static str {
        match self {
            Target::Armv7 => "TIZEN_ROOTSTRAP_ARMV7",
            Target::X86_64 => "TIZEN_ROOTSTRAP_X86_64",
            Target::Aarch64 => "TIZEN_ROOTSTRAP_AARCH64",
        }
    }

    /// The rootstrap's directory name in the Tizen SDK (Tizen 8.0, common profile).
    pub fn sdk_rootstrap_name(self) -> &'static str {
        match self {
            Target::Armv7 => "tizen-8.0-device.core",
            Target::X86_64 => "tizen-8.0-emulator64.core",
            Target::Aarch64 => "tizen-8.0-device64.core",
        }
    }

    /// The rootstrap's default location inside a Tizen SDK installation.
    pub fn sdk_rootstrap(self, sdk: &Path) -> PathBuf {
        sdk.join("platforms/tizen-8.0/tizen/rootstraps")
            .join(self.sdk_rootstrap_name())
    }

    /// The suffix Cargo and `cc` use in per-target environment variables (`CC_<suffix>`).
    pub fn env_suffix(self) -> String {
        self.spec_name().replace('-', "_")
    }

    /// `CARGO_TARGET_<SUFFIX>_...`.
    pub fn cargo_env(self, key: &str) -> String {
        format!("CARGO_TARGET_{}_{key}", self.env_suffix().to_uppercase())
    }
}

/// The Tizen SDK installation: `TIZEN_SDK`, else `~/tizen-studio` (the SDK installer's default).
pub fn sdk_root(tizen_sdk: Option<&str>, home: Option<&Path>) -> Option<PathBuf> {
    match (tizen_sdk.filter(|v| !v.is_empty()), home) {
        (Some(sdk), _) => Some(PathBuf::from(sdk)),
        (None, Some(home)) => Some(home.join("tizen-studio")),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests;
