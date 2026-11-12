# [gles] Support EGL 1.4 drivers on Wayland

## Connections

Fixes #10708.

## Description

We run wgpu's GLES backend on Samsung TVs (Tizen), which have the same driver situation as #10708: EGL 1.4 on
Wayland. wgpu finds the adapter there but cannot present to the window. The issue lists three causes, and this PR
fixes all three in `wgpu-hal/src/gles/egl.rs`.

1. **Display.** The Wayland path called `eglGetPlatformDisplay`, which only exists in EGL 1.5. On EGL 1.4 the
   instance fell back to `eglGetDisplay(EGL_DEFAULT_DISPLAY)` with an unknown window kind, so
   `Surface::configure` later rejected the Wayland window. Now, when `eglGetPlatformDisplay` is missing, it uses
   `eglGetPlatformDisplayEXT` if the client extensions include `EGL_EXT_platform_base` and `EGL_EXT_platform_wayland`,
   and otherwise `eglGetDisplay` on the `wl_display`. In all three cases the window kind is Wayland. With EGL 1.5
   nothing changes.
2. **Config.** These drivers report `EGL_NATIVE_RENDERABLE = 0` on every config, and on Linux wgpu only counted
   natively renderable configs as presentable. On Wayland the window's buffers come from `wl_egl_window`, so a config
   with `EGL_WINDOW_BIT` is enough, as it already is on Android, Windows and OHOS. X11 keeps the old requirement.
3. **Render buffer.** Some EGL 1.4 drivers refuse `EGL_RENDER_BUFFER = EGL_SINGLE_BUFFER` when creating the window
   surface. If that fails with `EGL_BAD_ATTRIBUTE` or `EGL_BAD_MATCH`, the surface is created once more with
   `EGL_BACK_BUFFER`.

Each decision is a small function (`wayland_display_path`, `presentable_tier` with `choose_tier`, and
`render_buffer_retry`), so it can be unit tested without a driver. `choose_tier` keeps #10048's single
`SURFACE_TYPE` attribute.

Not changed: the surfaceless path still requires EGL 1.5 (the `TODO` quoted in the issue), because it only runs
without a window system.

## Testing

- New unit tests in `gles::egl::tests`: `display_path` (EGL 1.5 and 1.4, with and without the extensions, extension
  names matched whole), `config_tiers` (fake Mesa-like and Mali-like configs; Wayland presents from the
  window-capable tier and X11 does not; each attribute list has one `SURFACE_TYPE`) and `render_buffer` (which errors
  retry, and only once).
- `cargo test -p wgpu-hal --features gles --lib` on aarch64 Linux with Mesa 25.2.8: 9 passed.
- `cargo clippy -p wgpu-hal --features gles --all-targets -- -D warnings` and `cargo fmt --check -p wgpu-hal`.
- Mesa is EGL 1.5, so on Mesa the Wayland path is the same as before. [OWNER: say here whether you ran it on a TV or
another EGL 1.4 driver from this branch; if not, say so plainly.]

## Squash or Rebase?

One commit; ready to rebase as it is.

## LLM Use?

[OWNER: rewrite in your own words and keep only what is true.] I used an AI coding agent (Claude Code) for most of
this PR. It wrote the code and the tests from my direction, first on our v30 fork for the TV build and then ported
to `trunk`, and it drafted this description. I reviewed the diff, checked the three changes against the issue and
the EGL 1.4 and EGL_EXT_platform_wayland specs, and rewrote the description. I understand every change and will
answer review comments myself.

<!--
Thanks for filing! Reviewers are assigned for non-draft PRs in the weekly wgpu maintainers meetings.

After you get a review and have addressed any comments, please explicitly re-request a review from the
person(s) who reviewed your changes. This will make sure it gets re-added to their review queue - you're not bothering us!
-->

## Checklist

<!-- Note that checking all the boxes is not necessary to open a PR. -->

- [ ] I self-reviewed and fully understand this PR.
- [x] `CHANGELOG.md` entries for the user-facing effects of this change are present. <!-- See instructions at the top of `CHANGELOG.md`. -->
- [x] The PR is minimal, and doesn't make sense to land as multiple PRs.
- [x] Commits are logically scoped and individually reviewable.
- [x] The PR description has enough context to understand the motivation and solution implemented.
- [ ] (If applicable) WebGPU implementations built with `wgpu` may be affected behaviorally.
- [ ] (If applicable) Validation and feature gates are in place to confine behavioral changes.
- [x] (If applicable) Tests demonstrate the validation and altered logic works. <!-- See `docs/testing.md` -->

<!-- Agentic agents creating a PR, please add the following to the list and fill it out accurately. Checking this does not automatically invalidate the PR.
- [ ] This PR was automatically created by an Agent.
- [ ] The description was automatically written by an Agent. [OWNER: tick this if you keep the draft mostly as it is]
      -->
