# fast: draw into a target the caller supplies, with partial redraws

## Description

Part of #(issue): GPUI on Samsung TVs (Tizen), change 5. Needs #(scene-damage PR) (scene damage), merged first.

**This adds public API** to `gpui_wgpu`: `WgpuExternalRenderer` and `WgpuFrameDamage`. The issue explains why; in
short, a platform that owns its EGL context and surface cannot otherwise draw GPUI into it.

### Why

On GL, wgpu's own surface renders into an offscreen renderbuffer and copies it, flipped, into the window's
framebuffer on every present (`glBlitFramebuffer` in wgpu-hal's `Surface::present`). That is one full-window copy per
frame (1920x1080x4, about 8 MB read and written on a TV), and the surface cannot present only what changed. A platform
that owns the EGL context and window surface can present partial frames (`EGL_EXT_buffer_age`,
`EGL_KHR_partial_update`, `eglSwapBuffersWithDamageKHR`), register the driver's shader cache on its display
(`EGL_ANDROID_blob_cache`) and choose its config. It lends the context to wgpu (`wgpu_hal::gles::Adapter::new_external`,
`Instance::create_adapter_from_hal`) and wraps the window's framebuffer as a texture
(`Texture::default_framebuffer`), but GPUI's renderer can only be reached through `WgpuRenderer::new`, which makes and
presents a `wgpu::Surface`. `WgpuRendererCore::render_frame` already draws into a given view; it is `pub(crate)`.

### What

`WgpuExternalRenderer` (`crates/gpui_wgpu/src/fast/external_target.rs`, not built for wasm):

- `new(instance, adapter, device, queue, format) -> Result<Self>`: a renderer on a device the caller made, drawing into
  views of `format`. It installs the device's lost and uncaptured-error callbacks, as `WgpuRenderer` does.
- `draw(scene, target, size, flip_y, age, before_drawing) -> Result<WgpuFrameDamage>`: draws `scene` into `target`, a
  view of a `size` texture whose contents were drawn `age` frames ago.
  - The damage comes from `fast::damage` (#(scene-damage PR)); the renderer keeps the last 4 frames' damage. Given the
    age, it loads the target, clears the bounding rectangle of the last `age` frames' damage, and draws only inside
    it, scissored (a pass has one scissor rectangle, hence the bounding box). Age 0 (unknown), a target older than
    the history, or a frame damaged whole draws everything; a frame without damage draws nothing.
  - `before_drawing` receives the frame's damage before any GPU work, for `eglSetDamageRegionKHR`; `draw` returns the
    same, for `eglSwapBuffersWithDamageKHR`. Both are in the target's coordinates (rows from the bottom when
    flipped, as EGL counts them).
  - `flip_y` is for a GL default framebuffer. The frame is drawn into a texture the renderer keeps, which always
    holds the previous frame, and the redrawn region is copied into the target upside down by a scissored pass.
    GPUI's shaders are untouched; flipping the projection instead would also need every fragment shader's
    `position` flipped. The cost is one more target-sized texture.
  - A GPU error raised since the previous frame, or a lost device, fails the frame; after any failure the next frame
    is drawn whole.
- `sprite_atlas()` for the platform window's `PlatformWindow::sprite_atlas`; `context()`, the context dynamic textures
  painted into its frames are made from; `invalidate()`, when the target's contents or the atlas were lost;
  `release_targets()`, which also frees the target-sized textures while the app is paused.

`WgpuFrameDamage { damage, redraw }`: where the frame's pixels differ from the previous frame's, and what it draws
into the target for its age. Public fields, `Vec<Bounds<DevicePixels>>`.

Hooks and the rest:

- `fast/frame.rs`: `FrameState::partial`; a partial frame keeps the texture's contents and scissors the main passes.
- `gpui_wgpu.rs`: the export (`#[cfg(not(target_family = "wasm"))]`).
- Visibility bumps to `pub(crate)`: `WgpuContext::from_native_adapter_and_device`,
  `WgpuContext::select_color_texture_format`, `WgpuResources::invalidate_intermediate_textures`.
- `docs/upstream-sync.md`: the exception, under "Where our API differs from upstream's", the third after
  `fast::dynamic_textures` and `WebPlatform::wgpu_context()`.

`WgpuRenderer` and the surface path do not change, and nothing changes for an app that does not build an
`WgpuExternalRenderer`.

### On a TV

| | wgpu's surface | This renderer, partial presents |
|---|---|---|
| Frame times while scrolling with key repeat | [OWNER: DEV-3: frame times on <device>, wgpu's surface] | [OWNER: DEV-3: frame times on <device>, platform EGL with partial presents] |
| Launch to first frame, cache cold / warm | [OWNER: DEV-3: launch to first frame on <device>, wgpu's surface] | [OWNER: DEV-3: launch to first frame on <device>, platform EGL] |
| Memory (PSS) after scrolling the long list | [OWNER: DEV-3: PSS on <device>, wgpu's surface] | [OWNER: DEV-3: PSS on <device>, platform EGL] |

## How to Test

- `cargo test -p gpui_wgpu --features test-support --lib fast::external_target::tests` (13 tests; needs a GPU
  adapter, Metal on macOS or lavapipe on Linux; they use `WgpuContext::create_headless`'s device):
  - draws like the surface path (pixel for pixel against `WgpuHeadlessRenderer`), and upside down for a flipped target;
  - a partial frame keeps every pixel outside its damage and matches a full redraw inside, flipped and unflipped, and
    clears what disappears;
  - an unknown age or an old target redraws everything; an older target gets the damage of every frame since;
  - an unchanged frame draws nothing and is not counted in the age;
  - the damage is handed over before drawing, in the target's coordinates;
  - a GPU error fails the next frame and the one after is whole;
  - released targets are freed and the next frame is whole;
  - `context()` is on the caller's device, and dynamic textures can be made from it;
  - the redraw region follows the target's age (no GPU).
- `cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests`.
- `script/check-upstream`, `cargo fmt -p gpui_wgpu -- --check`,
  `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`,
  `cargo check -p gpui_web --lib --target wasm32-unknown-unknown` (the module is left out on the web).
- Used by a Tizen platform that owns its EGL context ([dXn.GpuiTizen]([OWNER: link once public])), on the Tizen TV
  emulator and TVs.

## Checklist

- [ ] `script/check-upstream` passes: three visibility bumps and an export in upstream files, the rest in `fast/`.
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::external_target::tests` passes (13 tests).
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests` passes.
- [ ] `cargo fmt -p gpui_wgpu -- --check` and `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`
  pass; the wasm check passes.
- [ ] The TV numbers above are filled in.
- [x] The new public API is listed as an exception in `docs/upstream-sync.md`; `WgpuRenderer` is unchanged.
