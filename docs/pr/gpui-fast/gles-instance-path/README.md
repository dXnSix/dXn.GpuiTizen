# gles-instance-path

| | |
|---|---|
| Change | 1, GL ES 3.0 devices |
| Upstream PR | ready, not opened: nothing is opened at `longbridge/gpui-fast` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Branch | `gles-instance-path` on `dXnSix/gpui-fast`, head `9906814ad`, pushed 2026-11-12 (rebased from `fbd3ed9`) |
| Base | `upstream/main` `d1d6286` (#139, fetched 2026-11-12); rebased onto it without conflicts in its own files |
| Size | 3 files, +215 −1: `crates/gpui_wgpu/src/fast/gles.rs` (new, with its tests), `fast/mod.rs` (+1), `wgpu_context.rs` (+5 −1, three hooks) |
| Its commits | `9906814` fast: draw GL ES 3.0 devices through the instance-texture path |
| Text | [PR.md](PR.md) |
| On `tizen` | `25fc4a9` |
| Needs | the issue ([../issue.md](../issue.md)); no new API |

GL adapters without vertex storage buffers (GL ES 3.0, and GL ES 3.1+ drivers that expose none to vertex shaders) use
the renderer's WebGL2 path natively: instance data from a texture, the WebGL2 limits within the adapter's, and
`Rgba8Unorm` atlases on every GL adapter. Upstream turns that path on only for wasm, and a native GL ES 3.0 adapter fails
`request_device` (it cannot give `downlevel_defaults()`).

The three hooks in `wgpu_context.rs`: `required_limits` passes upstream's limits through
`crate::fast::gles::required_limits`, `select_color_texture_format` returns `crate::fast::gles::atlas_format` when it
has one, and `uses_webgl_instance_data` returns `crate::fast::gles::uses_instance_textures(&self.adapter)`.

Visible to others: GL adapters with vertex storage keep storage buffers, but get `Rgba8Unorm` atlases where native
code chose `Bgra8Unorm` before. Vulkan, Metal and the web are unchanged.

`wgpu-backend-env` (change 2) creates `fast/gles.rs` too: whichever merges second adds its part to the file.

## Checks

To run on the branch before opening:

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::gles::tests::device   # 5 tests, no GPU
cargo test -p gpui_wgpu --features test-support --lib webgl_                      # upstream's WebGL2 shader tests
script/check-upstream
cargo fmt -p gpui_wgpu -- --check
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
cargo test -p gpui --features test-support                                        # upstream CI's test step
cargo +nightly check -p gpui_web --lib --target wasm32-unknown-unknown                  # the web path goes through fast/gles.rs too
```

On Linux, the GL ES path end to end (Mesa, no storage buffers; `WGPU_BACKEND` needs change 2, or a machine without a
Vulkan driver):

```bash
WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 MESA_EXTENSION_OVERRIDE=-GL_ARB_shader_storage_buffer_object \
  RUST_LOG=info cargo run -p gpui --example hello_world
# the log says "... (Gl): instance textures and WebGL2 limits (no vertex storage buffers)"
```

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS; nothing recorded on this branch
itself):

- 2026-11-08 23:07 UTC: `fast::gles::tests::device` passed (5 tests); `script/check-upstream` passed;
  `cargo clippy -p gpui_wgpu --all-targets --features test-support -- -D warnings` passed (with `wgpu-test-lints` on
  `tizen`).
- 2026-11-09 00:38 and 02:08 UTC: the examples' `gles_smoke` tests in the Linux VM (Mesa llvmpipe, `WGPU_BACKEND=gl`,
  storage buffers hidden) drew 60 frames each and logged the instance-texture path, against the release workspace
  generated from `tizen`.
- Not run: the upstream CI steps on Linux and Windows, the web build.


### Run on the rebased tip (2026-11-12)

On `9906814ad`, macOS (`../gpui-fast-wt/pr`): `fast::gles::tests::device` 5 passed, `webgl_` 3, `cargo test -p gpui --features test-support` 842 (and its doc tests), `script/check-upstream`, `cargo fmt`, the `gpui_wgpu` clippy, and `cargo +nightly check -p gpui_web --lib --target wasm32-unknown-unknown` (nightly, as upstream's `release.yml` checks the web crate; the stable command above cannot build `wasm_thread`). Logs: `artifacts-temp/agentx/tizen-gpui-platform/t25-gf/gles-instance-path.log`.

## Updating and opening

Rebase onto the fresh `upstream/main`, run the checks above, tick the checklist in `PR.md`, replace `#(issue)`. The
owner runs this:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast
git fetch upstream origin
git worktree add ../gpui-fast-wt/pr gles-instance-path 2>/dev/null || git -C ../gpui-fast-wt/pr switch gles-instance-path
cd ../gpui-fast-wt/pr
git rebase upstream/main
# the checks above, then
git push --force-with-lease -u origin gles-instance-path
gh pr create --repo longbridge/gpui-fast --base main --head dXnSix:gles-instance-path \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/gles-instance-path/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/gles-instance-path/PR.md)
```

Then record the PR number here and in [../README.md](../README.md).
