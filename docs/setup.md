# Tizen SDK setup

What a machine needs before `cargo xtask cross` can link a TV library, and how to install it on a clean machine,
macOS or Linux, without any Tizen software. This guide is written so an agent can follow it with no prior context.
Every step is non-interactive.

## What is needed, and why

| Piece | Where it comes from | Used for |
| --- | --- | --- |
| Pinned nightly Rust | `rust-toolchain.toml` (rustup installs it on first `cargo` run) | `-Zbuild-std`, the custom target specs |
| `clang` ≥ 12 | Xcode Command Line Tools (macOS) or the distribution (Linux) | compiling C dependencies and driving the link (`--ld-path`) |
| Tizen SDK (Baseline SDK) | Samsung's CLI installer | the package manager that installs everything below |
| Tizen 8.0 rootstraps | SDK package `TIZEN-8.0-NativeAppDevelopment-CLI` | the sysroot: libc headers and the Tizen shared libraries |
| GCC 9.2 toolchain | SDK package `NativeToolchain-Gcc-9.2` (pulled in by the package above) | `crtbeginS.o`, `crtendS.o`, `libgcc_s` and GNU `ld` |

Which builds need what:

- **rlib builds and `--clippy`** (for example `cargo xtask cross --target armv7 -p dxn_gpui_tizen`) need only Rust and
  clang. Nothing from the Tizen SDK.
- **Linking a TV library** (any `cdylib`: `--all-tv-libs`, `-p dxn_gpui_tizen_diag`, the examples) needs the rootstrap
  *and* the GCC toolchain. `xtask` fails early, with install instructions, when either is missing.
- **Crates with C code built by `cc`** (for example `ring`) need the rootstrap's libc headers even without linking,
  because `xtask` compiles them with `--sysroot=<rootstrap>`.

Two facts about the Tizen 8.0 rootstraps shape `xtask` (`xtask/src/gcc/`):

1. They leave out GCC's start files and `libgcc_s`, so clang cannot link a shared library against them alone. These
   come from the SDK's GCC toolchain (`-B` / `-L` link arguments).
2. Their shared libraries (glibc, GLES, tbm, dlog, ... practically all ~600 per rootstrap) list imported symbols as
   LOCAL after `.dynsym`'s first global. GNU ld accepts this; **lld rejects it** (`invalid local symbol '' in global
   part of symbol table`). So linking uses the toolchain's GNU `ld.bfd` (`--ld-path`), not lld.

## Tizen Studio is gone

Tizen Studio (the Eclipse IDE) is deprecated and must not be installed or used; Samsung's docs say so explicitly
(<https://samsungtizenos.com/docs/sdk-tools/dotnet/vscode/baseline-sdk/setup/install-sdk>). Its replacement is the
**Tizen SDK** (the "Baseline SDK"), installed either by the VS Code / Visual Studio extension or by a CLI installer.
This repository uses the **CLI installer**: no IDE is needed. The installer still installs into `~/tizen-studio` by
default and still uses the `tizenstudio` package repository; those are names, not the old IDE.

## Install on a clean machine

### 1. Prerequisites

macOS (Apple Silicon or Intel):

```bash
xcode-select --install          # clang; skip if `clang --version` already works
```

```bash
softwareupdate --install-rosetta --agree-to-license   # Apple Silicon only: the SDK's binaries are x86_64
```

The SDK's tools, including `ld.bfd`, are x86_64 Mach-O binaries; on Apple Silicon they run under Rosetta 2. Check with
`pgrep oahd` (prints a PID when Rosetta is installed and running).

Linux (Ubuntu or Debian, x86_64):

```bash
sudo apt-get update && sudo apt-get install -y clang curl
```

Both: Rust through rustup (<https://rustup.rs>). The pinned nightly installs itself on the first `cargo` command in
this repository. The SDK bundles its own JDK, so no system Java is needed.

### 2. Download the CLI installer

The download page is <https://samsungtizenos.com/tools-download/> ("Tizen SDK", choose the OS). Direct links for
Tizen SDK 10.0 (verified 2026-11-08):

| OS | Installer |
| --- | --- |
| macOS | `https://usa.sdk-dl.tizen.org/web-cli_Tizen_SDK_10.0_usa_macos-64.bin` (≈311 MB) |
| Ubuntu | `https://usa.sdk-dl.tizen.org/web-cli_Tizen_SDK_10.0_usa_ubuntu-64.bin` |

Download into a fresh, empty directory outside the repository (an installer is untrusted input until it has run):

```bash
mkdir -p ~/Downloads/tizen-sdk && cd ~/Downloads/tizen-sdk
```

```bash
curl -fL -o tizen-sdk-cli.bin https://usa.sdk-dl.tizen.org/web-cli_Tizen_SDK_10.0_usa_macos-64.bin
```

(Use the `ubuntu-64` URL on Linux.) If the download page lists a newer SDK version, it should work too, but this guide
was verified with 10.0; confirm the package names with `show-pkgs` (step 5) before installing.

### 3. Run the installer

```bash
chmod +x tizen-sdk-cli.bin
```

```bash
yes | ./tizen-sdk-cli.bin --accept-license "$HOME/tizen-studio"
```

It ends with `Installation has been completed!` and creates `~/tizen-studio` (SDK), `~/tizen-studio-data` (data) and
`~/.package-manager` (package manager logs and its JDK). Options: `--accept-license`, `--no-java-check`, and the
install directory as the last argument (default `~/tizen-studio`). The installer only installs the web tools; the
native packages come next.

### 4. Fix the package repository URL (required)

The installer configures the package repository as **`http://`**`usa.sdk-dl.tizen.org/sdk/tizenstudio`, and that
mirror answers plain HTTP with `403`. Every package manager command that needs the network then fails with
`Cannot connect the repository server` or `Network disconnected`. Switch it to HTTPS:

```bash
cp ~/tizen-studio/.info/repository.info ~/tizen-studio/.info/repository.info.orig
```

```bash
sed -i.bak 's#^Repository: http://#Repository: https://#' ~/tizen-studio/.info/repository.info
```

(`sed -i.bak` works on both BSD and GNU sed.) The file should now start with
`Repository: https://usa.sdk-dl.tizen.org/sdk/tizenstudio`. Passing `-r <url>` to `install` does **not** help: the
package manager still reads `repository.info` first.

### 5. Install the Tizen 8.0 native packages

```bash
~/tizen-studio/package-manager/package-manager-cli.bin install --accept-license TIZEN-8.0-NativeAppDevelopment-CLI
```

This installs `NativeToolchain-Gcc-9.2`, `NativeCLI` and `TIZEN-8.0-NativeAppDevelopment-CLI`, and updates `WebCLI`.
It takes a few minutes. On Linux, packages that need root take `-p <sudo password>`; prefer running the command in
a shell where `sudo` is already authorized instead of putting a password on the command line.

To see what is available (after step 4): `package-manager-cli.bin show-pkgs --tree`, or, without relying on
`repository.info`:

```bash
~/tizen-studio/package-manager/package-manager-cli.bin show-repo-info -r https://download.tizen.org/sdk/tizenstudio -d official -s Tizen_SDK_10.0 --tree
```

### 6. Verify

```bash
ls ~/tizen-studio/platforms/tizen-8.0/tizen/rootstraps/
```

expects `tizen-8.0-device.core` (armv7), `tizen-8.0-device64.core` (aarch64), `tizen-8.0-emulator64.core` (x86_64)
and `tizen-8.0-emulator.core` (unused).

```bash
ls ~/tizen-studio/tools/ | grep gcc-9.2
```

expects `arm-linux-gnueabi-gcc-9.2`, `aarch64-linux-gnu-gcc-9.2`, `x86_64-linux-gnu-gcc-9.2` (and `i586-...`).

Then, from this repository:

```bash
cargo xtask cross --target armv7 --all-tv-libs
```

which must finish and produce `target/armv7-tizen-linux-gnueabi/debug/lib{dxn_gpui_tizen_diag,example_counter,example_long_list,example_video}.so`
(`file` reports `ELF 32-bit LSB shared object, ARM, EABI5`). Repeat with `--target aarch64` and `--target x86_64`.

Disk use after setup: about 3.5 GB in `~/tizen-studio` and 0.3 GB in `~/.package-manager`.

## How `xtask` finds the SDK

All lookups are per target; nothing needs configuring when the SDK is in `~/tizen-studio`.

| What | Environment variable (wins) | Default |
| --- | --- | --- |
| SDK root | `TIZEN_SDK` | `~/tizen-studio` |
| armv7 rootstrap | `TIZEN_ROOTSTRAP_ARMV7` | `<SDK>/platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-device.core` |
| x86_64 rootstrap | `TIZEN_ROOTSTRAP_X86_64` | `<SDK>/platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-emulator64.core` |
| aarch64 rootstrap | `TIZEN_ROOTSTRAP_AARCH64` | `<SDK>/platforms/tizen-8.0/tizen/rootstraps/tizen-8.0-device64.core` |
| armv7 GCC toolchain | `TIZEN_GCC_TOOLCHAIN_ARMV7` | `<SDK>/tools/arm-linux-gnueabi-gcc-9.2` |
| x86_64 GCC toolchain | `TIZEN_GCC_TOOLCHAIN_X86_64` | `<SDK>/tools/x86_64-linux-gnu-gcc-9.2` |
| aarch64 GCC toolchain | `TIZEN_GCC_TOOLCHAIN_AARCH64` | `<SDK>/tools/aarch64-linux-gnu-gcc-9.2` |
| C compiler | `TIZEN_CLANG` | `clang` |

From the toolchain, `xtask` passes the linker `--ld-path=<toolchain>/<gcc triple>/bin/ld.bfd`,
`-B`/`-L<toolchain>/lib/gcc/<gcc triple>/9.2.0` (start files, `libgcc`) and `-L<toolchain>/<gcc triple>/lib` (armv7)
or `lib64` (x86_64, aarch64) for `libgcc_s`. The GCC triples are `arm-tizen-linux-gnueabi`, `x86_64-tizen-linux-gnu`
and `aarch64-tizen-linux-gnu`. C code is still compiled by clang, never by the SDK's GCC.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `show-pkgs` prints `Cannot connect the repository server`; `install` prints `Network disconnected` | `repository.info` uses `http://`, which the mirror rejects (403); the log in `~/.package-manager/install-log/package-manager/` shows `Cannot connect the download URL` | step 4 |
| `install` prints `Failed to install the following packages automatically` with `libpython3.8`, `python3.8` (Ubuntu 24.04) | the package manager requires every OS prerequisite to show as installed in `dpkg -l`, and Ubuntu 24.04 has no Python 3.8; nothing this repository builds runs it | install empty stand-in packages of those names, as `.github/actions/tizen-sdk` does, then run step 5 again |
| `linking for ... needs the Tizen 8.0 common rootstrap ...` | rootstrap not installed or not found | step 5, or set `TIZEN_ROOTSTRAP_<ARCH>` |
| `linking for ... needs the Tizen SDK's GCC 9.2.0 toolchain ...` | `NativeToolchain-Gcc-9.2` missing | `package-manager-cli.bin install --accept-license NativeToolchain-Gcc-9.2`, or set `TIZEN_GCC_TOOLCHAIN_<ARCH>` |
| `cannot open crtbeginS.o`, `unable to find library -lgcc_s` | linking without the GCC toolchain arguments (an old `xtask`, or a hand-written link command) | use `cargo xtask cross`; it adds them |
| `invalid local symbol '' in global part of symbol table` from `rust-lld` | linking with lld | use `cargo xtask cross`; it links with the SDK's `ld.bfd` |
| `ld.bfd`: `Bad CPU type in executable` (macOS) | Rosetta 2 missing on Apple Silicon | `softwareupdate --install-rosetta --agree-to-license` |
| `fatal error: 'string.h' file not found` in a `*-sys` crate that builds C++ (`freetype-sys`) | `xtask` configures only the C compiler; C++ in a TV library is not allowed anyway (`AGENTS.md`) | find out why the crate is in the Tizen build (usually a desktop platform crate pulled in by a default feature) and remove it there |
| `pkg-config has not been configured to support cross-compilation` (`yeslogic-fontconfig-sys`, ...) | a desktop-only system library crate in the Tizen build | same as above: such crates must not reach a Tizen build |
| `unrecognised emulation mode: llvm` from `ld.bfd` | the GNU linker was injected through `RUSTFLAGS` and also reached host build scripts | do not pass link flags through `RUSTFLAGS`; `xtask` scopes them to the target |

Known gap: the armv7 rootstrap has no `libwayland-client` (headers or library). A crate that links the system
libwayland (for example `wayland-backend` without `dlopen`) cannot link against it.

## Rules

- Never commit the SDK, rootstraps, toolchains, Samsung certificates, `profiles.xml` or `.p12` files (`.gitignore`
  excludes them and a local `/rootstraps`). Certificates, the TV's developer mode and signing are in
  [`development_setup.md`](development_setup.md).
- Never install or use Tizen Studio.
- Keep raw build output under `artifacts-temp/` (ignored), for example
  `cargo xtask cross ... > artifacts-temp/cross-armv7.log 2>&1`.
- Do not run `cargo tree`, `cargo update` or plain `cargo build` inside the NSH checkout to diagnose a Tizen build:
  they rewrite NSH's `Cargo.lock`. `cargo xtask cross --manifest-path <nsh>/...` keeps a copy of a foreign lockfile
  under `artifacts-temp/locks/` instead. If NSH's lockfile changed anyway, restore it with
  `git -C <nsh checkout> checkout -- crates/gpui/Cargo.lock`.

## Uninstall

```bash
~/tizen-studio/package-manager/package-manager-uninstaller.bin
```

or remove `~/tizen-studio`, `~/tizen-studio-data` and `~/.package-manager`.
