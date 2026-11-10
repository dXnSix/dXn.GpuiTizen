# gpui: Build for Tizen

Part of #(Tizen issue). **Depends on:** #(pr/tizen/02-gpui-platform-optional), merged. No nsh-compat change.

## Description

A Samsung TV's Tizen is `target_os = "linux"` with `target_vendor = "tizen"` (custom target specs). The TV host is a
GPUI app on a GPUI platform of its own, so `nsh-platform-gpui` is built for it without default features
(`--no-default-features --features js`: no `gpui_platform`, no wry). Under `cfg(target_vendor = "tizen")` the Linux
desktop's code is left out, and so are its dependencies (zbus, x11rb, raw-window-handle, GTK, WebKitGTK, wry,
GStreamer: the manifest's target tables exclude the vendor), whatever the features:

- **Settings**: no settings portal or AT-SPI over D-Bus. The environment starts from the process's POSIX locale and
  time zone and the defaults (text size, motion on, no screen reader, OS version 0.0: the kernel's is not the TV's);
  the TV host passes the TV's settings on through `AppView`'s setters as they change (`host_settings.rs`, written over
  the system's as `HostEnvironment` is). The new `set_reduce_motion`, `set_screen_reader` and `set_time_zone` join
  `set_locale` and `set_font_scale`, on every platform.
- **Fonts** (`text/tv.rs`): the UI font is the first installed of Samsung's TV family and Tizen's (`SamsungOneUI`,
  `BreezeSans`, `TizenSans`) that GPUI can set in bold, names matched ignoring case and spaces (for the preferred
  family's line metrics too); the fallbacks are every installed family of those and of their scripts
  (`BreezeSansKorean`, …), as the set differs between TVs. The Linux desktop's families are not tried.
- **Web views** report that they are unavailable (as builds without `webview` now do too: #1439's
  `Environment::web_views`). **Videos** draw the placeholder where the TV's player goes (#(pr/tizen/05-video)).
  **Screenshots** report that they are unsupported.
- A Tizen build that keeps the default feature `platform` stops at a `compile_error!` naming the features to use
  (`gpui_platform` would bring gpui-fast-linux, X11, xkbcommon, fontconfig and FreeType into the TV's graph).

The GPUI workspace declares `target_vendor = "tizen"` to `unexpected_cfgs`. `docs/dev/gpui.md` ("Tizen") and
`docs/dev/embedding.md` describe it; the website's GPUI page (English and zh-CN) names the setters.

## Public API

- `nsh_platform_gpui::AppView::set_reduce_motion(&self, Option<bool>, cx)`, `set_screen_reader(&self, Option<bool>,
  cx)`, `set_time_zone(&self, Option<TimeZone>, cx)`: a host's values over the system's (`None` follows the system
  again), on every platform.

## How to Test

- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib host_settings::tests system::tv::tests text::tv::tests`
  (on any desktop: the settings over the system's, the TV's starting environment, the font choice and matching).
- With a Tizen target spec, its rootstrap and toolchain (dXn.GpuiTizen's cross driver):
  `cargo xtask cross --target armv7 --manifest-path <nsh>/crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features --features js`,
  and `cargo xtask tizen-deps` for the same, which finds no desktop platform crate in the graph.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): nothing apps use changes.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (GPUI only: not applicable).
