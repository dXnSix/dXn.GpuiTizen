# pr/tizen/05-video

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tizen/05-video`, head `d15dae74b`, local in `nsh-wt/tizen`, not pushed |
| Base | `pr/tizen/04-platform` at its head (merged in five times: `4fc06489c`, `3a199f3dc`, `e3af1acf0`, `274cd2c3a`, `d15dae74b`), on `upstream/main` `8c733731f`. Against `c16a70096` it has 04's conflicts. Not rebased |
| Depends on | 04 merged |
| Size | own: 9 files, +1706 −131: the new `crates/gpui/nsh-platform-gpui/src/native/video/tizen.rs` (+1496, with its tests), `native/video.rs` (the control bar shared with GStreamer's player), `native.rs`, `nsh-platform-gpui/Cargo.toml` (`dxn_gpui_tizen` for `cfg(target_vendor = "tizen")`), `crates/gpui/Cargo.lock`; `docs/dev/{gpui,tizen}.md`, the website's GPUI page (English and zh-CN). With 04 and below: 111 files, +7055 −424 |
| Its commits | `8cc1d2acd` gpui: Play videos on Samsung TVs through the TV's player; `1b7a46646` gpui: Seek a TV's video once its player is ready, and bring it back after an interruption; `acd5d258f` gpui: Say that Tizen videos need dXn.GpuiTizen on crates.io before a release (merges of 04 between and after them) |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; merged into 07 at its head |
| Review findings | `f-s18-video-seek-before-ready` (major: `seek` right after `VideoPlayer::new` failed with `WrongState` while the player was `Preparing`, so a reopened video lost its position) and `f-s18-video-interruption-unrecovered` (major: an `Interrupted(Idle)` player was never prepared again): `1b7a46646` adds the `Gate` that keeps a seek and playing until `Ready`, and `Gate::resume`, which prepares, restores the props and the position and plays once the app is active again, with the tests `seek_before_ready` and `interruption`. Part of `f-s18-publish-git-dependency`: `acd5d258f` says a release needs dXn.GpuiTizen on crates.io. Passed in `review-s18-2.json` |
| Needs | no nsh-compat change. The video plane's hole is gpui-fast's `video-hole` change ([../../../gpui-fast/video-hole/](../../../gpui-fast/video-hole/)), reached through dXn.GpuiTizen's player |

## Before opening

- It makes `nsh-platform-gpui`, which `release-crate.yml` publishes, depend on `dxn_gpui_tizen` on Tizen. `cargo
  publish` checks target-specific dependencies too, so the first NSH release with this needs dXn.GpuiTizen on
  crates.io (`acd5d258f` documents it). Tell the maintainers in the PR.
- `rate` stays 1: Tizen 8.0 has no public API for the playback rate.

## Checks

| Check | Command | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-video-tizen` | with `$TIZEN_ENV` ([../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)): `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib --config "$PATCH" --config "resolver.lockfile-path='$LOCK'" native::video::tizen` | `t18-video` | passed, 15 tests, 2026-11-10 15:49 UTC (s18) |
| `chk-nsh-tizen-link-*` | 04's three link checks (the video code is in the TV library) | `t18-link` | passed (s18) |

Not run on this branch alone; not run: the pre-open checks, a video on a TV or the emulator (the owner's `t23-video`).

## Updating and opening

After 04 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tizen/05-video pr/tizen/05-video
git rebase --onto upstream/main pr/tizen/04-platform pr/tizen/05-video
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tizen/05-video
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/05-video
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen/05-video \
  --title "gpui: Play videos on Samsung TVs through the TV's player" --body "$(sed '1,2d' PR.md)"
```
