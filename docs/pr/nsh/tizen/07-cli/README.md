# pr/tizen/07-cli

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tizen/07-cli`, head `cfa18ec5b`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-os` |
| Base | `pr/tizen/06-module` (head, merge `976dd2ac4`), with `pr/tizen/05-video` (head, merge `25280372a`) and `pr/tizen-web/01-package` (head, merge `b126bf60f`, and with it the latest `pr/tv/02-category`) merged in; on `upstream/main` `8c733731f`. Against `c16a70096` it has 04's and 01-package's conflicts and more: `.github/workflows/gpui.yml`, `nsh-platform-gpui`'s `app.rs` and `dev.rs`, `nsh-cli`'s `main.rs`, `package.rs`, `rust_app.rs`, `test_web.rs`, `crates/nsh-core/src/prelude.rs`, `crates/nsh-platform-web/src/browser/engine.rs`, `web/nsh.test.mjs`, `crates/xtask/src/dist.rs`, the website's CLI and project-structure pages (English and zh-CN) and `website/zh-CN/docs/gpui.md`. Not rebased |
| Depends on | 05, 06 and `pr/tizen-web/01-package` merged (the `nsh.json` `tizen` section is 01-package's `package::TizenConfig`) |
| Size | own: 40 files, +5069 −87 (against the 05, 06 and 01-package it holds): `nsh-cli` (the new `src/tizen.rs` +1979 and `src/tizen/symbols.rs` +430, `src/test_tizen.rs` +120, `src/runtime.rs` +101, `src/rust_app.rs` +433, `component.rs`, `dev.rs`, `main.rs`, `package.rs`, `run.rs`, `test.rs`, `test_host.rs`, `tizen_web.rs`, the new `tests/cli/tizen.rs` +413), `xtask` (the new `src/tizen.rs` +684, `dist.rs`, `main.rs`), `nsh-dev/src/protocol.rs` (`HostPlatform::Tizen`), `nsh-platform-gpui`'s `app.rs` and `dev.rs`, `nsh-platform-tizen` (`Cargo.toml`: feature `entry`, `README.md`, `src/dev.rs`, `src/lib.rs`), `AGENTS.md`, `docs/dev/{gpui,tizen}.md`, the website's new Tizen page and its CLI, GPUI, installation, platforms and project-structure pages (English and zh-CN). With everything below it: 164 files, +15709 −512 |
| Its commits | `20404bb35` tizen: Keep the C# host's project loadable (the same patch as 04's `384758c3c`); `5cea54e4b` dev: Tell Tizen hosts apart in the dev protocol; `1ca68cd89` tizen: Serve the UI sets of an app's Rust crates from its library; `ef365beaa` xtask: Build the Tizen runtime through dXn.GpuiTizen's cross driver; `a38dcbeca` cli: Run, package and test apps natively on Samsung TVs; `22c48fe78` tizen: Say in the crate's README how NSH's runtime is built; `fdcb28185` cli: Build an app's Rust crates into its Tizen library; `5c22de690` tizen: Give an app's Tizen library one export, checked and stripped as the runtime's; `9244408df` cli: Start the Tizen runtime the same way for nsh run and nsh test; `a5857a017` cli: Leave the emulator's package out of the Tizen store upload; `7cb4e3fb2` tizen: Say why the TV libraries keep unwinding, and connect_dev_server_with's signature; `55f2d6e7f` cli: Check an app's Tizen library against the component's allowlist, without dXn.GpuiTizen; `cfa18ec5b` cli: Sign Tizen packages from a zip and install them with tz (merges: `dda864fda`, `eb77cbb70`, `10ea4b8f6`, `dc2aa292b`, `03ed14fff`, `b126bf60f`, `25280372a`, `976dd2ac4`) |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merges `d1cf78021`, `4d1908579`, `b91fab916`, `1c4323c2b`) |
| Review findings | s19's nine (`review-s19.json` failed with 8, `review-s19-2.json` resolved them and raised a ninth, `review-s19-3.json` passed at snapshot `6cac72e9`): `f-s19-app-library-extra-exports` (major: an app's library exported `nsh_app_ui_sets` and `nsh_app_root` besides `nsh_tizen_main`) by `5c22de690`, the generated library defines the one export and `nsh-platform-tizen`'s own entry is behind its default feature `entry`; `f-s19-test-driver-open-sequence` (major: `nsh test --target tizen` failed its first open where the runtime was not running) and `f-s19-run-sequence-not-executed` (minor: `nsh run` picked the tested sequence's steps by position) by `9244408df`, one typed `tizen::open_sequence` with each step's failure policy, plus `nsh test --device`; `f-s19-store-zip-includes-emulator` (minor) by `a5857a017`, the upload takes armv7l and aarch64 only; `f-s19-registry-rust-apps-unresolvable` (minor: crates from crates.io cannot take the unpublished `nsh-platform-tizen`) and `f-s19-cross-hint-messages` (minor) by `5c22de690`, refused up front naming `nsh init --template rust --nsh-source <dir>`, and the hints fixed; `f-s19-tv-library-panic-strategy` (minor) and `f-s19-connect-dev-server-with-api-churn` (minor) by `7cb4e3fb2`, unwinding kept and documented as an exception, the signature change recorded under "Public API"; `f-s19-app-library-needs-dxn-checkout` (major, raised by the second review: `5c22de690`'s symbol check ran `cargo xtask check-symbols` in a dXn.GpuiTizen checkout) by `55f2d6e7f`, the CLI checks the library itself (`tizen::symbols`, a port of dXn.GpuiTizen's `symcheck`) against the `symbols.allow` the `tizen` component ships |
| Needs | no nsh-compat change |

## Before opening

- **Size.** The largest PR of the epic. Natural splits, each on its own: the dev protocol's `HostPlatform::Tizen`
  (`5cea54e4b`); `cargo xtask tizen` and the `tizen` component (`ef365beaa`); the CLI target (`a38dcbeca`,
  `9244408df`, `a5857a017`); apps with Rust crates (`fdcb28185`, `5c22de690`, `55f2d6e7f`; `1ca68cd89`'s `ui_sets!` /
  `root!` are removed again by `5c22de690`, so squash those two).
- **Fix:** `docs/dev/tizen.md` ("Public API") says `connect_dev_server_with`'s three-argument form "came in with
  `pr/tizen/06-module`"; it came with 04 (`affb03293`, `pr/tizen/04-platform`). Correct the branch name there when
  moving the branch (`docs/dev/gpui.md` names no branch).
- Nothing of it has run on a TV or the emulator (`docs/dev/tizen.md`, "Limits"): the `sdb` commands, where `tz pack`
  writes a TPK from a staged tree, and the multi-architecture packaging are dXn.GpuiTizen's conventions, unverified
  until the owner's runs (`t24-nsh-native`).
- TV libraries keep unwinding (`panic = "unwind"`), unlike dXn.GpuiTizen's `abort`, on purpose ("Panics"): a
  documented exception; say so in the PR.
- The review's non-blocking notes: `cargo xtask tizen` copies `symbols.allow` (as the target specs) from the
  dXn.GpuiTizen checkout's working tree, not from the `rev` NSH pins; and `tizen::symbols` is a port of dXn.GpuiTizen's
  `symcheck` that must follow its rules by hand.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-cli-tizen` | `cargo test -p nsh-cli --test cli tizen::` (at least 5) | `t19-config-targets`, `t19-package-run`, `t19-rust-apps-test` | passed, 6 tests, 2026-11-10 17:14 UTC (s19) |
| `chk-nsh-cli-tizen-unit` | `cargo test -p nsh-cli --bin nsh tizen::tests` | `t19-package-run` | passed, 13 tests (s19) |
| `chk-nsh-cli-tizen-symbols` | `cargo test -p nsh-cli --bin nsh tizen::symbols::tests` | `t19-rust-apps-test` | passed, 5 tests (s19) |
| `chk-nsh-cli-tizen-rust` | `cargo test -p nsh-cli --bin nsh rust_app::tests::a_tizen` | `t19-rust-apps-test` | passed, 2 tests (s19) |
| `chk-nsh-cli-tizen-test-host` | `cargo test -p nsh-cli --bin nsh test_host::tests::tizen` | `t19-rust-apps-test` | passed, 1 test (s19) |
| `chk-nsh-cli-tizen-device` | `cargo test -p nsh-cli --bin nsh test_host::tests::a_device_is_for_the_tizen_target_only` | `t19-rust-apps-test` | passed, 1 test (s19) |
| `chk-nsh-component` | `cargo test -p nsh-cli --bin nsh component::tests::tizen` | `t19-runtime` | passed, 1 test (s19) |
| `chk-nsh-cli-runtime` | `cargo test -p xtask --bin xtask tizen::tests` | `t19-runtime` | passed, 8 tests, 17:14 UTC (s19) |
| `chk-nsh-hostplatform` | `cargo test -p nsh-dev --lib protocol::tests::tizen` | `t19-runtime` | passed, 1 test, 16:14 UTC (s19) |
| the `tz` signing and install fix (`cfa18ec5b`) | `cargo test -p nsh-cli --bin nsh tizen::tests::tz_commands` (new, 2 tests), `tizen::tests` (14), `--test cli tizen::` (6), `cargo clippy -p nsh-cli --all-targets -- -D warnings` | `t9-publish` (a finding of `CI-3`'s emulator runs) | passed on `pr/tizen/07-cli`, 2026-11-11 |

s19's final check (`check-s19.json`, revision 321): 9 passed, 8 run and 1 reused (`chk-nsh-hostplatform`), the 17:14 UTC
runs on `feat/tizen` after `55f2d6e7f` (17:14 UTC commit, merged by `b91fab916`). s19's tasks are `done`, its review
passed (`review-s19-3.json`) and the sprint is checkpointed.

**On this branch alone** (no result above is from it): every command runs as written; the branch holds 05, 06 and
`pr/tizen-web/01-package`, whose code the tests use. Not run: the pre-open checks, `cargo xtask tizen` for all three
architectures (one armv7l build is reported in `ef365beaa`: 4m 48s, a 41 MB stripped release library).

## Updating and opening

After 05, 06 and `pr/tizen-web/01-package` have merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen-os
git fetch upstream origin
git tag epic/tizen/07-cli pr/tizen/07-cli
git reset --hard upstream/main
git cherry-pick 5cea54e4b 1ca68cd89 ef365beaa a38dcbeca 22c48fe78 fdcb28185 5c22de690 9244408df a5857a017 7cb4e3fb2 \
  55f2d6e7f
# (20404bb35 is left out: 04's 384758c3c is the same patch)
# the checks above and the pre-open checks, then (owner runs this)
git push --force-with-lease -u origin pr/tizen/07-cli
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/07-cli
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen/07-cli \
  --title "cli: Run, package and test apps natively on Samsung TVs" --body "$(sed '1,2d' PR.md)"
```
