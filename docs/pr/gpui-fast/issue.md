# gpui-fast on Samsung TVs (Tizen): GL ES 3.0 devices, a pipeline cache, partial redraws and video holes

We are writing a Tizen platform for GPUI so that GPUI apps run on Samsung TVs
([dXnSix/dXn.GpuiTizen]([OWNER: link once public]), its own repository, no change to `gpui_platform`). It builds on
gpui-fast 0.1.13 from crates.io and needs six changes in `gpui_wgpu`, one of them also in `gpui`. They are ready as
branches on [dXnSix/gpui-fast](https://github.com/dXnSix/gpui-fast), each cut from `main` and written to
`docs/upstream-sync.md`: logic in `src/fast/`, one-line hooks in upstream files, `script/check-upstream` clean.

Four add no public API. Two do, and would be new entries in "Where our API differs from upstream's". Before opening
PRs we would like to know whether you want them here or in Zed's GPUI, which of them, and in what shape.

## The TV

- Samsung TVs on Tizen 8.0 and later (2024 models on), and the Tizen TV emulator.
- GPUs with GL ES 3.0 drivers behind EGL 1.4. Some expose GL ES 3.1+, not all of them with storage buffers in vertex
  shaders. The emulator gives GL ES 3.0 through the host's GPU.
- 32-bit ARM userspace (armv7, NEON), also on SoCs with a 64-bit kernel. The 32-bit build is the build.
- One full-screen window on the UI plane, 1920x1080. Video plays on a separate hardware plane under it.
- A per-app memory budget of 300 / 380 MB, GPU memory included, and a watchdog that ends an app whose main loop
  stops answering. GPUI draws on the main thread, so start-up and per-frame costs count.

## The changes

| # | Branch | Touches | New API |
|---|---|---|---|
| 1 | `gles-instance-path` | `gpui_wgpu`: `fast/gles.rs`; 3 hooks in `wgpu_context.rs` | no |
| 2 | `wgpu-backend-env` | `gpui_wgpu`: `fast/gles.rs`; 1 hook in `wgpu_context.rs` | no (reads `WGPU_BACKEND`) |
| 3 | `pipeline-cache` | `gpui_wgpu`: `fast/pipeline_cache.rs`; 2 hooks in `wgpu_context.rs`, 3 in `wgpu_renderer.rs` | no (reads `GPUI_PIPELINE_CACHE_DIR`) |
| 5a | `scene-damage` | `gpui_wgpu`: `fast/damage.rs` and its tests; no upstream file | no |
| 5b | `external-target` | `gpui_wgpu`: `fast/external_target.rs`, a scissor in `fast/frame.rs`; an export in `gpui_wgpu.rs`, 3 visibility bumps | yes |
| hole | `video-hole` | `gpui`: `fast/hole.rs`; `gpui_wgpu`: `fast/hole.rs`, a batch arm in `fast/frame.rs`, 3 hooks in `wgpu_renderer.rs` | yes |

(Our numbering; change 4, keeping `gpui_linux` out of the TV build, is made in the app framework, not here.)

**1. GL ES 3.0 devices.** `gpui_wgpu` already has a renderer path for GPUs without storage buffers, the WebGL2 one
(instance records decoded from a texture, `shaders_webgl.wgsl`), but turns it on only when compiled for the web. On a
native GL ES 3.0 adapter `request_device` fails, because native asks for `Limits::downlevel_defaults()`. The branch
picks the path by what the adapter can do: GL without `DownlevelFlags::VERTEX_STORAGE`, or with no storage buffers per
stage, gets the instance-texture path and the WebGL2 limits; GL adapters get `Rgba8Unorm` atlases. Adapters with
vertex storage keep storage buffers.

**2. GL on request.** `WgpuContext::instance()` asks for `VULKAN | GL`, and Vulkan adapters rank first, so a desktop or
CI machine with Vulkan (lavapipe included) never runs the GL ES path. With `WGPU_BACKEND` set, the backends come from
`wgpu::Backends::from_env()`; unset, nothing changes. #120 (closed) also changed the default, to GL only when Vulkan
finds no adapter; this one keeps today's default.

**3. A pipeline cache on disk.** Every pipeline is made with `cache: None`, so nothing compiled survives a restart.
When the adapter offers `Features::PIPELINE_CACHE` (Vulkan in wgpu 30), the device asks for it, the cache is loaded
from `GPUI_PIPELINE_CACHE_DIR` (default: the platform's cache directory) under `wgpu::util::pipeline_cache_key`, with
`fallback: true`, every pipeline uses it, and its data is written back atomically once the pipelines exist. On GL it
does nothing until wgpu's GL backend implements the cache; a wgpu PR that stores GL program binaries for it is
prepared separately.

**5a. Scene damage.** For two consecutive scenes of a window, the device-pixel rectangles whose pixels can differ: each
primitive is hashed without its draw order, the two sequences are diffed (Myers), and the areas of what was added or
removed are merged into at most eight rectangles. The whole window on the first frame, after a resize or atlas loss,
past 512 changes, and always with `GPUI_PARTIAL_REDRAW=0`. On its own it changes nothing: only 5b reads it. It is
split out to keep 5b readable; one PR is fine with us too.

**5b. Drawing into a supplied target.** A renderer on a wgpu instance, adapter, device and queue the caller created,
drawing a scene into a texture view the caller supplies, with partial redraws by the target's age. See below.

**The hole.** `Window::paint_hole` paints a rounded rectangle that replaces the window's pixels with transparent black,
so a video plane under the window shows through. See below.

## Why two of them need public API

`docs/upstream-sync.md` rule 5 allows no new public API beyond the documented exceptions (`fast::dynamic_textures`,
#86; `WebPlatform::wgpu_context`, #102). These two cannot be reached from a platform crate without it.

### 5b: `WgpuExternalRenderer` and `WgpuFrameDamage`

On GL, wgpu's own surface renders into an offscreen renderbuffer and copies it, flipped, into the window's framebuffer
on every present (`glBlitFramebuffer` in wgpu-hal's `Surface::present`): one full-window copy per frame (1920x1080x4,
about 8 MB read and written), and no way to present only what changed. A platform that owns the EGL context and surface
can instead present partial frames (`EGL_EXT_buffer_age`, `EGL_KHR_partial_update`, `eglSwapBuffersWithDamageKHR`),
register the driver's shader cache on its display (`EGL_ANDROID_blob_cache`), and pick the window's config itself. It
lends that context to wgpu (`wgpu_hal::gles::Adapter::new_external`) and needs GPUI to draw into the framebuffer it
gives it.

From outside `gpui_wgpu`, GPUI's renderer is only reachable through `WgpuRenderer::new`, which makes and presents a
`wgpu::Surface`; `WgpuRendererCore::render_frame`, which already draws into a given view, is `pub(crate)`. The API is
what the platform needs and no more:

- `WgpuExternalRenderer::new(instance, adapter, device, queue, format)`;
- `draw(scene, target_view, size, flip_y, age, before_drawing) -> Result<WgpuFrameDamage>`: `flip_y` for a GL default
  framebuffer (drawn through a texture the renderer keeps and copied upside down, so no shader changes), `age` from
  `EGL_EXT_buffer_age` (0: unknown, draws everything), `before_drawing` gets the damage before any GPU work, for
  `eglSetDamageRegionKHR`;
- `WgpuFrameDamage { damage, redraw }` in the target's coordinates (rows from the bottom when flipped, as EGL counts);
- `sprite_atlas()` for `PlatformWindow::sprite_atlas`, `context()` for dynamic textures, `invalidate()` when the
  target's contents are lost, `release_targets()` while the app is paused.

It is not built for wasm. `WgpuRenderer` and the surface path do not change; everything else stays `pub(crate)`.

### The hole: `Window::paint_hole`, `Scene::has_holes`, `AtlasTextureId::hole` / `is_hole`

On the TV, video plays on a hardware plane below the UI plane, through the platform's player: protected content can
only be shown there, and decoding into a texture costs a conversion and an upload per frame at best. The plane shows
only where the window's pixels are transparent, and GPUI's primitives blend: painting something transparent leaves what
is under it, and nothing makes pixels more transparent. A hole replaces them instead (`dst * (1 - coverage)`), in paint
order, so controls painted after it blend over the video.

- `Window::paint_hole(bounds, corner_radii)` is called by the video element, which lives in the platform crate
  because it also positions the video plane.
- `Scene::has_holes()` lets the platform make its window translucent only while a frame has a hole; an opaque window
  saves the compositor a blend.
- `AtlasTextureId::hole()` / `is_hole()` mark the hole sprites for the renderer crates, as the exported layer tile ids
  do.

A hole is a polychrome sprite with a reserved texture id (`0xDFFF_FFFF`), so the scene, its batching and its primitive
kinds do not change. Only the wgpu renderer draws holes; the others find no such texture and skip the batch.

## Numbers from the TV

Measured by us on the devices below, both renderer paths (5b with partial presents, and wgpu's own surface), cache
cold and warm.

| | Value |
|---|---|
| Stripped library size | [DEV-3: stripped library size on <device>] |
| Launch to first frame | [DEV-3: launch to first frame on <device>, cache cold / warm, per renderer path] |
| Memory (PSS) after start | [DEV-3: PSS after start on <device>, against the 300 / 380 MB budget] |
| Memory (PSS) after scrolling a long list | [DEV-3: PSS after scrolling the long list on <device>] |
| Frame times while scrolling with key repeat | [DEV-3: frame times on <device>, 5b with partial presents vs wgpu's surface] |

## Questions

1. **Here or in Zed's GPUI?** `gpui_wgpu`'s files are Zed's, and 1–3 change code paths that are upstream's
   (`uses_webgl_instance_data`, the device limits, `WgpuContext::instance`, pipeline creation). Would you rather take
   them here, or should they go to Zed's GPUI (one by one, or all)? With the code in `fast/`, moving a change to Zed is
   replacing its hooks with the code.
2. **The two API exceptions.** Are 5b and the hole acceptable as exceptions in this shape, or would you prefer another
   one (a cargo feature, `#[doc(hidden)]`, a different entry point)?
3. **5a and 5b**: two PRs, or one?
4. **The pipeline cache's default**: on (a file in the user's cache directory, on Linux with Vulkan today), or only when
   `GPUI_PIPELINE_CACHE_DIR` is set?
5. **CI**: the GPU tests of 5b and the hole run on the headless renderer, like `fast::dynamic_textures` on macOS. Should
   the PRs add CI steps for them?

The branches, in the order we would open them: `gles-instance-path`, `wgpu-backend-env`, `pipeline-cache`,
`scene-damage`, `external-target`, `video-hole`.
