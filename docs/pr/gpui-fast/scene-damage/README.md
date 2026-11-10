# scene-damage

| | |
|---|---|
| Change | 5, first half: the damage `external-target` draws partial frames with |
| Upstream PR | not opened |
| Branch | `scene-damage`, local in `/Users/dinko/work-private/projects/dXn/gpui-fast`, not pushed |
| Base | `upstream/main` `a427a7d` (2026-11-09, #123), the fetched `upstream/main`; one commit behind live `main` `820088284` (#129, which touches no file of this branch). Not rebased |
| Size | 3 files, +914 −0: `crates/gpui_wgpu/src/fast/damage.rs` (+533), `fast/damage/tests.rs` (+380), `fast/mod.rs` (+1). No upstream file |
| Its commits | `c60d3bd` gpui_wgpu: compute scene damage between frames; `8e546fd` gpui_wgpu: bound damage merging and key layer tiles and clip groups by what draws them |
| Text | [PR.md](PR.md) |
| On `tizen` | `3a04ba6`, `9099eaa` (with `external-target`'s `79b5148`), `d87f521`; `fast/damage*` identical to `external-target`'s |
| On `external-target` | `f6cab52` (same tree as `c60d3bd`) and `7567428` (same change as `8e546fd`) |
| Needs | the issue ([../issue.md](../issue.md)); no new API. Nothing reads it until `external-target` |

`fast/damage.rs` summarizes a scene as its primitives in paint order (the order of `Scene::batches`), each a hash of
what decides its pixels except its draw order, with the area it can touch (bounds clipped by the content mask, widened
for blur, wavy underlines and antialiasing). Two summaries are diffed (Myers, at most 512 edits); the areas of the
primitives off the common subsequence, clamped to the window, are the damage, merged into at most 8 rectangles (more
than 64 areas become one bounding rectangle first). The whole window is damaged on the first frame, after a resize, an
`invalidate` (scale change, atlas loss, device recovery), past 512 edits, and always with `GPUI_PARTIAL_REDRAW=0`.
Surfaces are damaged every frame, sprites with a non-identity transform damage the whole window, scroll-layer tiles
change with their layer's generation, and rounded clip groups hash their content and geometry frame.

On its own the branch changes nothing: no hook calls it, and a crate-level `allow(dead_code)` (not under `test`)
says so with the reason "read by partial redraws, on the external-target branch". `external-target` removes that
`allow`. If the maintainers prefer one PR (question 3 of the issue), close this one and open `external-target` alone.

Upstream PR #45 (huacnlee, "Scene damage and adaptive CPU rendering", closed 2026-10-08 unmerged) computed exact damage
in `crates/gpui/src/fast/damage.rs` with `Scene::damage`. This one stays inside `gpui_wgpu`, is conservative instead of
exact, and adds nothing to `Scene`.

### Before opening

- The PR title uses `fast:`; the commit subjects say `gpui_wgpu:`. Reword them to `fast:` if the commits stay
  separate.
- `damage.rs` reads primitives as bytes (`std::slice::from_raw_parts` over `#[repr(C)]` values) to hash them. The
  `SAFETY:` comment accepts that compiler padding makes equal primitives look different. Reading padding bytes is
  undefined behaviour in Rust, so check that the hashed types have no implicit padding (they are uploaded to the GPU
  as they are), or hash their fields instead, before a reviewer asks.

## Checks

To run on the branch before opening:

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests   # 13 tests, no GPU
script/check-upstream
cargo fmt -p gpui_wgpu -- --check
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
```

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS; nothing recorded on this branch
itself):

- 2026-11-09 09:58 UTC, after both commits: `fast::damage::tests` passed (13 tests); `script/check-upstream` and the
  `gpui_wgpu` clippy passed; the release workspace regenerated from `tizen` checked.
- Not run: the upstream CI steps.

## Updating and opening

Rebase onto the fresh `upstream/main`, run the checks, tick the checklist in `PR.md`, replace `#(issue)`. The owner runs
this:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast
git fetch upstream origin
git worktree add ../gpui-fast-wt/pr scene-damage 2>/dev/null || git -C ../gpui-fast-wt/pr switch scene-damage
cd ../gpui-fast-wt/pr
git rebase upstream/main
# the checks above, then
git push --force-with-lease -u origin scene-damage
gh pr create --repo longbridge/gpui-fast --base main --head dXnSix:scene-damage \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/scene-damage/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/scene-damage/PR.md)
```

Then record the PR number here and in [../README.md](../README.md), and put it at the top of
[../external-target/PR.md](../external-target/PR.md).
