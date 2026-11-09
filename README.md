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

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
