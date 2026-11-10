# tizen: `Os::Tizen` and the native Samsung TV target

NSH apps running natively on Samsung TVs (Tizen 8.0 and later): `Os::Tizen`; `nsh-platform-gpui` building for Tizen
without the desktop platform; `nsh-platform-tizen`, a TPK's `libnsh.so` that runs the GPUI backend's `AppView` full
screen on [dXn.GpuiTizen](https://github.com/dXnSix/dXn.GpuiTizen)'s Tizen platform for GPUI; videos through the TV's
player; `nsh/tizen`; and `--target tizen` in the CLI. The GPUI backend becomes the runtime of packaged apps there,
which the issue raises first ([issue.md](issue.md)).

## The PRs, in order

All local in `nsh-wt/tizen`, not pushed; built on `upstream/main` `8c733731f`. Own size: the branch against the parents
it holds; total: against `8c733731f`.

| # | Folder | Branch | Built on | Title | Own size | Needs | Status |
|---|---|---|---|---|---|---|---|
| 01 | [01-os/](01-os/) | `pr/tizen/01-os` | `pr/tv/02-category` (head, merged in by `5c1bfe2fe`) | core: Add Os::Tizen | 28 files, +374 −80 | the issue; `pr/tv/02-category` merged; nsh-compat [`tizen-platform`](../compat/tizen-platform/) | not opened |
| 02 | [02-gpui-platform-optional/](02-gpui-platform-optional/) | `pr/tizen/02-gpui-platform-optional` | `upstream/main` `8c733731f` | gpui: Make gpui_platform an optional default feature | 8 files, +35 −17 | the issue | not opened |
| 03 | [03-gpui-tizen-cfg/](03-gpui-tizen-cfg/) | `pr/tizen/03-gpui-tizen-cfg` | 02 | gpui: Build for Tizen | 18 files, +645 −62 | 02 merged | not opened |
| 04 | [04-platform/](04-platform/) | `pr/tizen/04-platform` | 03, merges 01 and `pr/tv/01-keys` (all at their heads) | tizen: Add nsh-platform-tizen, the Samsung TV backend | 39 files, +2659 −75 | 01, 03, `pr/tv/01-keys` merged; dXn.GpuiTizen on crates.io; a gpui-fast release with its changes | not opened |
| 05 | [05-video/](05-video/) | `pr/tizen/05-video` | 04 | gpui: Play videos on Samsung TVs through the TV's player | 9 files, +1706 −131 | 04 merged | not opened |
| 06 | [06-module/](06-module/) | `pr/tizen/06-module` | 04 | core: Add nsh/tizen, what Samsung TVs have | 17 files, +790 −7 | 04 merged | not opened |
| 07 | [07-cli/](07-cli/) | `pr/tizen/07-cli` | 06, merges 05 and `pr/tizen-web/01-package` (all at their heads) | cli: Run, package and test apps natively on Samsung TVs | 40 files, +5069 −87 | 05, 06 and `pr/tizen-web/01-package` merged | not opened |

```
pr/tv/02-category ── 01-os ──────────────┐
pr/tv/01-keys ───────────────────────────┤
02-gpui-platform-optional ── 03-gpui-tizen-cfg ── 04-platform ──┬── 05-video ──┐
                                                                └── 06-module ─┴── 07-cli
pr/tv/02 + pr/tv/01 ── pr/tizen-web/01-package ────────────────────────────────────┘
```

The order of opening: the issue; 01, 02 (they need nothing of dXn.GpuiTizen and may go earlier, the owner's call);
03; then, once a gpui-fast release has the TV changes and dXn.GpuiTizen is on crates.io, 04; 05 and 06; then
`pr/tizen-web/01-package` ([../tizen-web/README.md](../tizen-web/README.md)); then 07.

**07 needs `pr/tizen-web/01-package`**: the native target reads the `nsh.json` `tizen` section that 01-package adds
(`package::TizenConfig`). So in [../../README.md](../../README.md)'s order the first `tizen-web` PR comes before the
last `tizen` one, after the loader probe ([../tizen-web/README.md](../tizen-web/README.md#the-loader-probe-dev-6)).

## Stack state (2026-11-10, 19:30 CET)

The branches moved forward by merges only (no rebase) through the s17–s21 repairs; at 18:45 each took its parents'
latest heads:

- **01 holds the latest `pr/tv/02-category`** (merge `5c1bfe2fe`). Moving it with `git rebase --onto upstream/main
  pr/tv/02-category pr/tizen/01-os` after 02 merges leaves `6b69e7433` and `8b3bee883`.
- **04 holds the latest `pr/tv/01-keys`** (merge `39e6b0c8e`: `b8bc8dee1`, `c621bc72b`, `f908ff626`, so `keys.rs` and
  the Info key's name, `info` with `xf86info` an alias, now meet on the branch), 03 at its head, and since 19:45 CET
  01 at its head (merge `a892c2d4c`, and with it the latest 02: `532345852`, `a8f7ad63c`).
- 05 and 06 hold 04 at its head (merges `d15dae74b`, `8f55629e3`); 07 holds 05, 06 and `pr/tizen-web/01-package` at
  their heads (merges `25280372a`, `976dd2ac4`, `b126bf60f`).
- `pr/tizen-web/03-module` holds 06 at its head (merge `661d69f2d`).
- 07's `20404bb35` is the same patch as 04's `384758c3c` (`git patch-id`); it drops out of the move.
- All of them are in `feat/tizen` (head `7c5cd1c13`).
- Against `upstream/main` `c16a70096` (unchanged): 01 conflicts in `crates/nsh-core/src/prelude.rs` and
  `crates/nsh-platform-web/src/browser/engine.rs` (as `pr/tv/02-category`); 02 and 03 in
  `website/zh-CN/docs/gpui.md`; 04–06 in `.github/workflows/gpui.yml`, `nsh-platform-gpui`'s `app.rs` and `dev.rs`,
  `crates/nsh-core/src/prelude.rs`, `crates/nsh-platform-web/src/browser/engine.rs` (02's, since `a892c2d4c`),
  `crates/nsh-platform-web/web/nsh.test.mjs`, `website/zh-CN/docs/gpui.md`; 07 also in `nsh-cli`'s `main.rs`, `package.rs`, `rust_app.rs`, `test_web.rs`, `crates/nsh-platform-web/src/browser/engine.rs`,
  `crates/xtask/src/dist.rs` and the website's CLI and project-structure pages (English and zh-CN). Upstream's #1439
  (web views report they are unavailable, `Environment::web_views`) overlaps 03's web view placeholder, #1550
  (`data_dir()`) 04's data directories; #1420 (a hidden main window), #1439, #1467 and #1476 changed
  `nsh-platform-gpui`'s `app.rs` or `dev.rs`, where 03 and 04 add the TV's setters, `set_lifecycle`,
  `on_unhandled_back` and `connect_dev_server_with`.

## The reviews

Each sprint's independent review failed first and passed after its repairs, every repair a commit appended to the
branch (no rebase) with its own test group:

| Sprint, review | Finding | Branch | What changed |
|---|---|---|---|
| s17, `review-s17-2.json` | `f-s17-js-tizen-tests-unchecked` (major) | 01 | no source change: the new check `chk-nsh-os-tizen-js` runs the four `env::tests::tizen` |
| | `f-s17-appview-doctest-needs-platform` (minor) | 02 | `AppView`'s example compiles without the `platform` feature (`2f7eb1758`, `c22f7a995`) |
| | `f-s17-platform-feature-reaches-tizen` (minor) | 03 | a Tizen build that keeps `platform` stops with a `compile_error!` naming `--no-default-features --features js` (`5c719ccbf`) |
| | `f-s17-tv-ui-lines-name-match` (minor) | 03 | a TV's UI family is matched ignoring case and spaces for its line metrics (`6325f81ea`, `2e0e29dc5`) |
| s18, `review-s18-2.json` | `f-s18-host-csproj-xml` (major) | 04 | the C# project's comment had `--`, illegal in XML (MSB4025); fixed (`384758c3c`) and parsed by `host::tests` (`a1225fec5`) |
| | `f-s18-dev-listen-all-interfaces` (major) | 04 | the dev fallback listens on the TV's loopback only, not `0.0.0.0` (`ce04c6d13`) |
| | `f-s18-publish-git-dependency` (minor) | 04, 05 | `nsh-platform-tizen` is `publish = false`; the git dependencies carry `version = "0.1.0"` (`6310504d6`); 05 says videos need dXn.GpuiTizen on crates.io (`acd5d258f`) |
| | `f-s18-settings-after-load` (minor) | 04 | the TV's settings are set before the app loads (`cb6aa2573`) |
| | `f-s18-exit-prompt-locale-a11y` (minor) | 04 | the exit question is localized, follows the font scale and is an alert Voice Guide reads (`e752c035f`) |
| | `f-s18-video-seek-before-ready` (major) | 05 | seeks wait for the player's `Ready` (`1b7a46646`) |
| | `f-s18-video-interruption-unrecovered` (major) | 05 | an interrupted video is prepared, restored and resumed once the app is active again (`1b7a46646`) |
| s19, `review-s19-3.json` | `f-s19-app-library-extra-exports` (major) | 07 | an app's library has one export, `nsh_tizen_main`, through feature `entry` (`5c22de690`) |
| | `f-s19-test-driver-open-sequence` (major) | 07 | `nsh run` and `nsh test` start the runtime through one typed `open_sequence` with failure policies (`9244408df`) |
| | `f-s19-run-sequence-not-executed` (minor) | 07 | the same (`9244408df`) |
| | `f-s19-app-library-needs-dxn-checkout` (major) | 07 | the CLI checks an app's library itself against the component's `symbols.allow` (`55f2d6e7f`) |
| | `f-s19-store-zip-includes-emulator` (minor) | 07 | the store upload takes armv7l and aarch64 only (`a5857a017`) |
| | `f-s19-registry-rust-apps-unresolvable` (minor) | 07 | apps that take nsh from crates.io are refused for `--target tizen` up front (`5c22de690`) |
| | `f-s19-cross-hint-messages` (minor) | 07 | the SDK hints lose a run of spaces and give `setup.md`'s install command (`5c22de690`) |
| | `f-s19-tv-library-panic-strategy` (minor) | 07 | unwinding is kept and documented as an exception to dXn.GpuiTizen's `abort` (`7cb4e3fb2`) |
| | `f-s19-connect-dev-server-with-api-churn` (minor) | 07 | the four-argument `connect_dev_server_with` is recorded as superseding the three-argument one (`7cb4e3fb2`) |

06 had no finding. s16's `f-s16-tv5-os-tizen-unmet` moved `Os::Tizen`'s compat case to 01 (NTZ-2) by amendment.

## Outside NSH: what 04 onwards need

- **dXn.GpuiTizen on crates.io.** The GPUI workspace takes `dxn_gpui_tizen` and `dxn_gpui_tizen_sys` as git
  dependencies pinned by `rev` (`83cd3e076c7c09861e67f7d1f37f975c3362e828`, dXn.GpuiTizen's "docs: describe pause and
  resume reporting, run-time key grabs and system_info"; its `main` is `21412a9` since, 8 commits on), with
  `version = "0.1.0"` beside them for `cargo publish`.
  `release-crate.yml` publishes `nsh-platform-gpui`, which depends on `dxn_gpui_tizen` on Tizen from 05 on, so
  dXn.GpuiTizen must be on crates.io first; upstream will want the `git`/`rev` gone. dXn.GpuiTizen is not on GitHub
  yet either (`git ls-remote https://github.com/dXnSix/dXn.GpuiTizen` lists no refs, 2026-11-10 19:30 CET).
- **gpui-fast and wgpu forks.** dXn.GpuiTizen builds on Linux only against the gpui-fast fork (its EGL path uses the
  external-target renderer, its video the holes) and the wgpu fork (EGL 1.4, GL program binaries). In NSH that patch
  is a local Cargo config, never committed; `nsh-platform-tizen` is out of the GPUI workspace's `default-members`, so
  upstream's CI builds without it. The upstream side of those forks: [../../gpui-fast/README.md](../../gpui-fast/README.md)
  and [../../wgpu/README.md](../../wgpu/README.md).
- **One wgpu**: `pr/engine/01-wgpu-30` ([../engine/README.md](../engine/README.md)) for one GL device on the TV.

## Rules for this area

nsh-root's ([../README.md](../README.md#rules)). Upstream has no Tizen CI: the TV builds are checked by the fork's
own workflow ([../fork/README.md](../fork/README.md)); upstream's CI keeps building the GPUI workspace's default
members.
