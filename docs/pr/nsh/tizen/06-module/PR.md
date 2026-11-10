# core: Add nsh/tizen, what Samsung TVs have

Part of #(Tizen issue). **Depends on:** #(pr/tizen/04-platform), merged. Its compat case runs in the `tv` domain of
#(pr/tv/02-category) (longbridge/nsh-compat#(tv-idiom)).

## Description

A Samsung TV app needs a few things no other platform has: the TV's product information, the remote's color, media,
channel and Info keys (which the TV keeps unless the app registers them), leaving the app, and Smart Hub's preview.

`nsh/tizen` is a runtime module of its own (`runtime/tizen.rs`, flat functions, declared from its spec), importable
everywhere:

- `isAvailable()`: whether the app runs on a Samsung TV (the TV's host put its side of the module, a `TizenHost`, in
  the engine's context as `TizenServices`).
- `productInfo()`: the TV's model, Tizen version, firmware build and maker.
- `registerKey(key)` / `unregisterKey(key)`: the remote's color, media, channel and Info keys by their `nsh/keymap`
  names, checked here (a `TypeError` for no key, a `RangeError` for one the TV does not give apps) before the host
  takes or gives them back.
- `exitApp()`: leaves the app (`Command::ExitApp`).
- `setPreviewData(data)`: Smart Hub's preview, which has no public native API: natively it rejects; a TV's web app
  sets it.

Without the TV's host every call rejects with `ERR_FEATURE_UNAVAILABLE_ON_PLATFORM`.

The TV host (`nsh-platform-tizen`'s `tv.rs`) gives the product information from dXn.GpuiTizen's
`system_info::product_info` (read once at start) and the keys by their Tizen names (`keys::tizen_name`): it checks a
key on the engine's thread (`keygrab::validate`) and grabs or releases it on the main thread (`keygrab::grab`,
`release`). The color, media and channel keys are the app's from the start, Info on request; giving one back hands it
to the TV, and the grabs follow pause and resume.

Docs: the website's new `nsh/tizen` page and the modules index (English and zh-CN), `docs/dev/{js-modules,tizen}.md`.

## Public API

- `nsh/tizen` (TS): `isAvailable(): boolean`, `productInfo(): Promise<ProductInfo>` (`model`, `platformVersion`,
  `build`, `manufacturer`; each `string | null`), `registerKey(key: string): Promise<void>`, `unregisterKey(key: string):
  Promise<void>`, `exitApp(): Promise<void>`, `setPreviewData(data: unknown): Promise<void>`.
- `nsh_core::runtime`: `TizenHost` (trait), `TizenServices`, `TvProduct`.

## How to Test

- `cargo test -p nsh-core --lib runtime::tizen` (the calls with and without a host, the keys' checks and errors).
- `cargo test -p nsh-js --lib declarations::tests::tizen`.
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-tizen --lib tv::tests keys::tests` (with the
  forks' Cargo config, `docs/dev/tizen.md`): what is grabbable, the grabs sent to the main thread, the product
  information, every NSH key's Tizen name.
- `cargo xtask compat`: `tv/mod-tizen` (no TV: unavailable, every call rejecting with
  `ERR_FEATURE_UNAVAILABLE_ON_PLATFORM`).

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): `tv/mod-tizen`.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (unavailable there, as the compat case checks).
