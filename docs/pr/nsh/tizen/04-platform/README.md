# pr/tizen/04-platform

| | |
|---|---|
| Upstream PR | not opened |
| Fork PR | none of its own: in dXnSix/nsh#201, dXnSix/nsh#202 (its stack tops'), into the fork's `main`, on the owner's go |
| Branch | `pr/tizen/04-platform`, head `c8fd8b867`, pushed to `dXnSix/nsh` |
| Base | `pr/tizen/03-gpui-tizen-cfg` (head, `d389a5889`), with `pr/tizen/01-os` (head: `da9128b19` at `8b3bee883`, then `a892c2d4c` for `5c1bfe2fe` and with it the latest `pr/tv/02-category`) and `pr/tv/01-keys` (head: `9f083a6c9`, then `39e6b0c8e` for its last three commits) merged in; on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `.github/workflows/gpui.yml`, `nsh-platform-gpui`'s `app.rs` and `dev.rs`, `crates/nsh-core/src/prelude.rs`, `crates/nsh-platform-web/src/browser/engine.rs` (02's, new with `a892c2d4c`), `crates/nsh-platform-web/web/nsh.test.mjs`, `website/zh-CN/docs/gpui.md`. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `ac5701419`: `ee2b2f4d4`, `c71bb812c`, `8c8e58a8e`, `c8fd8b867` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 01, 03 and `pr/tv/01-keys` merged; **dXn.GpuiTizen on crates.io** (`dxn_gpui_tizen`, `dxn_gpui_tizen_sys` 0.1.0) and a gpui-fast release with the TV changes, so the git dependencies and the local patch go ([../README.md](../README.md#outside-nsh-what-04-onwards-need)) |
| Size | own: 39 files, +2659 −75 (against the 01, 03 and `pr/tv/01-keys` it holds): the new crate `crates/gpui/nsh-platform-tizen/` (`Cargo.toml`, `README.md`, `host/` with `Program.cs` and `Nsh.Tizen.Host.csproj`, `src/{app,back,dev,host,keys,lib,lifecycle,package,prompt,settings,tizen}.rs`); `nsh-platform-gpui` (`app.rs`, `dev.rs`, `host_settings.rs`, `keys.rs`, `lib.rs`, `navigation.rs`, `surface.rs`, `system.rs`); `nsh-dev` (`client.rs`, `lib.rs`, `tests/roundtrip.rs`); `crates/gpui/Cargo.toml` and `Cargo.lock`; `.github/ci/paths.py`, `.github/workflows/gpui.yml`; `AGENTS.md`; the new `docs/dev/tizen.md`, `docs/dev/{embedding,gpui,tooling}.md`, the website's GPUI page (English and zh-CN). With everything below it: 110 files, +5368 −312 |
| Its commits | `832c3fe4f` gpui: Let embedding apps set the app's state and take an unhandled Back; `affb03293` dev: Let a dev host connect over connections of its own; `83a99e7c9` tizen: Add nsh-platform-tizen, the Samsung TV backend; `9109e7ac3` tizen: Add the TV's development runtime; `23af6e7f0` tizen: Build the TV backend by name in the GPUI workspace; `339182cda` tizen: Take dXn.GpuiTizen with pause and resume reported to GPUI; `384758c3c` tizen: Keep the C# host's project loadable; `a1225fec5` tizen: Check that the C# host's project loads as MSBuild reads it; `ce04c6d13` tizen: Listen for sdb forward on the TV's loopback only; `6310504d6` tizen: Keep the GPUI workspace's crates publishable with dXn.GpuiTizen; `cb6aa2573` tizen: Start the app with the TV's settings; `e752c035f` tizen: Ask whether to leave in the TV's language, at its text size, to Voice Guide too (merges: `da9128b19`, `9f083a6c9`, `d389a5889`, `39e6b0c8e`, `a892c2d4c`) |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; held at its head by 05, 06, 07 and `pr/tizen-web/03-module` |
| Review findings | s18's (`review-s18.json` failed, `review-s18-2.json` passed at snapshot `d90aea97`): `f-s18-host-csproj-xml` (major: `--` in the project's comment, illegal in XML, made MSBuild refuse it) fixed by `384758c3c` and checked by `a1225fec5` (`host::tests` parse the csproj as XML, `the_project_is_well_formed_xml_on_tizens_sdk`, `a_double_hyphen_in_a_comment_is_refused`); `f-s18-dev-listen-all-interfaces` (major: the dev fallback bound `0.0.0.0:8791`, so any host on the TV's LAN could act as the dev server) fixed by `ce04c6d13`, loopback only (`the_tv_listens_on_its_loopback_only`); `f-s18-publish-git-dependency` (minor: `cargo publish` rejects git dependencies without a version) fixed by `6310504d6`, `publish = false` on `nsh-platform-tizen` and `version = "0.1.0"` beside `git` and `rev`; `f-s18-settings-after-load` (minor: the engine started with the process defaults) fixed by `cb6aa2573`, the TV's settings before `load` (`the_tvs_settings_are_set_before_the_app_loads`); `f-s18-exit-prompt-locale-a11y` (minor: fixed English, fixed sizes, nothing for Voice Guide) fixed by `e752c035f`, the question in the TV's language, at its text size, as an alert dialog |
| Needs | the Tizen issue answered (GPUI as the runtime of packaged apps; the dXn.GpuiTizen dependency); no nsh-compat change |

## Before opening

- **The dependency.** `crates/gpui/Cargo.toml` has `dxn_gpui_tizen` and `dxn_gpui_tizen_sys` as
  `{ version = "0.1.0", git = "https://github.com/dXnSix/dXn.GpuiTizen", rev = "83cd3e07…" }`. Upstream will want a
  crates.io version only: publish dXn.GpuiTizen first, then drop `git` and `rev`, and refresh `crates/gpui/Cargo.lock`.
  dXn.GpuiTizen is not on GitHub yet either.
- **The forks.** Until gpui-fast releases its TV changes (and wgpu its GL ones), the TV build needs a local
  `[patch.crates-io]`; `nsh-platform-tizen` stays out of `default-members`, so upstream's `gpui.yml` builds without it.
  Say in the PR what upstream can build and what only the fork's CI does.
- **Size.** 12 commits and a new crate. If the maintainers ask for a split: `832c3fe4f` (AppView's
  `set_lifecycle`, `on_unhandled_back`, `nsh_key`) and `affb03293` (`DevTransport`, `connect_dev_server_with`) are
  generic and can go first on their own; the crate follows.
- `384758c3c`, `a1225fec5`, `ce04c6d13`, `cb6aa2573` and `e752c035f` are follow-ups to `83a99e7c9` and `9109e7ac3`
  (the C# project, the `sdb forward` listener's address, the settings before the first load, the exit question's
  words): squash them into those if the maintainers prefer one commit per part.
- `keys.rs` now meets `pr/tv/01-keys`' last commit (`f908ff626`: Info is `info`, `xf86info` an alias) on the branch
  (merge `39e6b0c8e`), and 01 and `pr/tv/02-category` are held at their heads (`a892c2d4c`); the s18 checks ran before
  those merges, on `feat/tizen`, which had all of it.

## Checks

The Tizen backend's checks need `$TIZEN_ENV`, `$PATCH` and `$LOCK` ([../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)).

| Check | Command | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-platform-tizen` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-tizen --lib --config "$PATCH" --config "resolver.lockfile-path='$LOCK'"` | `t18-platform`, `t18-dev`, `t18-module` | passed, 27 tests (04's 23, and 06's `tv::tests` and `keys::tests::nsh_keys_have_their_tizen_names`), 2026-11-10 15:49 UTC (s18) |
| `chk-nsh-platform-tizen-dev` | the same with `--features dev` and the filter `dev::` | `t18-dev` | passed, 5 tests, 16:09 UTC (s18) |
| `chk-nsh-gpui-embedding-tv` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib --config "$PATCH" --config "resolver.lockfile-path='$LOCK'" -- host_settings::tests::lifecycle navigation::tests::unhandled_back keys::tests` | `t18-platform` | passed, 5 tests (s18) |
| `chk-nsh-dev-transport` | `cargo test -p nsh-dev --features client,server --test roundtrip` | `t18-dev` | passed, 12 tests (s18) |
| `chk-nsh-tizen-link-armv7` | `cargo xtask cross --target armv7 --release --manifest-path ../nsh/nsh-wt/tizen/crates/gpui/Cargo.toml -p nsh-platform-tizen --with-dev-variant --check-symbols --entry nsh_tizen_main --config "$PATCH"` (from dXn.GpuiTizen, with `$TIZEN_ENV`) | `t18-link` | passed, 15:56 UTC (s18) |
| `chk-nsh-tizen-link-x86_64` | the same with `--target x86_64` | `t18-link` | passed, 16:02 UTC (s18) |
| `chk-nsh-tizen-link-aarch64` | the same with `--target aarch64` | `t18-link` | passed, 16:09 UTC (s18) |

The link checks need the Tizen 8.0 rootstraps and the SDK's GCC toolchains.

**On this branch alone** (no result above is from it): `chk-nsh-platform-tizen` runs as written but selects 23 tests
there: 4 of the recorded 27 are 06's. The link checks' `--manifest-path` names `nsh-wt/tizen`, which is `feat/tizen`:
point it at a worktree that has this branch. The other commands run as written. Not run: the pre-open checks, anything
on a TV or the emulator (the owner's `DEV` runs), `dotnet build` of the host.

## Updating and opening

Nothing is opened before the owner's agreement with NSH's owners ([../../../README.md](../../../README.md#prepared-not-opened-2026-11-12)).
The branch that opens is `up/tizen/04-platform`, never `pr/tizen/04-platform`, which stays as the epic's records name it.
Before opening, rebuild `up/tizen/04-platform` on the newest `upstream/main` (or its parent's `up/` branch, once that has
merged upstream) as [../../README.md](../../README.md#updating-a-branch-and-opening-it) says, and run the checks
above on its tip. Then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git push --force-with-lease origin up/tizen/04-platform
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/04-platform
gh pr create --repo longbridge/nsh --base main --head dXnSix:up/tizen/04-platform \
  --title "tizen: Add nsh-platform-tizen, the Samsung TV backend" --body "$(sed '1,2d' PR.md)"
```
