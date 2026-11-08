//! The Tizen SDK's GCC toolchain, which supplies what the Tizen 8.0 rootstraps leave out for
//! linking a shared library: the C runtime start files (`crtbeginS.o`, `crtendS.o`) and `libgcc_s`.
//! clang drives the toolchain's GNU `ld.bfd` (`--ld-path`) and finds them through the `-B` / `-L`
//! arguments built here; nothing else is used from the toolchain (C code is still compiled with
//! clang, never by the SDK's GCC).

use std::path::{Path, PathBuf};

use crate::targets::Target;

/// The GCC release the Tizen SDK's `NativeToolchain-Gcc-9.2` package installs.
pub const GCC_VERSION: &str = "9.2.0";

/// The environment variable that names this target's GCC toolchain directory.
pub fn env(target: Target) -> &'static str {
    match target {
        Target::Armv7 => "TIZEN_GCC_TOOLCHAIN_ARMV7",
        Target::X86_64 => "TIZEN_GCC_TOOLCHAIN_X86_64",
        Target::Aarch64 => "TIZEN_GCC_TOOLCHAIN_AARCH64",
    }
}

/// The toolchain's directory name under the Tizen SDK's `tools/`.
pub fn sdk_dir_name(target: Target) -> &'static str {
    match target {
        Target::Armv7 => "arm-linux-gnueabi-gcc-9.2",
        Target::X86_64 => "x86_64-linux-gnu-gcc-9.2",
        Target::Aarch64 => "aarch64-linux-gnu-gcc-9.2",
    }
}

/// GCC's own triple, which names its directories inside the toolchain.
pub fn triple(target: Target) -> &'static str {
    match target {
        Target::Armv7 => "arm-tizen-linux-gnueabi",
        Target::X86_64 => "x86_64-tizen-linux-gnu",
        Target::Aarch64 => "aarch64-tizen-linux-gnu",
    }
}

/// The directory under `<triple>/` that holds `libgcc_s.so`.
fn runtime_lib_dir(target: Target) -> &'static str {
    match target {
        Target::Armv7 => "lib",
        Target::X86_64 | Target::Aarch64 => "lib64",
    }
}

/// The toolchain candidates for a target, in order: the environment variable, then the Tizen
/// SDK's `tools/` directory (`TIZEN_SDK`, else `~/tizen-studio`).
pub fn candidates(
    target: Target,
    env_value: Option<&str>,
    tizen_sdk: Option<&str>,
    home: Option<&Path>,
) -> Vec<PathBuf> {
    if let Some(value) = env_value.filter(|v| !v.is_empty()) {
        return vec![PathBuf::from(value)];
    }
    crate::targets::sdk_root(tizen_sdk, home)
        .map(|sdk| vec![sdk.join("tools").join(sdk_dir_name(target))])
        .unwrap_or_default()
}

/// The toolchain's GNU linker. Linking against a Tizen 8.0 rootstrap needs it rather than lld:
/// the rootstraps' shared libraries list imported symbols as LOCAL after `.dynsym`'s first global
/// (`sh_info`), which GNU ld accepts and lld rejects ("invalid local symbol in global part of
/// symbol table").
pub fn linker(target: Target, toolchain: &Path) -> PathBuf {
    toolchain.join(triple(target)).join("bin/ld.bfd")
}

/// The link arguments for clang: the toolchain's GNU linker, start files and `libgcc_s`.
pub fn link_args(target: Target, toolchain: &Path) -> Vec<String> {
    let gcc_lib = toolchain
        .join("lib/gcc")
        .join(triple(target))
        .join(GCC_VERSION);
    let runtime = toolchain.join(triple(target)).join(runtime_lib_dir(target));
    vec![
        format!("--ld-path={}", linker(target, toolchain).display()),
        format!("-B{}", gcc_lib.display()),
        format!("-L{}", gcc_lib.display()),
        format!("-L{}", runtime.display()),
    ]
}

/// The error for a linking build without the toolchain.
pub fn missing(target: Target, searched: &[PathBuf]) -> String {
    let searched: Vec<String> = searched.iter().map(|p| p.display().to_string()).collect();
    format!(
        "linking for {} needs the Tizen SDK's GCC {GCC_VERSION} toolchain `{}` (for crtbeginS.o and \
         libgcc_s, which the rootstrap leaves out), which was not found (looked in: {}).\n\
         Install it with the Tizen SDK's package manager (`package-manager-cli install \
         NativeToolchain-Gcc-9.2`), or point `{}` at the toolchain directory.",
        target.spec_name(),
        sdk_dir_name(target),
        if searched.is_empty() {
            "nowhere".to_owned()
        } else {
            searched.join(", ")
        },
        env(target),
    )
}

#[cfg(test)]
mod tests;
