# dXn.GpuiTizen

The Tizen platform for [GPUI](https://github.com/longbridge/gpui-fast) (gpui-fast), so GPUI apps run on Samsung TVs.
gpui-fast renders through wgpu, and wgpu speaks GL ES to the TV's GPU; this repository adds what Tizen needs around
that: target specs and cross builds, bindings to the documented Tizen C APIs, the platform (one full-screen window on
Ecore_Wl2, a dispatcher on Ecore's main loop, TV keys, the on-screen keyboard, the lifecycle, one display), a
diagnostic app, examples, and the small C# host that Tizen's app model requires.

| Crate | What it is |
|---|---|
| `dxn_gpui_tizen_sys` | hand-written `extern "C"` bindings to the Tizen 8.0 native APIs used |
| `dxn_gpui_tizen` | the Tizen platform for GPUI and its `run` entry point |
| `dxn_gpui_tizen_diag` | the diagnostic app: EGL / GL ES details, CPU, display, launch timings |
| `example_counter`, `example_long_list` | plain GPUI apps for the TV and the desktop |
| `example_video` | a video player for the TV (a placeholder on the desktop) |

## The repositories

- **dXn.GpuiTizen** (this one): everything GPUI and wgpu on Tizen, nothing of NSH.
- **[dXnSix/gpui-fast](https://github.com/dXnSix/gpui-fast)**: renderer changes (GL ES 3.0 devices, GL on request, the
  pipeline cache, partial redraws), one branch per change, merged on `tizen` and released as `tizen-release`.
- **[dXnSix/wgpu](https://github.com/dXnSix/wgpu)**: EGL 1.4 fixes and GL program binaries on `v30-tizen`.
- **[dXnSix/nsh](https://github.com/dXnSix/nsh)**: the NSH part (`nsh-platform-tizen`, the `tizen` and `tizen-web`
  targets), which consumes this repository.

## Building

The toolchain is the dated nightly in `rust-toolchain.toml` (cross builds need `-Zbuild-std`). Host builds and tests
are ordinary Cargo commands; cross builds go through `xtask`:

```bash
cargo xtask cross --target armv7 -p dxn_gpui_tizen            # 32-bit ARM TVs (softfp, NEON)
cargo xtask cross --target x86_64 --release --all-tv-libs     # the Tizen emulator
cargo xtask cross --target aarch64 -p dxn_gpui_tizen          # 64-bit ARM devices
```

Linking a TV library (a `cdylib`) needs the public Tizen 8.0 common rootstrap, installed with the Tizen SDK and found
through `TIZEN_ROOTSTRAP_ARMV7` / `TIZEN_ROOTSTRAP_X86_64` / `TIZEN_ROOTSTRAP_AARCH64` or the SDK's default
location; C code is compiled with clang against it. Linking also needs the SDK's GCC 9.2 toolchain (`NativeToolchain-Gcc-9.2`,
found under the SDK's `tools/` or through `TIZEN_GCC_TOOLCHAIN_ARMV7` / `_X86_64` / `_AARCH64`): it supplies the
start files and `libgcc_s` the rootstrap leaves out, and its GNU `ld`, since lld rejects the rootstrap's libraries.
[docs/setup.md](docs/setup.md) installs all of it on a clean macOS or Linux machine;
[docs/development_setup.md](docs/development_setup.md) covers the certificates, the TV's developer mode and the
repository's secrets. Rootstraps and Samsung certificates are never committed.

TV builds take gpui-fast and wgpu from the forks: `Cargo.toml`'s `[patch.crates-io]` takes all nine `gpui-fast-*`
crates from `dXnSix/gpui-fast` at `tizen-release` and the wgpu stack from `dXnSix/wgpu` at `v30-tizen` (the lockfile
pins the commits). While changing the forks themselves, run `cargo xtask gpui-fast-release --verify` with
`../gpui-fast` on `tizen` and `../wgpu` on `v30-tizen`: it writes `artifacts-temp/gpui-fast-patch.toml`, which
overrides the git patch with those checkouts in every `cargo xtask cross` (and `scripts/lima-run.sh`) build until you
delete it. `cargo xtask gpui-fast-release` without `--verify` commits a new `tizen-release` from `tizen`; push it and
run `cargo update -p gpui-fast` to move the lockfile to it.

## Rendering

By default the platform owns EGL (`RunOptions::renderer = Renderer::Auto`): it creates the GL ES 3 context on the
window's Wayland surface, registers the driver's shader cache (`EGL_ANDROID_blob_cache`, under the app's cache
directory) and presents partial frames with buffer age and damage, drawing through gpui_wgpu's external-target
renderer. When that context does not come up it falls back to wgpu's own surface and logs why once.
`DXN_GPUI_TIZEN_RENDERER=egl|wgpu|auto` overrides the choice.

## Video

`dxn_gpui_tizen::video::VideoPlayer` plays media with Tizen's public `capi-media-player` and paints it with
`VideoPlayer::element`. By default the video plays on the TV's video plane under the window: the element paints a hole
(gpui-fast's `Window::paint_hole`) and moves the player's exported surface there, and the window turns translucent
while its frame has holes (on the EGL path with `EGL_KHR_no_config_context`). Where the window cannot turn
translucent, or when asked (`Output::Texture`), decoded frames are converted to RGBA and drawn as a dynamic texture,
which also works inside transformed or clipped UI. `examples/video` plays a bundled clip with a control row over it.

## Lifecycle, keys and system information

Pausing the app (the TV switches to another app or input) stops frames, frees the surface and reports the window to
GPUI as inactive (`on_active_status_change(false)`) and hidden (`WindowVisibility::Hidden`); resuming rebuilds the
surface, reports it visible and active, then redraws everything. `is_active()` and `visibility()` follow.

While the window is shown the color, media and channel keys are grabbed so they reach the app instead of the TV
(`RunOptions::grab_keys` chooses the set at start). `dxn_gpui_tizen::keygrab::{grab, release, set}` change the set
while the app runs, by Tizen (`XF86Info`) or GPUI (`info`) key name, on the main thread; the set survives pause and
resume. The remote's Info key is grabbed only on request; Return, Exit and Home never are.

`dxn_gpui_tizen::system_info` reads `system_info`'s documented platform keys (`platform_string`, `platform_int`,
`platform_bool`), and `product_info()` the model name, manufacturer, platform name and version, build and screen size
an about screen shows; every read is `None` off Tizen.

## Packaging and store submission

`cargo xtask package --app <app> --target <armv7|x86_64|aarch64>` stages a TPK tree under `target/tpk/`;
`--target all` stages one package per TV architecture (armv7 and aarch64, each with its own fourth version number
and CPU feature) and the multi-architecture `.zip` for Seller Office (see the assumptions below). Packages are signed only with
`--sign <profile>`, a certificate profile of `tz security-profiles` (a Samsung certificate, for a TV and for the
emulator; `docs/development_setup.md`): the staged tree is zipped and re-signed with `tz pack -b`,
because `tz pack` does not sign a plain directory. The signed package is `target/tpk/<package id>-<version>-<arch>.tpk`.

Before submitting, `cargo xtask store-check <tpk or staged dir>` checks what can be checked on the Mac: the manifest
(package id, version, API version 8.0, the `tv-samsung` profile, the app id), the icon (a 512×423 PNG), the
privileges against those the feature description declares, one export and only allowlisted imports in each native
library, the sizes, and the feature description file itself. The templates are in `store/feature-description.<app>.md`;
complete them for the app being submitted.

NSH apps package with `nsh package --target tizen`; check the TPK it writes with the NSH entry point and the app's own
feature description:

```bash
cargo xtask store-check path/to/App.tpk --entry nsh_tizen_main --feature-description store/feature-description.my_app.md
```

`store-check` also takes the multi-architecture `.zip`: it checks the upload as a whole (one package id and
version, a fourth version number unique to each package, each package's CPU feature against its library's
architecture) and then each package.

The multi-architecture layout follows Seller Office's launch checklist, which leaves details open. These are
assumptions until the first submission confirms them (the owner's store sprint): the CPU feature keys
(`http://tizen.org/feature/platform.core.cpu.arch.armv7` / `armv8`), that a four-part version is accepted in
`tizen-manifest.xml`, and that the `.zip` holds the packages at its top level.

Submission itself (Seller Office, the certificates, the answer on native libraries for public sellers) is the
owner's.

## Continuous integration

`.github/workflows/ci.yml` starts with a `plan` job (`.github/actions/plan/`, tested by `bash
.github/actions/plan/plan_test.sh`) that chooses what runs:

| Run | What it runs |
| --- | --- |
| manual (`workflow_dispatch`) | the `jobs`, `platforms` (`ubuntu`, `macos`) and `grep` inputs; `all` is everything |
| pull request | labels: `ci:<job>` limits the jobs (several add up), `ci:all`, `ci-os:<ubuntu\|macos>`; without a `ci:` label, a change that touches only documentation runs only `check` |
| push to `main`, weekly schedule | everything |

The jobs: `check` (`cargo xtask policy`, `cargo fmt --check`, clippy with `-D warnings` on Linux and macOS, and the
workflow files), `test` (the host test groups, `cargo test --workspace --lib`), `host` (`dotnet build` of the C# host
with `Tizen.NET.Sdk` from NuGet), `cross` (armv7 and x86_64: clippy for the target, the release TV libraries with the
symbol check, and the Tizen dependency graph, with the Tizen SDK and the 8.0 rootstraps installed in the job), `gles`
(both examples' `gles_smoke` tests under Xvfb with Mesa's llvmpipe on the GL ES 3.0 path) and `egl` (the
platform-owned EGL path, `egl_wayland`, under headless Weston). `grep` filters the test names of `test`, `gles` and
`egl`.

The jobs build with `Cargo.toml`'s git patch, so both forks must be readable by the workflow. Without the committed
patch (after gpui-fast and wgpu release the changes, or on a branch that drops it), each job that builds checks out
`dXnSix/gpui-fast` at `tizen-release` and `dXnSix/wgpu` at `v30-tizen` under `artifacts-temp/forks/` and writes the
patch config with `cargo xtask gpui-fast-release --from artifacts-temp/forks/gpui-fast --wgpu
artifacts-temp/forks/wgpu` instead.

`.github/workflows/tizen-emulator.yml` (manual and weekly) boots the Samsung TV emulator headless on an Ubuntu runner
with KVM, packages and signs the x86_64 diagnostic TPK, installs and launches it, pulls its report with `cargo xtask
diag pull` and uploads the report and the emulator's logs. Installing needs a Samsung certificate whose distributor
lists the emulator's DUID, because the Samsung TV emulator, like a TV, trusts only Samsung's distributor roots. It
comes from the secrets `TIZEN_AUTHOR_P12`, `TIZEN_AUTHOR_PASSWORD`, `TIZEN_DISTRIBUTOR_P12` and
`TIZEN_DISTRIBUTOR_PASSWORD` ([docs/development_setup.md](docs/development_setup.md), section 5). Without them,
the job signs with a throwaway Tizen certificate, boots the emulator and skips the install.
[docs/ci-emulator.md](docs/ci-emulator.md) describes what the job needs and what each step works around.

Both workflows install the Tizen SDK CLI with `--accept-license`: running them is the owner's acceptance of Samsung's
SDK licence on the runner.

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
