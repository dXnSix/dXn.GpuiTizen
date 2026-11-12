# wgpu: GL ES on Samsung TVs (Tizen)

Two changes to wgpu's GLES backend that dXn.GpuiTizen's TV builds carry in the fork (`dXnSix/wgpu`, branch
`v30-tizen`), proposed upstream to [gfx-rs/wgpu](https://github.com/gfx-rs/wgpu) so that the TV builds can use a
released wgpu instead of the fork. Both live in `wgpu-hal/src/gles/`; the program binaries also add a GL case to
wgpu-core's pipeline cache key and to `wgpu::util::pipeline_cache_key`. Neither adds public API.

Each folder holds the branch's `README.md` (status, base, commits, checks, how to update and open it) and `PR.md`
(the title and the description to paste, in wgpu's pull request template).

## The PRs, in order

Both on the fork `dXnSix/wgpu` (checkout `/Users/dinko/work-private/projects/dXn/wgpu`), local only: `origin` has
only `trunk` as of the last fetch. Each is cut from `upstream/trunk` on its own; they touch different files and
merge with each other cleanly (`git merge-tree`). Either can be opened first. Sizes are
`git diff --stat upstream/trunk <branch>`.

| # | Folder | Branch | Base | Title | Size | Connections | Status |
|---|---|---|---|---|---|---|---|
| 01 | [egl14-wayland/](egl14-wayland/) | `egl14-wayland` | `c92258fca` | [gles] Support EGL 1.4 drivers on Wayland | 2 files, +406 −97 | fixes [#10708](https://github.com/gfx-rs/wgpu/issues/10708) | ready, not opened: rebased, checked and pushed 2026-11-12 (`ef3af726c`) |
| 02 | [gles-program-binaries/](gles-program-binaries/) | `gles-program-binaries` | `b672a5400` | [gles] Implement PipelineCache with program binaries | 8 files, +752 −42 | gpui-fast's `pipeline-cache` uses it on GL ([../gpui-fast/pipeline-cache/](../gpui-fast/pipeline-cache/)) | not opened |

### The base (2026-11-10)

- `b672a5400` is "[naga] HitObject support (#10311)", 2026-11-06, `upstream/trunk` as last fetched. Nothing was
  fetched for this work.
- The live `gfx-rs/wgpu` `trunk` is `6838055e0` (`gh api`, not fetched): two commits more, #9624 (NV12 and P010 on
  Metal) and #10293 (`Features::CORE_FEATURES_AND_LIMITS`). #10293 edits `wgpu-hal/src/gles/adapter.rs` a few lines
  from where `gles-program-binaries` adds `program_binaries` to `AdapterShared`, and both add to `CHANGELOG.md`;
  expect a small conflict there when rebasing (keep both). `egl14-wayland` touches neither.
- **Rebase each branch onto the latest `upstream/trunk` before opening it**, then rerun its checks.

## The v30 fork and these branches

The fork's TV build uses wgpu 30 (`v30-tizen`, which starts at `v30.0.1` and holds the fork's other v30 fixes). The
two changes were made there first and ported to `trunk` by cherry-picking, then adapted to what `trunk` changed
since v30. Each branch README lists the adaptations.

| Change | v30 branch (merged into `v30-tizen` `4e3d80d5d`) | v30 commits | `trunk` branch | Its commits |
|---|---|---|---|---|
| EGL 1.4 on Wayland | `v30-egl14` | `66e825217` | `egl14-wayland` | `01f93a569` |
| Program binaries | `v30-gles-program-binaries` | `fe2a46eac`, `de226d2fa` | `gles-program-binaries` | `2bc8346d8`, `03cab4b4b` |

`v30-tizen` is never sent upstream. When an upstream PR changes under review, port the change back onto its `v30-*`
branch and merge that into `v30-tizen`, so the TV build keeps what upstream gets. Once a wgpu release holds both and
gpui-fast moves to it, the fork's patch goes away.

## Rules for this area

- Each branch is cut from `upstream/trunk` and never stacked on the other; it holds only its topic's commits.
- Nothing is pushed to `origin` or upstream, and no PR is opened, without the owner's go. The owner runs the
  `git push` and `gh` commands in each branch's README.
- `../wgpu` stays on `v30-tizen`: it is the path patch other builds use. Work on these branches in a separate
  worktree, `git -C /Users/dinko/work-private/projects/dXn/wgpu worktree add ../wgpu-wt/trunk <branch>`, and remove it
  afterwards (`git worktree remove ../wgpu-wt/trunk`); the branches stay.
- Never edit `.github/**` in wgpu.
- Follow wgpu's `CONTRIBUTING.md` and `AGENTS.md`: a `CHANGELOG.md` entry under `## Unreleased` for user-visible
  changes, `cargo fmt`, `cargo clippy ... -- -D warnings`, tests for new behavior, commit titles `[area] ...`, no
  comments beyond what the code needs.
- **wgpu's policy on LLM use.** `AGENTS.md` says fully agentic PRs are not allowed and the contributor must
  understand and vouch for every change; the PR template asks "LLM Use?" and that significant descriptions be
  written by hand. The `PR.md` texts here are drafts: the owner rewrites the description in their own words, fills in
  the "LLM Use?" section themselves and decides the template's disclosure items before opening.
- No AI attribution in commits, PR texts or comments.

## Checks

wgpu's CI (`.github/workflows/ci.yml`, read-only) runs clippy and `cargo xtask test` across platforms; on Linux the
GL tests run on Mesa llvmpipe. What ran for these branches (2026-11-10), in the `gpuitizen` Lima VM (Ubuntu 24.04.5,
aarch64, Mesa 25.2.8 llvmpipe, Rust 1.96.1 from wgpu's `rust-toolchain.toml`), from the worktree
`/Users/dinko/work-private/projects/dXn/wgpu-wt/trunk`, through this repository's `scripts/lima-run.sh` (outside
this repository it sets `WGPU_BACKEND=gl` and plain Mesa, no GL ES 3.0 override, target dir
`$HOME/target-gpuitizen-trunk` in the VM). Logs are under `artifacts-temp/agentx/tizen-gpui-platform/t22-wgpu-*.log`
(ignored). Each branch README has the commands and counts.

Not run: the full `cargo xtask test`, Windows, macOS, WebGL/wasm builds, `typos`, the CTS.
