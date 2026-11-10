# [gles] Support EGL 1.4 drivers on Wayland

## Connections

Fixes #10708.

## Description

On an EGL 1.4 driver on Wayland (the Mali-G76 of #10708; Samsung's Tizen TVs are the same case), the GLES backend
finds the adapter but cannot present to a window, for the three reasons the issue lists:

1. The Wayland arm of `Instance::init` needs `upcast::<EGL1_5>()`, which needs the `eglGetPlatformDisplay` symbol.
   EGL 1.4 does not export it, so init falls back to `eglGetDisplay(EGL_DEFAULT_DISPLAY)` with `WindowKind::Unknown`,
   and `Surface::configure` refuses the Wayland window ("incompatible window kind").
2. Every config has `EGL_WINDOW_BIT` but `EGL_NATIVE_RENDERABLE = 0`. On desktop Linux only the "native-render" tier
   counts as presentable, so `surface_capabilities` returns `None`.
3. `configure` asks for `EGL_RENDER_BUFFER = EGL_SINGLE_BUFFER`, which some EGL 1.4 drivers refuse in
   `eglCreateWindowSurface`.

This PR fixes each one. The choice is made in a small pure function, with unit tests:

- `wayland_display_path`: EGL 1.5's `eglGetPlatformDisplay` when it exists (as before). Otherwise
  `eglGetPlatformDisplayEXT` when the client extensions have `EGL_EXT_platform_base` and `EGL_EXT_platform_wayland`
  (matched as whole names). Otherwise `eglGetDisplay` on the `wl_display`. All three give `WindowKind::Wayland`. With
  EGL 1.5 and no `EGL_EXT_platform_wayland`, the other arms decide, as before.
- `presentable_tier`: on Wayland the "presentation" tier (`EGL_WINDOW_BIT`) is enough, as it already is on Android,
  Windows and OHOS. The window's buffers come from `wl_egl_window`, so native renderability says nothing about
  presenting. X11 and unknown windows still require the "native-render" tier. `choose_config`'s tier loop becomes
  `choose_tier`, which keeps #10048's single `SURFACE_TYPE` attribute.
- `render_buffer_retry`: when `EGL_SINGLE_BUFFER` is refused with `EGL_BAD_ATTRIBUTE` or `EGL_BAD_MATCH`, the window
  surface is created once more with `EGL_BACK_BUFFER`, and an info line is logged. Other errors are returned as
  before.

Not changed: the surfaceless arm still requires EGL 1.5 (the `TODO` the issue quotes). That arm only runs without a
window system.

## Testing

- New unit tests in `wgpu-hal/src/gles/egl.rs` (`gles::egl::tests`):
  - `display_path`: the path for EGL 1.5 and 1.4 with and without the extensions, and whole-name matching.
  - `config_tiers`: `choose_tier` with fake configs (Mesa-like, the issue's Mali-like, pbuffer-only, no alpha). Each
    attribute list has one `SURFACE_TYPE`. Wayland presents from the presentation tier and X11 does not.
  - `render_buffer`: which errors lead to the retry, and that it happens only once.
- `cargo test -p wgpu-hal --features gles --lib` on Linux (aarch64, Mesa 25.2.8): 6 passed.
- `cargo clippy -p wgpu-hal --features gles --all-targets -- -D warnings` and `cargo fmt --check -p wgpu-hal`.
- Not run on an EGL 1.4 driver from this branch. Mesa is EGL 1.5, so on Mesa the Wayland path is the one used
  before (`eglGetPlatformDisplay`). _(Owner: add the TV or Mali run here if there is one.)_

## Squash or Rebase?

One commit; ready to rebase as it is.

## LLM Use?

_(The owner writes this section.)_

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
- [ ] The description was automatically written by an Agent.
      -->
