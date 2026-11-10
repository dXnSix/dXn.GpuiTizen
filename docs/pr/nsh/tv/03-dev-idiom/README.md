# pr/tv/03-dev-idiom

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tv/03-dev-idiom`, head `2cb11014a`, local, checked out in `nsh-wt/tizen-engine`, not pushed |
| Base | `pr/tv/02-category` at its head (merged in by `af909ed26`), on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in 02's `crates/nsh-core/src/prelude.rs` and `crates/nsh-platform-web/src/browser/engine.rs`, and in `crates/nsh-cli/src/test_web.rs`. Not rebased |
| Depends on | 02 merged |
| Size | own: 32 files, +850 −54: `nsh-cli` (`main.rs`, `run.rs` +137, `dev.rs`, `package.rs`, `test.rs`, `test_host.rs`, `test_web.rs`, `web.rs`, `web/dev.html`, the new `tests/cli/dev_idiom.rs` +408), `nsh-platform-gpui/src/app.rs` (`AppOptions::idiom`, `idiom_tests`) and `lib.rs`, `nsh-gpui-host/src/lib.rs`, the new app spec `tests/apps/tv-idiom/` (`App.tsx`, `App.test.tsx`, `nsh.json`), `docs/dev/{gpui,tooling,web}.md`, the website's cli, gpui, platform-specific, project-structure, testing-and-trace and web pages (English and zh-CN). With 02: 69 files, +1805 −158 |
| Its commits | `694d31ba2` cli: Run apps in the TV idiom on the desktop; (merge `af909ed26` of 02); `35ce21821` cli: Run nsh test's apps in the TV idiom on the GPUI host and the web page; `2cb11014a` test: Add an app spec that runs in the TV idiom on the desktop |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `e7325eb31` for the last two commits) |
| Review findings | `f-s21-gpui-spec-not-tv-idiom` (major, from the CI review): `fork/tizen-ci`'s "TV idiom on the GPUI host" step ran `tests/apps/pressable-focus` in the desktop idiom, since `nsh test` had no `--idiom` and the test hosts never resolved one. `35ce21821` gives `nsh test --target gpui` and `--target web` `--idiom tv` or `desktop` over `nsh.json` `dev.idiom` (each test's GPUI host starts with `--idiom`, the web test page gets `IDIOM`; other targets and headless runs refuse the flag, and leave `nsh.json`'s setting out with a warning); `2cb11014a` adds `tests/apps/tv-idiom`, whose two tests fail in the desktop idiom. Resolved in `review-s21-2.json`, passed in `review-s21-3.json`. s16's review (`review-s16-2.json`, passed) covered `694d31ba2` with no finding |
| Needs | no nsh-compat change |

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

After 02 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tv/03-dev-idiom pr/tv/03-dev-idiom
git rebase --onto upstream/main pr/tv/02-category pr/tv/03-dev-idiom
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tv/03-dev-idiom
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tv/03-dev-idiom
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tv/03-dev-idiom \
  --title "cli: Run apps in the TV idiom on the desktop" --body "$(sed '1,2d' PR.md)"
```
