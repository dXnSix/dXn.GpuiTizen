# pipeline-cache

| | |
|---|---|
| Change | 3, a pipeline cache on disk |
| Upstream PR | not opened |
| Branch | `pipeline-cache`, local in `/Users/dinko/work-private/projects/dXn/gpui-fast`, not pushed |
| Base | `upstream/main` `fb8cf94` (2026-11-08, #118). 7 commits behind the fetched `upstream/main` `a427a7d`, 8 behind live `main` `820088284`; none of them touches `crates/gpui_wgpu`, and the branch merges onto `a427a7d` without conflict. Not rebased |
| Size | 4 files, +279 −1: `crates/gpui_wgpu/src/fast/pipeline_cache.rs` (new, with its tests), `fast/mod.rs` (+1), `wgpu_context.rs` (+2, two hooks), `wgpu_renderer.rs` (+3 −1, three hooks) |
| Its commits | `a21267e` fast: keep compiled pipelines in a cache on disk; `5fa71b0` fast: drop the previous device's pipeline cache when opening a new one; `9864e93` fast: keep the pipeline cache slot on the main thread |
| Text | [PR.md](PR.md) |
| On `tizen` | `6b5f88c`, `8e88253`, `3b6ea11` (`fast/pipeline_cache.rs` identical) |
| Needs | the issue ([../issue.md](../issue.md)); no new API. If `video-hole` merges first, pass the cache to the hole pipeline too (see below) |

When the adapter offers `wgpu::Features::PIPELINE_CACHE` and `wgpu::util::pipeline_cache_key` gives a key, the device
requests the feature, a `wgpu::PipelineCache` is created from `<dir>/<key>` with `fallback: true`, every GPUI pipeline
is created with it, and `get_data()` is written back (temporary file, then rename) once the pipelines exist. `<dir>` is
`GPUI_PIPELINE_CACHE_DIR`, else `$XDG_CACHE_HOME` or `~/.cache` (Linux), `~/Library/Caches` (macOS), `%LOCALAPPDATA%`
(Windows), each with `gpui-fast/pipeline-cache`. The cache lives in a thread-local slot for the current device (GPUI
makes its device and pipelines on the main thread); opening a new device clears it first. Only devices made by
`WgpuContext::create_device` get a cache: a renderer on a device the caller made (`external-target`) gets none.

In wgpu 30 only Vulkan has the feature, so today it does something on Linux with Vulkan only. On GL (the TVs) it does
nothing until wgpu's GL backend implements the cache; the fork's wgpu branch `v30-gles-program-binaries` does that with
program binaries, and its upstream text is UP-3, not this PR.

Hooks: `required_features |= crate::fast::pipeline_cache::features(adapter)` and
`crate::fast::pipeline_cache::open(adapter, &device)` in `wgpu_context.rs`; in `WgpuRendererCore`'s pipeline creation
`let fast_pipeline_cache = crate::fast::pipeline_cache::get(device)`, `cache: Option::as_ref(&fast_pipeline_cache)`
and `crate::fast::pipeline_cache::store(device)`.

### With `video-hole`

`video-hole` creates the hole pipeline with `cache: None`. Whichever of the two merges second changes that call to
`crate::fast::hole::pipeline(device, layouts, &shader_module, surface_format, Option::as_ref(&fast_pipeline_cache))`,
before `store`, as `tizen` does.

### Before opening

- `5fa71b0` and `9864e93` have no message body; give them one if they stay separate commits. `5fa71b0` clears the
  slot before deciding whether the new device gets a cache, so a rejected adapter's device is not kept alive;
  `9864e93` moves the slot from a `static Mutex` to a `thread_local!`.
- The question in the issue: on by default, or only when `GPUI_PIPELINE_CACHE_DIR` is set. Changing to opt-in is one
  line in `cache_dir`.

## Checks

To run on the branch before opening:

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::pipeline_cache::tests   # 5 tests, no GPU
script/check-upstream
cargo fmt -p gpui_wgpu -- --check
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
cargo test -p gpui --features test-support
```

On Linux with Vulkan, the cache end to end:

```bash
dir=$(mktemp -d)
GPUI_PIPELINE_CACHE_DIR=$dir RUST_LOG=info cargo run -p gpui --example hello_world   # "pipeline cache ... (new)"; close it
ls -l $dir                                                                            # wgpu_pipeline_cache_vulkan_<vendor>_<device>
GPUI_PIPELINE_CACHE_DIR=$dir RUST_LOG=info cargo run -p gpui --example hello_world   # "pipeline cache ... (loaded)"
```

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS; nothing recorded on this branch
itself):

- 2026-11-08 23:07 UTC: `fast::pipeline_cache::tests` passed (5 tests); `script/check-upstream` and the `gpui_wgpu`
  clippy passed. That was after `5fa71b0` (23:04 UTC) and before `9864e93` (00:11 UTC), whose copy on `tizen`
  (`3b6ea11`) went into the release workspace checked at 00:38 and 02:08 UTC; the unit tests were not run again.
- Not run: the end-to-end run on Linux with Vulkan (macOS has no pipeline cache feature, so nothing is written there),
  the upstream CI steps on Linux and Windows.

## Updating and opening

Rebase onto the fresh `upstream/main`, run the checks, tick the checklist in `PR.md`, replace `#(issue)`. The owner runs
this:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast
git fetch upstream origin
git worktree add ../gpui-fast-wt/pr pipeline-cache 2>/dev/null || git -C ../gpui-fast-wt/pr switch pipeline-cache
cd ../gpui-fast-wt/pr
git rebase upstream/main
# the checks above, then
git push --force-with-lease -u origin pipeline-cache
gh pr create --repo longbridge/gpui-fast --base main --head dXnSix:pipeline-cache \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/pipeline-cache/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/pipeline-cache/PR.md)
```

Then record the PR number here and in [../README.md](../README.md).
