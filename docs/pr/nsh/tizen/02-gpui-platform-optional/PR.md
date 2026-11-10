# gpui: Make gpui_platform an optional default feature

Part of #(Tizen issue). No nsh-compat change.

## Description

`nsh-platform-gpui` always takes GPUI's platform crate (`gpui-fast-platform`, and with it `gpui-fast-linux`,
font-kit, fontconfig, FreeType and Wayland on Linux), even where the host brings a GPUI platform of its own. A
Samsung TV's host does: dXn.GpuiTizen is a GPUI platform for Tizen, and the Linux desktop's crates neither build nor
belong in a TV library.

- The new default feature **`platform`** takes `gpui_platform`. `run_app`, the `embed` example and the text tests use
  it.
- Without it, `AppView` and `open_app_window` draw in a GPUI app whose platform the host brings.
- The development host (`nsh-gpui-host`) turns it on. Default builds are unchanged.
- `AppView`'s doc example opens its window in a GPUI app the reader runs, whichever platform runs it, so the doc tests
  build without the feature.

`docs/dev/gpui.md` and the website's GPUI page (English and zh-CN) describe the feature.

## Public API

- `nsh-platform-gpui`: feature `platform` (default), taking `gpui_platform`; `nsh_platform_gpui::run_app` exists only
  with it.

## Breaking Changes

None with default features. A crate that depends on `nsh-platform-gpui` with `default-features = false` and calls
`run_app` adds the feature:

```diff
- nsh-platform-gpui = { version = "0.4", default-features = false, features = ["js"] }
+ nsh-platform-gpui = { version = "0.4", default-features = false, features = ["platform", "js"] }
```

## How to Test

- `cargo check --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui` (default features).
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --doc --no-default-features --features js`:
  the doc tests without the platform.
- `cargo tree --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features --features js -i gpui-fast-platform`:
  not in the graph.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): Cargo features only.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (GPUI only: not applicable).
