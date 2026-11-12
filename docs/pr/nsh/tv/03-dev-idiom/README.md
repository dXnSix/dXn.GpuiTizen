# pr/tv/03-dev-idiom

| | |
|---|---|
| Upstream PR | ready, not opened: nothing goes to `longbridge/nsh` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Fork PR | dXnSix/nsh#196 (merge commit), into the fork's `main`, on the owner's go |
| Upstream branch | `up/tv/03-dev-idiom` on `dXnSix/nsh`, head `d29948463`, pushed 2026-11-12 (rebuilt from `54f305d94`): `up/tv/02-category`, on `upstream/main` `092438e52` (#1836); see below |
| Branch | `pr/tv/03-dev-idiom`, head `cc0685a75`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-engine` |
| Base | `pr/tv/02-category` at its head (merged in by `af909ed26`), on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in 02's `crates/nsh-core/src/prelude.rs` and `crates/nsh-platform-web/src/browser/engine.rs`, and in `crates/nsh-cli/src/test_web.rs`. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `2cb11014a`: `880e9dfb5`; then `cc0685a75` cli: Allow the test host's starter its options, one each ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 02 merged |
| Size | own: 32 files, +850 −54: `nsh-cli` (`main.rs`, `run.rs` +137, `dev.rs`, `package.rs`, `test.rs`, `test_host.rs`, `test_web.rs`, `web.rs`, `web/dev.html`, the new `tests/cli/dev_idiom.rs` +408), `nsh-platform-gpui/src/app.rs` (`AppOptions::idiom`, `idiom_tests`) and `lib.rs`, `nsh-gpui-host/src/lib.rs`, the new app spec `tests/apps/tv-idiom/` (`App.tsx`, `App.test.tsx`, `nsh.json`), `docs/dev/{gpui,tooling,web}.md`, the website's cli, gpui, platform-specific, project-structure, testing-and-trace and web pages (English and zh-CN). With 02: 69 files, +1805 −158 |
| Its commits | `694d31ba2` cli: Run apps in the TV idiom on the desktop; (merge `af909ed26` of 02); `35ce21821` cli: Run nsh test's apps in the TV idiom on the GPUI host and the web page; `2cb11014a` test: Add an app spec that runs in the TV idiom on the desktop |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `e7325eb31` for the last two commits) |
| Review findings | `f-s21-gpui-spec-not-tv-idiom` (major, from the CI review): `fork/tizen-ci`'s "TV idiom on the GPUI host" step ran `tests/apps/pressable-focus` in the desktop idiom, since `nsh test` had no `--idiom` and the test hosts never resolved one. `35ce21821` gives `nsh test --target gpui` and `--target web` `--idiom tv` or `desktop` over `nsh.json` `dev.idiom` (each test's GPUI host starts with `--idiom`, the web test page gets `IDIOM`; other targets and headless runs refuse the flag, and leave `nsh.json`'s setting out with a warning); `2cb11014a` adds `tests/apps/tv-idiom`, whose two tests fail in the desktop idiom. Resolved in `review-s21-2.json`, passed in `review-s21-3.json`. s16's review (`review-s16-2.json`, passed) covered `694d31ba2` with no finding |
| Needs | no nsh-compat change |


## The upstream branch (2026-11-12)

`up/tv/03-dev-idiom` holds this branch's own commits on `up/tv/02-category`, on `upstream/main` `092438e52` (#1836), cherry-picked (the `pr/*` branch, which carries merges of the
fork's `main`, stays as it is): `7e361387c` cli: Run apps in the TV idiom on the desktop; `ab8d01e06` cli: Run nsh test's apps in the TV idiom on the GPUI host and the web page; `f46079844` test: Add an app spec that runs in the TV idiom on the desktop; `d29948463` cli: Allow the test host's starter its options, one each.

Rebuilt on the newer `main`: #1813 (the test hosts run in the background) met this branch in `nsh-gpui-host/src/lib.rs` (both imports kept), `nsh-cli/src/test_host.rs` (the starter's doc says both; #1813's new test calls pass the `idiom` argument too) and the CLI pages (English, zh-CN: #1813's rows and the `--idiom` row). The test agent's Tab wrap (`48389b3f9`) is not here: it calls focus/06's `input::tab_past_ends`, so it moved to `up/focus/06-gpui-tab-stops`.

Checks on its tip `d29948463` (`artifacts-temp/agentx/tizen-gpui-platform/t25-nsh/tv-03-dev-idiom-final2.log`): `dev_idiom::` 8 passed, `app::idiom_tests` 1, `test_host::` 3, `tests/apps/tv-idiom --target gpui --idiom tv`, fmt, the `nsh-cli` and GPUI clippy. Not run here: the `tizen-web` check and the `pressable-focus` part (they need branches this one does not have).

## Before opening

- `35ce21821` and `2cb11014a` came after s16; they are this PR's too (the TV idiom for `nsh test`). If the maintainers
  prefer, open them as a second PR after this one.
- `tests/apps/pressable-focus` in the TV idiom passes its arrow steps (`--filter arrows`), not its Tab steps (one of
  four fails there: `s21-merge-pf-tv-gpui.log`); the spec is the desktops' (`pr/focus/03-desktop-stops`). Say so if
  the PR mentions it.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-dev-idiom` | `cargo test -p nsh-cli --test cli dev_idiom::` | `t16-dev-idiom` | passed, 4 tests, 2026-11-10 12:24 UTC (s16), before `35ce21821`'s four new tests; the 8 tests passed in `s21-idiom-test1.log` (18:09 UTC, the work that became `35ce21821`), not recorded as a check |
| `chk-nsh-dev-idiom-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib app::idiom_tests` | `t16-dev-idiom` | passed, 1 test, 12:42 UTC (s16) |
| `chk-nsh-tv-idiom-gpui` | `bash -c 'cargo build --manifest-path crates/gpui/Cargo.toml -p nsh-gpui-host && cargo run -p nsh-cli -- test tests/apps/tv-idiom --target gpui --idiom tv && cargo run -p nsh-cli -- test tests/apps/pressable-focus --target gpui --idiom tv --filter arrows'` (the `pressable-focus` part needs `pr/focus/02-gpui` and `03-desktop-stops`: on `feat/tizen` only) | `t21-nsh-ci` | passed, 3 tests, 18:28 UTC (s21) |
| `chk-nsh-tv-idiom-tizen-web` | `bash -c '$WEB_RUNTIME && NSH_RUNTIME="$PWD/target/nsh-runtime" cargo run -p nsh-cli -- test tests/apps/tv-idiom --target tizen-web'` (`$WEB_RUNTIME`: [../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)) (needs `pr/tizen-web/01-package`: on `feat/tizen` only) | `t21-nsh-ci` | passed, 2 tests, 18:30 UTC (s21) |

`--target web --idiom tv` ran by hand (`s21-tv-idiom-web-tv.log`, 2 passed; `s21-tv-idiom-web-desktop.log`, the desktop
idiom, fails both as it should).

**On this branch alone** (no result above is from it): `chk-nsh-dev-idiom` (8 tests there) and `chk-nsh-dev-idiom-gpui`
run as written. `chk-nsh-tv-idiom-gpui` does not: its last command runs `tests/apps/pressable-focus`, which is
`pr/focus/03-desktop-stops`'s, with the arrows moving GPUI's focus, which is `pr/focus/02-gpui`'s; on this branch run
its first part:

```bash
cargo build --manifest-path crates/gpui/Cargo.toml -p nsh-gpui-host && \
  cargo run -p nsh-cli -- test tests/apps/tv-idiom --target gpui --idiom tv
```

`chk-nsh-tv-idiom-tizen-web` needs `pr/tizen-web/01-package`'s `--target tizen-web`: no command runs it on this branch.
Not run: the pre-open checks.

## Updating and opening

Nothing is opened before the owner's agreement with NSH's owners ([../../../README.md](../../../README.md#prepared-not-opened-2026-11-12)).
The branch that opens is `up/tv/03-dev-idiom`, never `pr/tv/03-dev-idiom`, which stays as the epic's records name it.
Before opening, rebuild `up/tv/03-dev-idiom` on the newest `upstream/main` (or its parent's `up/` branch, once that has
merged upstream) as [../../README.md](../../README.md#updating-a-branch-and-opening-it) says, and run the checks
above on its tip. Then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git push --force-with-lease origin up/tv/03-dev-idiom
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tv/03-dev-idiom
gh pr create --repo longbridge/nsh --base main --head dXnSix:up/tv/03-dev-idiom \
  --title "cli: Run apps in the TV idiom on the desktop" --body "$(sed '1,2d' PR.md)"
```
