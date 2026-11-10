# fast: compute scene damage between frames

## Description

Part of #(issue): GPUI on Samsung TVs (Tizen), the first half of change 5. Nothing calls it yet: #(external-target PR)
draws partial frames with it. It is split out to keep that PR readable; if you prefer one PR, we close this one.

A renderer that keeps the last frame's pixels (an EGL window surface with `EGL_EXT_buffer_age`) only needs to redraw
where the new frame can differ. This computes that, for two consecutive scenes of a window, as a short list of
device-pixel rectangles.

- A scene is summarized as its primitives in paint order (the order of `Scene::batches`): each one a hash of
  everything that decides its pixels except its draw order, and the area it can touch: its bounds clipped by its
  content mask, widened for blur, wavy underlines and a pixel of antialiasing.
- Two summaries are compared with a shortest-edit-script diff (Myers). Primitives on the common subsequence keep
  their content and their relative paint order, so a pixel only they cover is unchanged; the damage is the area of
  every primitive removed or added. An inserted primitive therefore does not damage those painted after it.
- The areas are clamped to the window and merged into at most 8 rectangles; more than 64 areas become one bounding
  rectangle first, so merging stays cheap.
- Damage is conservative, never smaller than what changed. The whole window is damaged on the first frame, after a
  resize, after `DamageTracker::invalidate` (scale change, atlas loss, device recovery), when the scenes differ by
  more than 512 primitives, and always with `GPUI_PARTIAL_REDRAW=0`. Surfaces are damaged every frame (their pixels
  change while their description does not); sprites with a non-identity transform cover the whole window; scroll-layer
  tiles change with their layer's generation, and rounded clip groups with their content and geometry frame.

All of it is in `crates/gpui_wgpu/src/fast/damage.rs` and its tests; no upstream file changes, and the renderer does
not call it. Until #(external-target PR) does, the module carries an `allow(dead_code)` outside tests that says why.
No public API.

#45 computed exact damage in `gpui` and added `Scene::damage`. This stays in `gpui_wgpu`, conservative, and adds
nothing to `Scene`.

## How to Test

- `cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests` (13 tests, no GPU): unchanged scenes
  have no damage; a changed quad damages its bounds only; a moved sprite damages where it was and where it is; a clip
  change damages the clipped primitive; an inserted primitive does not damage those after it; many small changes merge
  into a bounded list inside the window; many changed areas become one rectangle; `GPUI_PARTIAL_REDRAW=0`, a resize,
  `invalidate` and more than 512 changes damage everything; a transformed sprite damages the whole window; a dirty
  layer tile is damaged only when its layer changes; a clip group's vertical radii are part of it; the diff keeps the
  longest common subsequence.
- `script/check-upstream`, `cargo fmt -p gpui_wgpu -- --check`,
  `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`.

## Checklist

- [ ] `script/check-upstream` passes (no upstream file changes).
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests` passes.
- [ ] `cargo fmt -p gpui_wgpu -- --check` and `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`
  pass.
- [x] No new public API; no change to what any renderer draws.
