# gpui-fast: Samsung TVs on Tizen

Renderer changes to [gpui-fast](https://github.com/longbridge/gpui-fast) that let GPUI draw on Samsung TVs running
Tizen 8.0 and later. dXn.GpuiTizen (this repository, the Tizen platform for GPUI) builds on them; they are proposed
upstream so that it can use a released gpui-fast instead of the fork. All of them live in `crates/gpui_wgpu` (the hole
also in `crates/gpui`, and the damage's padding test), in `src/fast/` with one-line hooks in upstream files, as gpui-fast's
`docs/upstream-sync.md` asks. Four add no public API; two (the external target and the hole) are new API exceptions
and say why.

One issue comes first ([issue.md](issue.md)), then one PR per branch. Each folder holds the branch's `README.md`
(status, base, commits, checks, how to update and open it) and `PR.md` (the title and description to paste).

## Which change is which

The research ([gpui-fast-changes.md](/Users/dinko/work-private/projects/dXn/nsh/nsh-root/_projects/tizen/gpui-fast-changes.md),
read-only) numbers the changes 1 to 5; the epic's design `d23-damage-external-target` splits change 5 in two, and the
hole (`VID-1`) came after it.

| Research | Branch | In gpui-fast? |
|---|---|---|
| 1. GL ES 3.0 devices | `gles-instance-path` | yes |
| 2. GL on request (`WGPU_BACKEND`) | `wgpu-backend-env` | yes |
| 3. A pipeline cache on disk | `pipeline-cache` | yes |
| 4. No `gpui_linux` in the TV build | none | no: made in the NSH fork (`nsh-platform-gpui`'s `gpui_platform` behind a feature) |
| 5. Drawing into a supplied target | `scene-damage` (the damage) and `external-target` (the renderer) | yes |
| The hole (video under the window) | `video-hole` | yes |

## The PRs, in order

All on the fork `dXnSix/gpui-fast` (checkout `/Users/dinko/work-private/projects/dXn/gpui-fast`), local only: `origin`
has only `main` as of the last fetch. Sizes are `git diff --stat $(git merge-base upstream/main <branch>) <branch>`.

| # | Folder | Branch | Base | Title | Size | New API | Needs | Status |
|---|---|---|---|---|---|---|---|---|
| 01 | [gles-instance-path/](gles-instance-path/) | `gles-instance-path` | `fb8cf94` | fast: draw GL ES 3.0 devices through the instance-texture path | 3 files, +215 −1 | no | the issue | not opened |
| 02 | [wgpu-backend-env/](wgpu-backend-env/) | `wgpu-backend-env` | `fb8cf94` | fast: let WGPU_BACKEND choose the native backends | 3 files, +66 −1 | no | the issue; a rebase if 01 merges first (both create `fast/gles.rs`) | not opened |
| 03 | [pipeline-cache/](pipeline-cache/) | `pipeline-cache` | `fb8cf94` | fast: keep compiled pipelines in a cache on disk | 4 files, +279 −1 | no | the issue | not opened |
| 04 | [scene-damage/](scene-damage/) | `scene-damage` | `a427a7d` | fast: compute scene damage between frames | 5 files, +1050 −0 | no | the issue (unused until 05) | not opened |
| 05 | [external-target/](external-target/) | `external-target` | `a427a7d` | fast: draw into a target the caller supplies, with partial redraws | 11 files, +2089 −5 (own part: 9 files, +1184 −5) | yes: `WgpuExternalRenderer`, `WgpuFrameDamage` | 04 merged; the maintainers' answer on the API | not opened |
| 06 | [video-hole/](video-hole/) | `video-hole` | `a427a7d` | fast: paint holes that replace the window's pixels with transparent black | 10 files, +477 −1 | yes: `Window::paint_hole`, `Scene::has_holes`, `AtlasTextureId::hole` / `is_hole` | the maintainers' answer on the API | not opened |

Bases: `fb8cf94` is "fast: count tab stops only when the accessibility tree is dumped (#118)", 2026-11-08; `a427a7d`
is "chore: Sync GPUI upstream d2a4fb6ded (#123)", 2026-11-09, the fork's `upstream/main` as last fetched.

### Are the bases current? (2026-11-10, 19:30 CET)

- `upstream/main` in the checkout is `a427a7d` (not fetched for this refresh). The live `longbridge/gpui-fast` `main`
  is `c2bec823e` (`git ls-remote`), 4 commits on (GitHub's compare API): #129 "fast: measure a measure_all list over
  later frames after a width change" (`820088284`, 2026-11-09), #132 "fast: Fit truncated text to its shaped width",
  #131 "fast: give Windows a dynamic texture backend", #130 "composition: keep GPUI-started mouse drags on GPUI" (all
  2026-11-10). They touch `crates/gpui` (`elements/list.rs`, `fast/list_measure.rs`, `fast/truncation.rs`, `fast/text.rs`,
  `text_system/line_wrapper.rs`, the two `mod` lists in `fast/mod.rs` and `fast/tests/mod.rs`), `gpui_apple`,
  `gpui_macos`, `gpui_windows` and `script/upstream-allowlist`; none touches `crates/gpui_wgpu` or
  `docs/upstream-sync.md`.
- 01–03 are 7 commits behind `a427a7d` (11 behind live `main`). None of those commits touches `crates/gpui_wgpu`, and
  each branch merges onto `a427a7d` without conflict (`git merge-tree`).
- 04–06 are cut from `a427a7d`, 4 commits behind live `main`. #129 and #132 add `mod list_measure;` and
  `mod truncation;` to the two `mod` lists where 06 adds `mod hole;` and 04 (since `6b15d78`) `mod scene_padding;`, in
  other hunks (lines apart, compared through the API). `git merge-tree` against live `main` has not run (not fetched).
- Nothing was rebased. Rebase each one onto the fresh `upstream/main` when it is opened.

### Between the branches

Checked with `git merge-tree` (no branch touched):

- 01 + 02 conflict in `fast/gles.rs`: both create it. The second to merge adds its part to the file; on `tizen` the
  file holds both, with the test groups `fast::gles::tests::device` (01) and `fast::gles::tests::backends` (02).
- 04 + 05 conflict in `fast/damage.rs`: 05 carries 04's first two commits and drops 04's `allow(dead_code)`. 05 does
  not carry 04's third, `6b15d78` (the padding test, `gpui::fast::scene_padding`). 05 is opened after 04 merges,
  rebased so it holds only its own commits ([external-target/README.md](external-target/README.md), which says how to
  keep `6b15d78` out of the diff it applies).
- 05 and 06 each call their API exception "the third" in `docs/upstream-sync.md`; on its own branch (both cut from
  `main`) each is right. The second to merge becomes the fourth.
- 03 + 06 merge cleanly, but 06 creates the hole pipeline with `cache: None`. Whichever lands second passes the
  pipeline cache to it, as `tizen` does (`Option::as_ref(&fast_pipeline_cache)`).
- Every other pair merges cleanly.

## The issue

[issue.md](issue.md): the TV use, changes 1–3, 5 and the hole, what each touches, why two need API, the TV numbers
(placeholders until the owner's `DEV-3` measurements), and whether the maintainers want this in gpui-fast or in Zed's
GPUI.

**Issues are closed on `longbridge/gpui-fast`.**
`gh repo view longbridge/gpui-fast --json hasIssuesEnabled,hasDiscussionsEnabled` gives `false` for both
(2026-11-10), and `CONTRIBUTING.md` says issues are not open for now. The text cannot be opened as an issue as it
stands. How to post it is the owner's call: ask a maintainer where they want it, or open it as the
description of the first PR (01). The PR texts say `Part of #(issue)`; replace that with the link to wherever the
text is posted. If issues open again:

```bash
# owner runs this
gh issue create --repo longbridge/gpui-fast \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/issue.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/issue.md)
```

`CONTRIBUTING.md` also says PRs are welcome for Retained Mode, and for other needs only when they are clearly
reasonable and pressing; everything else belongs in GPUI upstream. None of these changes is Retained Mode work, which
is why the issue asks first.

## Rules for this area

- Each branch is cut from upstream `main` and never stacked on another; it holds only its topic's commits. The one
  exception is `external-target`, which carries `scene-damage`'s two commits because it cannot build without them, and
  is rebased to drop them once `scene-damage` has merged.
- Nothing is pushed to `origin` or upstream, and no issue or PR is opened, without the owner's go. The owner runs the
  `git push` and `gh` commands in each branch's README.
- Before opening: `git fetch upstream origin`, rebase onto `upstream/main`, run the checks in the branch's README,
  update the numbers and the checklist in its `PR.md`.
- PR titles start with the area, as gpui-fast's `CLAUDE.md` asks (`fast:` for gpui-fast's own work, never `gpui:`).
- gpui-fast has no PR template (`.github/` holds only `workflows/`); the texts follow the NSH format: Description, How to
  Test, Checklist.
- GitHub Actions stay off on the fork (its workflows are upstream's and `release.yml` needs secrets the fork lacks).
- No AI attribution in commits, PR texts or comments.

Suggested place to work, so that `tizen` stays checked out in `../gpui-fast`:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast
git worktree add ../gpui-fast-wt/pr <branch>     # once; later: git -C ../gpui-fast-wt/pr switch <branch>
```

## `tizen` and `tizen-release`

- **`tizen`** carries all six changes for the TV build and is never sent upstream. It starts at the tag `v0.1.13`
  (`cd20d12`, 2026-11-02), the release dXn.GpuiTizen and NSH pin (`=0.1.13`), not at `main`, and is rebuilt from the
  topic branches by cherry-picking their commits. Besides them it holds what 0.1.13 lacks and the TV build needs:
  the dynamic textures of #86 and #99 (merged upstream after 0.1.13), `e4960b0` (opens `WgpuRendererCore::new`,
  `render_frame` and `WgpuContext::create_headless`, which `main` already has as `pub(crate)`), and `wgpu-test-lints`
  (`40d90c9`, see Checks). The files of the topic branches are the same on `tizen` (`fast/damage*`,
  `fast/external_target*`, `fast/hole*`, `fast/pipeline_cache.rs`, `gpui`'s `fast/scene_padding.rs`); `fast/gles.rs`
  holds 01 and 02 together. When a topic branch changes, its new commits are cherry-picked onto `tizen`.
- **`tizen-release`** holds only the output of `bun script/prepare-release.ts --tag v0.1.13` run on `tizen` (through
  `cargo xtask gpui-fast-release` in this repository), the `gpui-fast-*` crates with wgpu 30.0.1 from crates.io, never
  edited by hand. TV builds patch all nine `gpui-fast-*` crates from it through `[patch.crates-io]`; local builds use
  `artifacts-temp/gpui-fast-patch.toml`, written by `cargo xtask gpui-fast-release --verify`.
- `tizen` is at `7047b05` (2026-11-10): besides the six changes it now carries `scene-damage`'s `6b15d78` as `e392162`
  and `external-target`'s `5ee78d5` as `7047b05`. The committed `tizen-release` is **current**: `53659f0` (2026-11-10
  19:17 CET), generated from `tizen` at `7047b05` and verified ("gpui-fast patch ok (tizen at 7047b05fdb67, wgpu
  v30-tizen at 4e3d80d5d)"; `artifacts-temp/agentx/tizen-gpui-platform/gpui-fast-release-{commit2,verify2}.log`), which
  rewrote dXn.GpuiTizen's `artifacts-temp/gpui-fast-patch.toml`. The two before it were `d09584c` (from `e6ae34b`)
  and `1515355` (from `3b6ea11`). Neither branch is pushed: `origin` has `main` only.
- When a gpui-fast release has the changes and NSH requires it, the patch lines and both branches go away.

## Checks

gpui-fast's CI (`.github/workflows/ci.yml` on `main`) runs `script/check-upstream`,
`cargo fmt -p gpui -p gpui_perf -- --check`,
`cargo clippy -p gpui -p gpui_perf --all-targets --features gpui/test-support -- -D warnings`,
`cargo test -p gpui --features test-support` on Linux, macOS and Windows, `cargo check -p gpui_perf --all-targets`, and
`gpui_wgpu`'s GPU tests (`fast::dynamic_textures`) on macOS only. It does not lint `gpui_wgpu`.

`cargo clippy -p gpui_wgpu --all-targets` fails on `main` itself with two lints in test files (a manual
`is_multiple_of` in `fast/frame/tests.rs`, `duplicate_mod` in `fast/layers/rounded_clip_texture_tests.rs`), found on
`fb8cf94`; `crates/gpui_wgpu` is unchanged up to `a427a7d`. The fork's branch `wgpu-test-lints` (`40d90c9`, one commit
on `fb8cf94`) fixes them; it is on `tizen` and is not one of these PRs.
On a topic branch alone, lint the library target (`--lib`) and expect those two in `--all-targets`.

Every branch README lists its own commands. What has run so far was recorded by this repository's epic checks
(`.agentx/tasks/tizen-gpui-platform`, logs under `artifacts-temp/agentx/`), in `../gpui-fast` with `tizen` checked out,
on macOS (Metal for the GPU tests). Nothing is recorded as run on a topic branch itself. On Linux, the examples'
`gles_smoke` tests (`scripts/lima-run.sh`: Mesa llvmpipe, `WGPU_BACKEND=gl`,
`MESA_EXTENSION_OVERRIDE=-GL_ARB_shader_storage_buffer_object`) draw through change 1's path, built against the release
workspace generated from `tizen`.
