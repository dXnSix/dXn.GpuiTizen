# external-target

| | |
|---|---|
| Change | 5, second half: drawing into a target the caller supplies, with partial redraws |
| Upstream PR | ready, not opened: nothing is opened at `longbridge/gpui-fast` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)); after `scene-damage` has merged |
| Branch | `external-target` on `dXnSix/gpui-fast`, head `ee9632b09`, pushed 2026-11-12 (rebased from `5ee78d5`) |
| Base | `scene-damage` (`7cccca9`), on `upstream/main` `d1d6286` (#139, fetched 2026-11-12); rebased onto it without conflicts in its own files |
| Size | 11 files, +2089 −5, of which `scene-damage`'s `fast/damage.rs` and `fast/damage/tests.rs` are 905 lines. Its own part: 9 files, +1184 −5: `fast/external_target.rs` (+600), `fast/external_target/tests.rs` (+527), `fast/frame.rs` (+23 −2, the scissor and its comment), `fast/layers/raster.rs` (+1), `fast/mod.rs` (+3), `gpui_wgpu.rs` (+2, the export), `wgpu_context.rs` (+5 −2, two visibility bumps), `wgpu_renderer.rs` (+2 −1, one visibility bump), `docs/upstream-sync.md` (+21) |
| Text | [PR.md](PR.md) |
| On `tizen` | `905aad7`, `3868577`, `9099eaa` (with `scene-damage`'s `8e546fd`), `9d9b1f2`, `e6ae34b`, `7047b05` (`5ee78d5`), and `e4960b0` (only for the 0.1.13 base); `fast/external_target*` and `fast/damage.rs` identical |
| Needs | `scene-damage` merged; the maintainers' answer on the API exception (issue, question 2); the issue ([../issue.md](../issue.md)) |

## Its commits

Oldest first, on `scene-damage` (rebased 2026-11-12: the old head carried its own copies of two `scene-damage`
commits; those went, the renderer hooks the first copy added never come, and `damage.rs`'s `allow(dead_code)` goes
in the commit that starts reading the tracker):

| Commit | Subject | Whose |
|---|---|---|
| `21414f8` | gpui_wgpu: compute scene damage between frames | `scene-damage` |
| `1858415` | gpui_wgpu: bound damage merging and key layer tiles and clip groups by what draws them | `scene-damage` |
| `7cccca9` | gpui: Prove the primitives partial redraws hash carry no padding | `scene-damage` |
| `8f97ead` | gpui_wgpu: draw into a target the caller supplies, with partial redraws | this branch |
| `707976f` | gpui_wgpu: move the headless context's parts into the external target test | this branch |
| `c24f70b` | gpui_wgpu: clear partial rectangles, hand damage over before drawing, recover from failed frames | this branch |
| `f3ebe04` | gpui_wgpu: free the external target's target-sized textures on release | this branch |
| `79e3264` | gpui_wgpu: give the external target's context to dynamic textures | this branch |
| `ee9632b` | gpui_wgpu: Put begin_main_pass's comment on it, count the exceptions right | this branch |

`git diff scene-damage external-target` is this branch's own change; every file it touches is byte-for-byte what the
old head `5ee78d5` had.

## What it adds

`WgpuExternalRenderer` (`fast::external_target`, not built for wasm) builds a renderer on a wgpu instance, adapter,
device and queue the caller made (for example over a GL context the platform owns, through
`wgpu_hal::gles::Adapter::new_external`) and draws a scene into a texture view the caller supplies for each frame. No
`wgpu::Surface` is made or presented.

- `new(instance, adapter, device, queue, format)` picks the atlas format from the adapter and dual-source blending
  from the device's features, builds a `WgpuContext` (`from_native_adapter_and_device`, now `pub(crate)`) and a
  `WgpuRendererCore`, and installs the device's lost and uncaptured-error callbacks.
- `draw(scene, target, size, flip_y, age, before_drawing) -> Result<WgpuFrameDamage>`: the damage of the frame comes
  from `fast::damage`, kept for the last 4 frames that had any. With `age` (`EGL_EXT_buffer_age`; 0 = unknown) the
  renderer loads the target, clears the bounding rectangle of the last `age` frames' damage and draws only inside it
  (one scissor rectangle per pass, so the bounding box); age 0, an older target, or a frame damaged whole draws
  everything; a frame without damage draws nothing. `before_drawing` gets the damage before any GPU work (for
  `eglSetDamageRegionKHR`), and the same is returned (for `eglSwapBuffersWithDamageKHR`), in the target's coordinates.
- With `flip_y` (a GL default framebuffer is bottom-up), the frame is drawn into a texture the renderer keeps, which
  always holds the previous frame, and the redrawn region is copied into the target upside down by a scissored pass.
  GPUI's shaders do not change. That texture is one more target-sized texture (about 8 MB at 1920x1080).
- A GPU error since the last frame or a lost device fails the next `draw`; any failure makes the following frame whole.
- `sprite_atlas()`, `context()`, `invalidate()`, `release_targets()` (also frees the path intermediates, through
  `WgpuResources::invalidate_intermediate_textures`, now `pub(crate)`).
- `WgpuFrameDamage { damage, redraw }`, both `Vec<Bounds<DevicePixels>>`, public fields.

`fast/frame.rs` gets a `partial: Option<Bounds<DevicePixels>>` in `FrameState`: a partial frame keeps the texture's
contents (no clear) and sets the scissor on the main passes. `WgpuRenderer`'s path does not change. The fork's
`docs/upstream-sync.md` lists the exception under "Where our API differs from upstream's", as the third public API
exception after `fast::dynamic_textures` and `WebPlatform::wgpu_context()` (`5ee78d5`).

### Before opening

- The commit subjects say `gpui_wgpu:`; the PR title uses `fast:`.
- `video-hole` also calls its exception the third (both are cut from `main`, where each is right on its own branch);
  whichever merges second becomes the fourth, as `tizen`'s `docs/upstream-sync.md` already says for the hole.

(`5ee78d5` fixed the two earlier items: `begin_main_pass`'s doc comment is back on it, `set_scissor` has its own, and
`docs/upstream-sync.md` counts the exception as the third, after `fast::dynamic_textures` and
`WebPlatform::wgpu_context()` (#102).)

## Rebasing once `scene-damage` has merged

The branch must hold only its own commits. Either drop the two `scene-damage` commits in an interactive rebase (the
owner's terminal; expect a conflict in `fast/frame.rs`, where `f6cab52`'s tracker field is gone on `main`, resolved to
this branch's version), or rebuild it from its own diff, which keeps the result exact:

```bash
# owner runs this, after scene-damage has merged upstream
cd /Users/dinko/work-private/projects/dXn/gpui-fast-wt/pr
git fetch upstream origin
git switch -c external-target-rebased upstream/main
git diff scene-damage external-target | git apply --3way --index
git commit    # subject: fast: draw into a target the caller supplies, with partial redraws; body from PR.md
git diff upstream/main -- crates/gpui_wgpu/src/fast/damage.rs   # only the allow(dead_code) block removed
git branch -M external-target-rebased external-target
```

If `scene-damage` changed in review, update `external-target` with the same change first (or the diff above carries
the old version back). **It has already:** `scene-damage`'s `6b15d78` (`gpui::fast::scene_padding`, the padding test)
is not on `external-target`, so `git diff scene-damage external-target` now also removes
`crates/gpui/src/fast/scene_padding.rs` and its `mod` line. Before applying the diff, cherry-pick `6b15d78` onto
`external-target` (it touches no file of this branch's own), or apply the diff with
`-- ':!crates/gpui/src/fast/scene_padding.rs' ':!crates/gpui/src/fast/mod.rs'`, and check that
`git diff upstream/main` shows no change under `crates/gpui/`.

## Checks

To run on the branch before opening (the GPU tests need an adapter: Metal on macOS, lavapipe or a GPU on Linux):

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::external_target::tests   # 13 tests, GPU
cargo test -p gpui_wgpu --features test-support --lib fast::damage::tests            # 13 tests
cargo test -p gpui_wgpu --features test-support --lib fast::dynamic_textures         # what upstream CI runs on macOS
script/check-upstream
cargo fmt -p gpui_wgpu -- --check
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
cargo +nightly check -p gpui_web --lib --target wasm32-unknown-unknown                    # as release.yml does; the module is left out on the web
```

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS Metal; nothing recorded on this branch
itself):

- 2026-11-09 09:58 UTC, after `79b5148`: `fast::external_target::tests` passed (11 tests), `fast::damage::tests` (13),
  `script/check-upstream`, the `gpui_wgpu` clippy.
- `5ee78d5` (2026-11-10) changes a doc comment and `docs/upstream-sync.md` only; `tizen` carries it as `7047b05`,
  and `tizen-release` was regenerated from that (`53659f0`).
- `0d26380` (10:37 UTC) and `987917f` (11:08 UTC) came after and add the two remaining tests
  (`released_targets_are_freed_and_the_next_frame_is_whole`, `dynamic_textures_are_made_from_the_renderers_context`).
  With their copies on `tizen`, `script/check-upstream` passed at 11:19 UTC and the release workspace checked at
  11:58 UTC; the test group was not run again, so the 13-test run has no record.
- dXn.GpuiTizen's EGL platform draws through this renderer; its TV libraries linked for armv7 and x86_64 against the
  release workspace on 2026-11-09. Frames on a TV are the owner's `DEV-3` measurements.
- Not run: the upstream CI steps, the wasm check.


### Run on the rebased tip (2026-11-12)

On `ee9632b09`, macOS (`../gpui-fast-wt/pr`): `fast::external_target::tests` 13 passed (macOS Metal), `fast::damage::tests` 13, `fast::dynamic_textures` 1, `script/check-upstream`, `cargo fmt`, the `gpui_wgpu` clippy, and the web check on nightly (`cargo +nightly check -p gpui_web --lib --target wasm32-unknown-unknown`, as upstream's `release.yml` does). Logs: `artifacts-temp/agentx/tizen-gpui-platform/t25-gf/external-target.log`.

## Opening

After rebasing as above, running the checks, ticking the checklist in `PR.md`, replacing `#(issue)` and
`#(scene-damage PR)`, and filling the `[OWNER: DEV-3: …]` numbers. The owner runs this:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast-wt/pr
git switch external-target
git push --force-with-lease -u origin external-target
gh pr create --repo longbridge/gpui-fast --base main --head dXnSix:external-target \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/external-target/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/external-target/PR.md)
```

Then record the PR number here and in [../README.md](../README.md).
