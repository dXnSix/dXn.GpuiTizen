# fast: draw GL ES 3.0 devices through the instance-texture path

## Description

Part of #(issue): GPUI on Samsung TVs (Tizen), change 1.

GL ES 3.0 has no storage buffers, and some GL ES 3.1+ drivers expose none to vertex shaders. `gpui_wgpu` already has a
renderer path for that, the WebGL2 one (`shaders_webgl.wgsl`, instance records decoded from a texture), but turns it
on only when compiled for the web. On a native GL ES 3.0 adapter (Samsung TVs, the Tizen TV emulator, desktop GL
drivers without vertex storage) the device is asked for `Limits::downlevel_defaults()`, which the adapter cannot give,
and `request_device` fails.

This picks the path by what the adapter can do, not by the target:

- `uses_webgl_instance_data()` is true for a GL adapter without `DownlevelFlags::VERTEX_STORAGE` or with
  `max_storage_buffers_per_shader_stage == 0`, and on the web as before.
- For those adapters the device asks for `Limits::downlevel_webgl2_defaults()` within the adapter's resolution and
  alignment, as the web build does.
- GL adapters get `Rgba8Unorm` atlases when they can sample and upload them (GL ES has no BGRA textures without an
  extension). The web build did this already.

The rest of the renderer already follows `uses_webgl_instance_data()`: bind group layouts, the shader source, the
instance-buffer alignment, and no dual-source blending on that path. Nothing else changes.

- `crates/gpui_wgpu/src/fast/gles.rs`: the three decisions over the adapter's backend, downlevel flags and limits,
  taken from an `AdapterFacts` value so they are tested without a GPU. The chosen path is logged once per device at
  `info`.
- `wgpu_context.rs`: three one-line hooks (the device limits, the atlas format, `uses_webgl_instance_data`).

What changes for existing adapters: GL adapters with vertex storage keep storage-buffer instance data but get
`Rgba8Unorm` atlases instead of `Bgra8Unorm`. Vulkan, Metal and the web build are unchanged. No public API.

## How to Test

- `cargo test -p gpui_wgpu --features test-support --lib fast::gles::tests::device`: GL ES 3.0 gets instance
  textures and limits the adapter can give, GL ES 3.1 without vertex storage gets instance textures, GL with vertex
  storage keeps storage buffers, Vulkan is unchanged, the web keeps its path.
- `cargo test -p gpui_wgpu --features test-support --lib webgl_`: the existing WebGL2 shader tests (the shader without
  storage buffers is valid WGSL, record sizes and layouts match the decoder).
- On Linux with Mesa, the path end to end. With `WGPU_BACKEND` (#(wgpu-backend-env PR)) or on a machine without a
  Vulkan driver:

  ```sh
  WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 MESA_EXTENSION_OVERRIDE=-GL_ARB_shader_storage_buffer_object \
    RUST_LOG=info cargo run -p gpui --example hello_world
  ```

  Hiding `GL_ARB_shader_storage_buffer_object` makes Mesa report GL without storage buffers, as on GL ES 3.0. The
  window draws, and the log says `instance textures and WebGL2 limits (no vertex storage buffers)`. Without this
  change the device asks for `downlevel_defaults()` there, which include storage buffers the adapter does not have.
- `script/check-upstream`, `cargo fmt -p gpui_wgpu -- --check`,
  `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`,
  `cargo check -p gpui_web --lib --target wasm32-unknown-unknown` (the web build takes the same decisions).

## Checklist

- [ ] `script/check-upstream` passes: the logic is in `fast/gles.rs`, upstream files only hold hooks.
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::gles::tests::device` and `webgl_` pass.
- [ ] `cargo test -p gpui --features test-support` passes.
- [ ] `cargo fmt -p gpui_wgpu -- --check` and `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`
  pass.
- [ ] The web build checks.
- [ ] Drawn on Linux with Mesa and storage buffers hidden (the command above).
- [x] No new public API.
