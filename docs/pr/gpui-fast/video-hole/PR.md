# fast: paint holes that replace the window's pixels with transparent black

## Description

Part of #(issue): GPUI on Samsung TVs (Tizen), video under the window.

**This adds public API** to `gpui`: `Window::paint_hole`, `Scene::has_holes`, `AtlasTextureId::hole` and
`AtlasTextureId::is_hole`. The issue explains why; in short, an element outside `gpui` has to paint the hole, and a
platform outside `gpui` has to know there is one.

### Why

On a TV, video plays on a hardware plane below the window's plane, through the platform's player: protected content
can only be shown there, and decoding into a texture costs a conversion and an upload per frame at best. The plane
shows only where the window's pixels are transparent. GPUI's primitives blend, so painting something transparent leaves
what is under it, and nothing can make a pixel more transparent than it is. The video element needs a primitive that
replaces the pixels instead.

### What

- `Window::paint_hole(bounds, corner_radii)` (in `crates/gpui/src/fast/hole.rs`) paints a rounded rectangle, within
  the current content mask, whose pixels become transparent black. What is painted after it blends over it, so
  controls drawn over the video look as they would over any background. The bounds are snapped edge by edge to device
  pixels, the grid a video plane is positioned on.
- `Scene::has_holes()` is true when the scene paints a hole, directly or inside a layer or rounded clip group it
  carries. A platform makes its window translucent only then; an opaque window saves the compositor a blend.
- A hole is a polychrome sprite whose tile has the reserved texture id `0xDFFF_FFFF` (`AtlasTextureId::hole()`,
  checked with `is_hole()`), below the dynamic textures' range (`0xE000_0000`) and the layer tiles'. The scene, its
  batching and its primitive kinds do not change, and a hole batch keeps its place in the scene's draw order. The two
  methods are public because the renderers live in other crates, as the layer tile ids are.
- The wgpu renderer draws hole batches with a pipeline of their own (`crates/gpui_wgpu/src/fast/hole.rs`): the
  polychrome sprite vertex shader, a fragment that outputs the rounded rectangle's coverage as alpha (honouring the
  rounded clip), and blending `dst * (1 - src_alpha)` on every channel. Inside, the target becomes transparent black;
  antialiased edges and rounded corners fade to the old pixels.
- Other renderers (Metal, Direct3D) have no such texture and skip the batch, as they do for a released atlas texture:
  there, a hole paints nothing. A hole inside a scroll layer's cached tiles clears those tiles' pixels and does not
  reach the window.

Hooks: in `wgpu_renderer.rs` the hole pipeline's field in `WgpuPipelines`, its creation, and the shader module's source
through `crate::fast::hole::shader_source`; in `fast/frame.rs` one batch arm. `docs/upstream-sync.md` lists the
exception. Nothing changes for an app that does not call `paint_hole`.

## How to Test

- `cargo test -p gpui_wgpu --features test-support --lib fast::hole::tests` (3 tests; needs a GPU adapter, Metal on
  macOS or lavapipe on Linux; pixels read back through `WgpuHeadlessRenderer`): a hole is transparent inside and
  leaves the outside; rounded corners keep the pixels outside the radius and fade on the arc; what is drawn after a hole
  blends over it.
- `cargo test -p gpui --features test-support --lib fast::tests::hole` (3 tests): the hole id is its own, bounds snap
  to device pixels, a scene reports its holes.
- `cargo test -p gpui --features test-support`.
- `script/check-upstream`, `cargo fmt -p gpui -p gpui_wgpu -- --check`,
  `cargo clippy -p gpui --all-targets --features gpui/test-support -- -D warnings`,
  `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`.
- Used by a Tizen platform's video player ([dXn.GpuiTizen]([OWNER: link once public])): the player's surface sits
  on the video plane under the hole, with the controls painted over it.

## Checklist

- [ ] `script/check-upstream` passes: three hooks in `wgpu_renderer.rs`, the rest in `fast/`.
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::hole::tests` passes.
- [ ] `cargo test -p gpui --features test-support` passes, `fast::tests::hole` included.
- [ ] `cargo fmt -p gpui -p gpui_wgpu -- --check` and both clippy runs pass.
- [x] The new public API is listed as an exception in `docs/upstream-sync.md`; nothing changes without `paint_hole`.
