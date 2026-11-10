# gpui: Play videos on Samsung TVs through the TV's player

Part of #(Tizen issue). **Depends on:** #(pr/tizen/04-platform), merged. No nsh-compat change.

## Description

On Tizen a `Video` drew the placeholder (#(pr/tizen/03-gpui-tizen-cfg)). It now plays through dXn.GpuiTizen's
`VideoPlayer` (Tizen's `capi-media-player`, a `cfg(target_vendor = "tizen")` dependency), in
`native/video/tizen.rs`:

- **Output.** The TV's video plane under the window, seen through a hole the view paints, while the view is drawn as
  laid out (no rotation or skew, opaque, unfiltered, not cut by a clip); otherwise a texture of its frames drawn as
  content. It is chosen where the video is first drawn; a video on the plane whose view comes to need the texture
  opens again as one, where it was (never back).
- **Props** become the player's calls (`paused`, `muted`, `volume`, `repeat`; `rate` stays 1: Tizen 8.0 has no public
  API for it), `seek` seeks, and the player's notices become NSH's events: `onLoadStart`, `onLoad` with the duration,
  `onProgress`, `onBuffer`, `onEnd`, `onError`.
- **Seeks** wait for the player's `Ready` (a seek before it, and playing behind it, are kept), so a video reopened as a
  texture resumes where it was.
- **Interruptions.** When the system takes the decoder (another app, typically around a pause and resume) and the
  video is lost, the app hears `onBuffer(true)`; once the app is active again the player is prepared again, given
  `muted`, `volume` and `repeat` again, sought to where it was and played as `paused` says, then `onBuffer(false)`
  (`onError` when it cannot be prepared).
- `controls` draws NSH's own control bar, now shared with GStreamer's player (`native/video.rs`).

`nsh-platform-gpui` depends on `dxn_gpui_tizen` on Tizen only, so a release that publishes it needs dXn.GpuiTizen on
crates.io first (`docs/dev/tizen.md`, "Building").

## Public API

None. `Video` gains a backend on Tizen.

## How to Test

- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib native::video::tizen`, on any desktop
  (the logic is host-tested): the output choice, the props' calls, the notices' events and their payloads,
  `seek_before_ready`, `interruption`.
- On a Samsung TV or the Tizen emulator: [OWNER: the video example on a TV, plane and texture, a pause and resume].

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): nothing apps use changes.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (Tizen only: on a TV instead).
