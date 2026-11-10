# fast: keep compiled pipelines in a cache on disk

## Description

Part of #(issue): GPUI on Samsung TVs (Tizen), change 3.

wgpu keeps compiled pipelines across starts only when it is given a `wgpu::PipelineCache`, and GPUI creates every
pipeline with `cache: None`. Each start compiles GPUI's pipelines from scratch while the first window's renderer is
made, on the main thread.

With this change, when the adapter offers `wgpu::Features::PIPELINE_CACHE` and `wgpu::util::pipeline_cache_key` gives
a key for it:

- the device requests the feature;
- a pipeline cache is created from the file `<dir>/<key>`, with `fallback: true`, so data wgpu cannot use (another
  driver version, a truncated file) is ignored instead of failing;
- every GPUI pipeline is created with it;
- once the pipelines exist, `get_data()` is written back to a temporary file and renamed into place, so a reader never
  sees a partial file.

`<dir>` is `GPUI_PIPELINE_CACHE_DIR` when set, else the platform's cache directory (`$XDG_CACHE_HOME` or `~/.cache`,
`~/Library/Caches`, `%LOCALAPPDATA%`) with `gpui-fast/pipeline-cache`. Without the feature or a key nothing is read or
written. In wgpu 30 only Vulkan offers the feature, so today this applies to Linux with Vulkan; GL gets it once wgpu's
GL backend implements the cache (with program binaries, proposed to wgpu separately), which is what GL ES TVs need.

- `crates/gpui_wgpu/src/fast/pipeline_cache.rs`: the feature check, the cache of the current device in a thread-local
  slot (GPUI makes its device and pipelines on the main thread and uses one device at a time; a new device clears the
  slot first, so an old device is never kept alive by it), loading, the atomic write, and tests.
- `wgpu_context.rs`: two hooks (request the feature, open the cache once the device exists).
- `wgpu_renderer.rs`: three hooks in `WgpuRendererCore`'s pipeline creation (get the cache, pass it to
  `create_pipeline`, store the data after the last pipeline).

No public API. What users see: a file in their cache directory on Linux with Vulkan, and fewer compiles on the next
start. If you prefer this opt-in (only with `GPUI_PIPELINE_CACHE_DIR` set), it is one line in `cache_dir`.

## How to Test

- `cargo test -p gpui_wgpu --features test-support --lib fast::pipeline_cache::tests`: a second start finds the file
  and creates the cache from it; corrupt or empty data falls back without error; nothing is read or written without the
  feature, a key or a directory; the write leaves no temporary file; the directory comes from the variable or the
  platform.
- On Linux with Vulkan:

  ```sh
  dir=$(mktemp -d)
  GPUI_PIPELINE_CACHE_DIR=$dir RUST_LOG=info cargo run -p gpui --example hello_world   # logs "pipeline cache ... (new)"
  ls -l $dir                                                                            # one wgpu_pipeline_cache_vulkan_* file
  GPUI_PIPELINE_CACHE_DIR=$dir RUST_LOG=info cargo run -p gpui --example hello_world   # logs "pipeline cache ... (loaded)"
  ```

- `script/check-upstream`, `cargo fmt -p gpui_wgpu -- --check`,
  `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`.

## Checklist

- [ ] `script/check-upstream` passes: five one-line hooks, the logic in `fast/pipeline_cache.rs`.
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::pipeline_cache::tests` passes.
- [ ] `cargo test -p gpui --features test-support` passes.
- [ ] `cargo fmt -p gpui_wgpu -- --check` and `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`
  pass.
- [ ] Checked on Linux with Vulkan that the second start loads the file.
- [x] No new public API.
