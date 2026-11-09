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
[docs/setup.md](docs/setup.md) installs all of it on a clean macOS or Linux machine. Rootstraps and Samsung
certificates are never committed.

TV builds take gpui-fast and wgpu from the forks (`../gpui-fast` on `tizen`, `../wgpu` on `v30-tizen`): run
`cargo xtask gpui-fast-release --verify` once, which writes `artifacts-temp/gpui-fast-patch.toml`; `cargo xtask cross`
then applies it to every build of this workspace.

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

## Packaging and store submission

`cargo xtask package --app <app> --target <armv7|x86_64|aarch64>` stages a TPK tree under `target/tpk/`;
`--target all` stages one package per TV architecture (armv7 and aarch64, each with its own fourth version number
and CPU feature) and the multi-architecture `.zip` for Seller Office (see the assumptions below). Packages are signed only with
`--sign <profile>`, the owner's Samsung certificate profile (`tz pack`).

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
`tizen-manifest.xml`, that the `.zip` holds the packages at its top level, and where `tz pack` writes the signed
package (`--target all --sign` looks next to and inside each staged tree).

Submission itself (Seller Office, the certificates, the answer on native libraries for public sellers) is the
owner's.

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
