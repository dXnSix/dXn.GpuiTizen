# core: Add Os::Tizen

Part of #(Tizen issue). **Depends on:** #(pr/tv/02-category), merged.

> [!NOTE]
> Needs longbridge/nsh-compat#(tizen-platform) ("compat: Run a script as a native Tizen app with `platform tizen`"):
> the new `tizen` compat domain runs on `platform tizen`. Merge it first.

## Description

A native app on a Samsung TV runs on Tizen, Samsung's Linux-based TV OS. It has an OS of its own: not a desktop, and a
TV by default.

- `Os::Tizen`: `platform.os` `"tizen"`, the JS `OS` type's `"tizen"`, `platform.select({ tizen })` through the OS's
  key (before `tv`). `Os::Tizen.is_desktop()` is false; keys follow the PC conventions, as on Linux.
- Its idiom is the TV's: `Idiom::default_on(os)` (a desktop OS's `Desktop`, Tizen's `Tv`, else `Phone`), which
  `Environment::default` and `Idiom::reported_on` use: `Idiom::Desktop` reads as the OS's own idiom where it does not
  apply. So desktop behaviour never applies on Tizen but in a terminal, and `platform.tv` follows the idiom as reported.
- A Tizen build is `target_os = "linux"` with `target_vendor = "tizen"`: `Os::default` checks the vendor first, and both
  workspaces declare it to `unexpected_cfgs`. The GPUI and terminal backends check it before falling back to Linux, and
  GPUI's environment takes the OS's default idiom; the agent's keystrokes parse for the same OS.
- A Tizen web app (`.wgt`) stays `Os::Web`, a TV by what its page tells the player (#(pr/tv/02-category)).

Docs: `docs/dev/{api-naming,compatibility,elements,js-modules,platforms}.md`; the website's device-and-environment,
platform-specific and `nsh` module pages (English and zh-CN).

## Public API

- `nsh_core::platform::Os::Tizen`; `Idiom::default_on(os: Os) -> Idiom`.
- TS: `OS` gains `"tizen"`; `platform.select` takes a `tizen` key.

## Breaking Changes

- `Os` gains a variant and is not `#[non_exhaustive]`: Rust code that matches it exhaustively needs an arm.
- `Environment::default()`'s idiom is the build OS's own (`Idiom::default_on(Os::default())`), not always `Phone`: on
  macOS, Windows and Linux builds it is `Desktop`. The backends and the headless driver set the idiom themselves, so
  apps see no change; the core's environment before a backend sets one, and embedders and tests that start from
  `Environment::default()` on a desktop, do.

## How to Test

- `cargo test -p nsh-core --lib platform::tests::tizen`: its own OS, never a desktop in a device idiom (a terminal is),
  a TV by default, `Idiom::Desktop` reported as `Tv` there.
- `cargo test -p nsh-js --lib env::tests::tizen` and `env::tests::tv`: `platform.os`, `platform.select({ tizen, tv })`
  order, `platform.tv`.
- `cargo xtask compat` with longbridge/nsh-compat#(tizen-platform): `tizen/mod-device-tizen`.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): `tizen/mod-device-tizen`.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (no change there: not applicable).
