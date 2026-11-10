# js: Add platform.tv, the tv prop group and the tv select key

Part of #(TV issue).

> [!NOTE]
> Needs longbridge/nsh-compat#(tv-idiom) ("compat: Run a script on a TV with `platform <os> tv`"): the new `tv`
> compat domain runs on `platform android tv`. Merge it first.

## Description

`Idiom::Tv` exists, but an app cannot tell it is on a TV, cannot style for it, and on a desktop OS the TV idiom counts
as desktop. A TV is a category of its own, on any OS (Android TV, a TV's browser, Samsung's Tizen):

- `platform.tv` in TSX, `Environment::tv()` and `is_tv()` in Rust: true in the TV idiom, on any OS.
- `tv={{…}}` props on every element apply only there (QuickJS' and the browser's renderers; `TvProps`, the props
  every element takes, as no prop is a TV's alone).
- `platform.select` looks for `tv` after the OS's own key and before `desktop` / `mobile`.
- A TV is never desktop (`Idiom::desktop_on` is false for `Idiom::Tv` on every OS, a desktop OS included), so
  `platform.desktop` is false there and `nsh/base` presents itself as on mobile.
- The web player's `idiom` option takes `"tv"`, which `nsh_environment` reads as `Idiom::Tv`: a page on a TV (a Tizen
  `.wgt`) tells the player it is on a TV, which the pointer heuristic cannot. The demo player takes `?idiom=tv`.

Docs: the website's device-and-environment and platform-specific pages, the `nsh` module and elements index (English
and zh-CN); `docs/dev/` (api-naming, compatibility, components, elements, js-modules, native-pipeline, web).

## Public API

- TSX: `platform.tv: boolean`; the `tv?: TvProps` prop group on every element; `platform.select({ tv })`.
- Rust: `nsh_core::platform::Environment::tv()`, `nsh_core::prelude::is_tv()` (`element::ambient`).
- The web player: `idiom: "tv"` (the option existed; `"tv"` is new).

## Breaking Changes

- `Idiom::Tv` on a desktop OS (macOS, Windows, Linux) was a desktop (`desktop={{…}}`, `platform.desktop`); it is not
  any more. No released backend reports `Idiom::Tv` on a desktop OS, so no shipped app sees it.

## How to Test

- `cargo test -p nsh-js --lib env::tests::tv`: `platform.tv`, `platform.select` order, never desktop.
- `cargo test -p nsh-base --test base tv_props::`: `tv={{…}}` applied on a TV only.
- The browser-tests module and
  `NSH_BROWSER_WASM=target/wasm32-unknown-unknown/web/nsh_platform_web.wasm node --expose-gc --test crates/nsh-platform-web/web/browser/tv.test.mjs`:
  the page's `idiom: "tv"` reaches the module (`nsh_environment`), the runtime gets idiom `tv`, `tv` true, `desktop`
  false, and applies `tv={{…}}` props, not `desktop={{…}}` ones; every documented idiom read back, an unknown one as a
  phone.
- `cargo xtask compat` with longbridge/nsh-compat#(tv-idiom): `tv/mod-device-tv`, `tv/props-platform-tv`.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): `tv/mod-device-tv`, `tv/props-platform-tv`.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (an Android TV emulator for the TV idiom).
