# gles-program-binaries

| | |
|---|---|
| Upstream PR | ready, not opened, and not opened within the epic (`d35-epic-ends-before-release`): after the code has run on a real TV (`s25-owner-devices`), the branch is rebased again and the run goes into the PR's Testing section (`t26-upstream-after-devices`); the owner opens it later ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Branch | `gles-program-binaries` on `dXnSix/wgpu`, head `8a00e91e2`, pushed 2026-11-12 (rebased from `03cab4b4b`) |
| Base | `upstream/trunk` `c92258fca` (#10889, fetched 2026-11-12). The rebase from `b672a5400` was clean, #10293's `adapter.rs` and `CHANGELOG.md` lines and #8014's GLES changes included; the entry stays under GLES in New Features |
| Size | 8 files, +752 −42: `wgpu-hal/src/gles/pipeline_cache.rs` (new: keys, the data format, its tests), `gles/device.rs`, `gles/adapter.rs`, `gles/mod.rs`, `wgpu-core/src/pipeline_cache.rs` (a GL adapter key and its test), `wgpu/src/util/mod.rs` (`pipeline_cache_key` for GL), `tests/tests/wgpu-gpu/pipeline_cache.rs` (a GL GPU test), `CHANGELOG.md` (+1) |
| Its commits | `af64e96d3` [gles] Implement PipelineCache with program binaries; `8a00e91e2` [gles] Prove program binary loading in tests, store shared programs in caches |
| Text | [PR.md](PR.md) |
| From v30 | `fe2a46eac`, `de226d2fa` on `v30-gles-program-binaries`, merged into `v30-tizen` |
| Needs | nothing; no new public API (`Features::PIPELINE_CACHE` and `util::pipeline_cache_key` exist, GL now reports or answers them) |
| Used by | gpui-fast's `pipeline-cache` ([../../gpui-fast/pipeline-cache/](../../gpui-fast/pipeline-cache/)): on GL it does nothing until wgpu's GL backend has the feature |

Where the context offers program binaries (GL ES 3.0, GL 4.1 or `GL_ARB_get_program_binary`, with
`GL_NUM_PROGRAM_BINARY_FORMATS > 0`, native only), the adapter reports `Features::PIPELINE_CACHE`. Each program is
keyed by a stable 128-bit hash of its GLSL sources and the adapter's `GL_RENDERER` and `GL_VERSION`; a cache holds
`glGetProgramBinary`'s output per key; `create_program` loads a cached binary with `glProgramBinary` and compiles when
the driver rejects it, replacing the entry; every program a pipeline uses (also one an earlier pipeline linked,
shared through the program cache) is stored into the cache passed to it. The data (`gles/pipeline_cache.rs`) has a
magic, version and adapter key, and a checksum per entry, so stale or corrupt data costs a compile, never an error.
wgpu-core gives GL caches an adapter key from the renderer and version (GL has no device id); `pipeline_cache_key`
returns `wgpu_pipeline_cache_gl_<renderer>`.

### Ported from v30

Cherry-picked from `fe2a46eac` and `de226d2fa`; adapted to `trunk`:

- `PipelineCache`'s entries use `wgpu_sync::Mutex` (trunk moved wgpu-hal off `parking_lot`, 8573e7a44).
- `#[cfg_attr(target_arch = "wasm32", ...)]` on `create_query_set` is trunk's `target_family = "wasm"`.
- The GL GPU test is declared with `#[apply(gpu_test!)]`, trunk's form (d0ecda96c), in both commits, so each builds.
- The `CHANGELOG.md` entry sits under `## Unreleased` → Added/New Features → GLES from the first commit; on v30 the
  second commit moved it there, so its message's last sentence ("The changelog entry moves to New Features") is
  dropped. The entry has no PR link yet: add `in [#NNNNN](https://github.com/gfx-rs/wgpu/pull/NNNNN)` once opened.

### Before opening

- The first commit adds test-only counters on `wgpu_hal::gles::Device` (`program_binary_counts`, a `pub fn`) that the
  second removes; the PR text asks for a squash. Squashing locally before opening is also fine.
- wgpu-core and `wgpu-hal` changes are usually not changelog-worthy, but the user-visible effect is
  (`Features::PIPELINE_CACHE` on GL), so the entry stays.

## Checks

Run 2026-11-12 on `8a00e91e2` (clippy also on `af64e96d3`), in the Lima VM (see [../README.md](../README.md#checks)),
from the worktree:

```bash
cd /Users/dinko/work-private/projects/dXn/wgpu-wt/trunk      # gles-program-binaries checked out
R=/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/scripts/lima-run.sh
$R cargo test -p wgpu-hal --features gles --lib
#   test result: ok. 9 passed; 0 failed  (gles::pipeline_cache::tests: keys_cover_sources_their_order_and_the_adapter,
#   data_round_trips, stale_or_corrupt_data_falls_back; and six existing)
$R cargo test -p wgpu-core --lib pipeline_cache
#   test result: ok. 13 passed; 0 failed  (the new gl_adapters_are_keyed_by_renderer_and_version and 12 existing)
$R bash -c 'WGPU_INFO_SAVE_GPUCONFIG_REPORT=1 cargo test -p wgpu-info generate_gpuconfig_report && cargo test -p wgpu-test --test wgpu-gpu -- pipeline_cache --test-threads=1'
#   generate_gpuconfig_report: 1 passed
#   [Executed] [Gl/llvmpipe (LLVM 20.1.2, 128 bits)/0] pipeline_cache and pipeline_cache_gl_program_binaries
#   test result: ok. 2 passed; 0 failed; 425 filtered out
$R cargo clippy -p wgpu-hal -p wgpu-core -p wgpu -p wgpu-test --features wgpu-hal/gles --all-targets -- -D warnings
#   passed on af64e96d3 and on 8a00e91e2
cargo fmt --check -p wgpu-hal -p wgpu-core -p wgpu -p wgpu-test   # on the Mac: clean
```

The GPU tests read `.gpuconfig`, which `cargo xtask test` writes the same way (wgpu-info's
`generate_gpuconfig_report`), and need `--test-threads=1` outside nextest. `lima-run.sh` sets `WGPU_BACKEND=gl`, so
only the GL adapter runs (`4.5 (Core Profile) Mesa 25.2.8`).

Logs: `artifacts-temp/agentx/tizen-gpui-platform/t25-wgpu-pb-hal-lib.log`, `t25-wgpu-pb-core-lib.log`,
`t25-wgpu-pb-gpu.log`, `t25-wgpu-pb-clippy.log`, `t25-wgpu-pb1-clippy.log`, `t25-wgpu-pb-fmt.log`. The first run
(2026-11-10, on `03cab4b4b` over `b672a5400`, logs `t22-wgpu-pb-*`) passed the same, with 6 hal tests then.

Not run: the tests on `af64e96d3` alone, the full `cargo xtask test`, GL ES drivers (the test runs on desktop GL
4.5 in the VM), Vulkan's `pipeline_cache` case (no Vulkan in the run), WebGL (program binaries are off there:
`cfg!(native)`), other platforms.

## Updating and opening

Rebase onto the latest `upstream/trunk`, run the checks, fill in the PR text's "LLM Use?" section and rewrite its
description in your own words (wgpu's policy, see [../README.md](../README.md#rules-for-this-area)), tick the
checklist. The owner runs this:

```bash
cd /Users/dinko/work-private/projects/dXn/wgpu
git fetch upstream origin
git worktree add ../wgpu-wt/trunk gles-program-binaries 2>/dev/null || git -C ../wgpu-wt/trunk switch gles-program-binaries
cd ../wgpu-wt/trunk
git rebase upstream/trunk          # only if trunk moved again since c92258fca; then the checks above and
git push --force-with-lease origin gles-program-binaries
# owner runs this
gh pr create --repo gfx-rs/wgpu --base trunk --head dXnSix:gles-program-binaries \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/wgpu/gles-program-binaries/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/wgpu/gles-program-binaries/PR.md)
cd /Users/dinko/work-private/projects/dXn/wgpu && git worktree remove ../wgpu-wt/trunk
```

Then put the PR number into the `CHANGELOG.md` entry (a commit on the branch), here and in
[../README.md](../README.md).
