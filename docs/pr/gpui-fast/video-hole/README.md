# video-hole

| | |
|---|---|
| Change | the hole: video on the TV's video plane under the window |
| Upstream PR | not opened |
| Branch | `video-hole`, local in `/Users/dinko/work-private/projects/dXn/gpui-fast`, not pushed |
| Base | `upstream/main` `a427a7d` (2026-11-09, #123), the fetched `upstream/main`; 4 commits behind live `main` `c2bec823e` (2026-11-10: #129, #132, #131, #130). #129 and #132 add `mod list_measure;` and `mod truncation;` to `crates/gpui/src/fast/mod.rs` and `fast/tests/mod.rs`, in other hunks than this branch's `mod hole;` (GitHub API; live `main` not fetched, so `git merge-tree` against it has not run). Merges onto `a427a7d` without conflict. Not rebased |
| Size | 10 files, +477 −1: `crates/gpui/src/fast/hole.rs` (+125), `crates/gpui/src/fast/tests/hole.rs` (+53), two `mod` lines in `gpui`; `crates/gpui_wgpu/src/fast/hole.rs` (+141), `fast/hole/tests.rs` (+126), `fast/frame.rs` (+13, the batch arm), `fast/mod.rs` (+1), `wgpu_renderer.rs` (+5 −1, three hooks); `docs/upstream-sync.md` (+11) |
| Its commits | `60ba887` gpui, gpui_wgpu: paint holes that replace the window's pixels with transparent black |
| Text | [PR.md](PR.md) |
| On `tizen` | `aecc769` (`fast/hole*` identical) |
| Needs | the maintainers' answer on the API exception (issue, question 2); the issue ([../issue.md](../issue.md)). If `pipeline-cache` merges first, the hole pipeline gets the cache ([../pipeline-cache/README.md](../pipeline-cache/README.md)) |

`Window::paint_hole(bounds, corner_radii)` paints a rounded rectangle that replaces the window's pixels with
transparent black instead of blending over them; what is painted after it blends over it as usual. `Scene::has_holes()`
tells the platform whether a frame has one (directly, or inside a layer or rounded clip group it carries), so it makes
its window translucent only then. A hole is a polychrome sprite whose tile has the reserved texture id
`0xDFFF_FFFF` (`AtlasTextureId::hole()`, `is_hole()`), below the dynamic textures' `0xE000_0000`, so the scene, its
batching and its primitive kinds do not change. Bounds are snapped edge by edge to device pixels, the grid a video
plane is positioned on.

The wgpu renderer draws hole batches with their own pipeline: the polychrome sprite vertex shader, a fragment that
outputs the rounded rectangle's coverage as alpha (and honours the rounded clip), and blending
`dst * (1 - src_alpha)` on every channel. Renderers without that arm (Metal, Direct3D) find no such texture and skip
the batch, as they do for a released atlas texture. A hole inside a scroll layer's cached tiles clears those tiles'
pixels; it does not reach the window.

Hooks in `wgpu_renderer.rs`: the `fast_hole` field in `WgpuPipelines`, the shader module's source through
`crate::fast::hole::shader_source` (which wraps `fast::rounded_clip::shader_source`), and the pipeline's creation.

### Before opening

- `docs/upstream-sync.md` puts the exception in rule 5 as "a third exception", after `fast::dynamic_textures` and
  `WebPlatform::wgpu_context()` (#102). That is right on this branch: it is cut from `main`, which has those two.
  `external-target`, also cut from `main`, now calls its own the third as well (`5ee78d5`), right on its branch too.
  Whichever of the two merges second renumbers itself the fourth when it is rebased (on `tizen`, which carries both,
  the hole says "a fourth exception here, the third on `video-hole` for `main`").
- The commit subject says `gpui, gpui_wgpu:`; the PR title uses `fast:`.
- The pipeline is created with `cache: None` (see Needs).

## Checks

To run on the branch before opening (the `gpui_wgpu` tests need a GPU adapter):

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::hole::tests        # 3 tests, GPU
cargo test -p gpui --features test-support --lib fast::tests::hole              # 3 tests, no GPU
cargo test -p gpui --features test-support                                      # upstream CI's test step
script/check-upstream
cargo fmt -p gpui -p gpui_wgpu -- --check
cargo clippy -p gpui --all-targets --features gpui/test-support -- -D warnings  # as upstream CI, for gpui
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
```

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS Metal; nothing recorded on this branch
itself):

- 2026-11-09 11:22 UTC: `gpui_wgpu`'s `fast::hole::tests` passed (3 tests); `script/check-upstream` passed at
  11:19 UTC; the release workspace regenerated from `tizen` checked at 11:58 UTC; the TV libraries with the video
  player linked for armv7 and x86_64 at 11:59 and 12:00 UTC.
- Not run: `gpui`'s `fast::tests::hole`, `cargo test -p gpui`, the `gpui` clippy, the upstream CI steps on Linux and
  Windows.

## Updating and opening

Rebase onto the fresh `upstream/main`, run the checks, tick the checklist in `PR.md`, replace `#(issue)`. The owner
runs this:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast
git fetch upstream origin
git worktree add ../gpui-fast-wt/pr video-hole 2>/dev/null || git -C ../gpui-fast-wt/pr switch video-hole
cd ../gpui-fast-wt/pr
git rebase upstream/main
# the checks above, then
git push --force-with-lease -u origin video-hole
gh pr create --repo longbridge/gpui-fast --base main --head dXnSix:video-hole \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/video-hole/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/video-hole/PR.md)
```

Then record the PR number here and in [../README.md](../README.md).
