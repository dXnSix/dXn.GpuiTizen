# scene-damage

| | |
|---|---|
| Change | 5, first half: the damage `external-target` draws partial frames with |
| Upstream PR | not opened |
| Branch | `scene-damage`, local in `/Users/dinko/work-private/projects/dXn/gpui-fast`, not pushed |
| Base | `upstream/main` `a427a7d` (2026-11-09, #123), the fetched `upstream/main`; 4 commits behind live `main` `c2bec823e` (2026-11-10: #129, #132, #131, #130). Only #129 and #132 touch a file of this branch, `crates/gpui/src/fast/mod.rs` (`mod list_measure;`, `mod truncation;`), in other hunks than its `mod scene_padding;` (compared through the GitHub API; live `main` is not fetched, so `git merge-tree` against it has not run). Merges onto `a427a7d` without conflict. Not rebased |
| Size | 5 files, +1050 −0: `crates/gpui_wgpu/src/fast/damage.rs` (+533), `fast/damage/tests.rs` (+380), `fast/mod.rs` (+1); `crates/gpui/src/fast/scene_padding.rs` (+135, tests only), `crates/gpui/src/fast/mod.rs` (+1). No upstream file |
| Its commits | `c60d3bd` gpui_wgpu: compute scene damage between frames; `8e546fd` gpui_wgpu: bound damage merging and key layer tiles and clip groups by what draws them; `6b15d78` gpui: Prove the primitives partial redraws hash carry no padding |
| Text | [PR.md](PR.md) |
| On `tizen` | `3a04ba6`, `9099eaa` (with `external-target`'s `79b5148`), `d87f521`, `e392162` (`6b15d78`); `fast/damage*` identical to `external-target`'s, `fast/scene_padding.rs` identical to this branch's |
| On `external-target` | `f6cab52` (same tree as `c60d3bd`) and `7567428` (same change as `8e546fd`); not `6b15d78` (see [../external-target/README.md](../external-target/README.md#rebasing-once-scene-damage-has-merged)) |
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

`6b15d78` proves the bytes hashed carry no padding. `hash_bytes` reads each primitive's bytes
(`std::slice::from_raw_parts` over the `#[repr(C)]` values the renderer uploads), and reading a padding byte would be
an uninitialized read. The test group `fast::scene_padding` in the `gpui` crate (where the types and their private
fields are) checks, for every type whose bytes are hashed, that its size is the sum of its fields' sizes: the geometry
(`Point`, `Size`, `Bounds`, `Corners`, `Edges`, `ContentMask`, `TransformationMatrix`), colors and tiles (`Hsla`,
`LinearColorStop`, `Background` with its private `pad`, `AtlasTextureId`, `AtlasTile`) and the primitives (`Shadow`,
`Quad`, `Underline`, `MonochromeSprite`, `SubpixelSprite`, `PolychromeSprite`). A field added with a gap fails there
instead of quietly hashing padding.

Upstream PR #45 (huacnlee, "Scene damage and adaptive CPU rendering", closed 2026-10-08 unmerged) computed exact damage
in `crates/gpui/src/fast/damage.rs` with `Scene::damage`. This one stays inside `gpui_wgpu` (but for the test module
`gpui::fast::scene_padding`), is conservative instead of exact, and adds nothing to `Scene`.

### Before opening

- The PR title uses `fast:`; the commit subjects say `gpui_wgpu:` and `gpui:`. Reword them to `fast:` if the commits
  stay separate.
- `hash_bytes`' `SAFETY:` comment still says padding "only makes equal primitives look different"; with `6b15d78` it
  can say there is none and name `fast::scene_padding` (optional, a comment only).

## Checks

To run on the branch before opening:

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests   # 13 tests, no GPU
cargo test -p gpui --features test-support --lib fast::scene_padding        # 3 tests
script/check-upstream
cargo fmt -p gpui_wgpu -- --check
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
cargo fmt -p gpui -- --check && cargo clippy -p gpui --all-targets --features test-support -- -D warnings   # as upstream CI, for 6b15d78
```

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS; nothing recorded on this branch
itself):

- 2026-11-09 09:58 UTC, after both commits: `fast::damage::tests` passed (13 tests); `script/check-upstream` and the
  `gpui_wgpu` clippy passed; the release workspace regenerated from `tizen` checked.
- 2026-11-10, `6b15d78`: `fast::scene_padding` passed (3 tests: `geometry_has_no_padding`,
  `colors_and_tiles_have_no_padding`, `the_hashed_primitives_have_no_padding`); `tizen` carries it as `e392162`, and
  `tizen-release` was regenerated from it (`53659f0`, from `tizen` `7047b05`). Run by the coordinator on 2026-11-10,
  `cargo test -p gpui --lib fast::scene_padding` in `../gpui-fast`, on `scene-damage` and on `tizen`: 3 passed each.
  No log file and no epic check records it.
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
