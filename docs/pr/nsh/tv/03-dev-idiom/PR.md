# cli: Run apps in the TV idiom on the desktop

Part of #(TV issue). **Depends on:** #(pr/tv/02-category), merged. No nsh-compat change.

## Description

TV layouts (10-foot sizes, `tv={{…}}` props, `platform.tv`) and TV focus are slow to build and to test on a TV. With
this they are built on a Mac with hot reload, and their specs run on the desktop and in CI:

- `nsh run` and `nsh dev` take `--idiom tv|desktop`, over `nsh.json`'s `"dev": { "idiom": "tv" }`.
- `nsh test --target gpui|web` takes the same `--idiom`, over the same `nsh.json` setting: each test's GPUI host starts
  with `--idiom`, and the web test page gets `IDIOM`, over the idiom a test's desktop `platform` asks for.
- The GPUI host gets `--idiom tv` (`AppOptions::idiom`, the surface's starting idiom; the window keeps its size).
- The web dev page gets `IDIOM`, the player's `idiom` option (also the page `nsh dev --server-only` serves).
- The other hosts (and headless `nsh test` runs) run in their device's own idiom: they refuse the flag before anything
  starts, and leave `nsh.json`'s setting out with a warning.
- `tests/apps/tv-idiom` is an app built for TVs (`dev.idiom`: `tv`) whose spec checks that `platform` reports a TV on
  this machine's OS and that `tv={{…}}` props apply and `desktop={{…}}` ones do not; in the desktop idiom both of its
  tests fail.

Docs: `docs/dev/{gpui,tooling,web}.md`, the website's CLI, GPUI, platform-specific, project-structure, testing and web
pages (English and zh-CN).

## Public API

- CLI: `nsh run --idiom <tv|desktop>`, `nsh dev --idiom <tv|desktop>`, `nsh test --idiom <tv|desktop>` (`--target gpui`
  and `--target web`).
- `nsh.json`: `dev.idiom` (`"tv"` or `"desktop"`).
- `nsh_platform_gpui::AppOptions::idiom: Idiom` (default `Idiom::Desktop`).

## How to Test

- `cargo test -p nsh-cli --test cli dev_idiom::` (8 tests): parsing, `nsh.json`'s setting under the flag, what the GPUI
  host, the web dev page and `nsh test`'s hosts and test page receive, the other targets refusing the flag.
- `nsh test tests/apps/tv-idiom --target gpui --idiom tv` (after `cargo build --manifest-path crates/gpui/Cargo.toml -p
  nsh-gpui-host`), and `--target web --idiom tv`: 2 passed each (also with `nsh.json`'s `dev.idiom` alone); with
  `--idiom desktop` both fail.
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib app::idiom_tests`.
- By hand: `nsh run --target gpui --idiom tv` on an app with `tv={{…}}` props: they apply, `platform.tv` is true.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): CLI and `nsh.json` only.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (development hosts only: not applicable).
