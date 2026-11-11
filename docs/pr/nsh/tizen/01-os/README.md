# pr/tizen/01-os

| | |
|---|---|
| Upstream PR | not opened |
| Fork PR | none of its own: in dXnSix/nsh#201, dXnSix/nsh#202 (its stack tops'), into the fork's `main`, on the owner's go |
| Branch | `pr/tizen/01-os`, head `6bb7ee28d`, pushed to `dXnSix/nsh` |
| Base | `pr/tv/02-category` at its head (cut from it at `f2eccf9c3`, its last two commits merged in by `5c1bfe2fe`), on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `crates/nsh-core/src/prelude.rs` and `crates/nsh-platform-web/src/browser/engine.rs` (as `pr/tv/02-category`). Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `5c1bfe2fe`: `393c2809b`; then `6bb7ee28d` core: Merge the Tizen platform tests' imports as rustfmt does ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | `pr/tv/02-category` merged; nsh-compat [`tizen-platform`](../../compat/tizen-platform/) merged first (`platform tizen` in `tests/compat/cases/tizen/script.txt`) |
| Size | own: 28 files, +374 −80: `nsh-core` (`platform.rs` +144 with the `tizen` tests, `element/ambient.rs`, `element/env.rs`, `keymap.rs`), `nsh-js` (`declarations.rs`, `env.rs`, `env/tests.rs` +83), `nsh-platform-gpui` (`agent.rs`, `app.rs`, `keymap.rs`), `nsh-platform-tui/src/app.rs`, `Cargo.toml` and `crates/gpui/Cargo.toml` (`target_vendor = "tizen"` in `unexpected_cfgs`), the compat domain `tests/compat/cases/tizen/` (new), `docs/dev/{api-naming,compatibility,elements,js-modules,platforms}.md`, the website's device-and-environment, platform-specific and `nsh` module pages (English and zh-CN). With `pr/tv/02-category`: 55 files, +1303 −158 |
| Its commits | `6b69e7433` core: Add Os::Tizen; `8b3bee883` gpui: Report Os::Tizen in Tizen builds; (merge `5c1bfe2fe` of 02) |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merges `bfbddbe1e`, `272ed0179`); merged at its head into `pr/tizen/04-platform` (`da9128b19`, then `a892c2d4c`) |
| Review findings | `f-s17-js-tizen-tests-unchecked` (major): the four `env::tests::tizen` JS environment tests ran in no check; resolved with no source change by the new check `chk-nsh-os-tizen-js` (at least 4 tests). And from s16, `f-s16-tv5-os-tizen-unmet`: by amendment `Os::Tizen`'s compat case (`tests/compat/cases/tizen/`) is this branch's (NTZ-2), checked by `chk-nsh-compat-tizen`. Passed in `review-s17-2.json` |
| Needs | the Tizen issue ([../issue.md](../issue.md)) answered on `Os::Tizen` itself |

## Before opening

- It changes `Environment::default()`'s idiom from `Idiom::Phone` to the build OS's own (`Idiom::default_on`:
  `Desktop` on macOS, Windows and Linux builds, `Tv` on Tizen, `Phone` elsewhere). Embedders and tests that build an
  `Environment` with `..Environment::default()` on a desktop now start in the desktop idiom. `PR.md` lists it under
  Breaking Changes; check NSH's own tests for it after the rebase (they passed on `feat/tizen`).
- `Os` is not `#[non_exhaustive]`: a new variant breaks exhaustive matches in Rust code (NSH's own are updated).

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-os-tizen` | `cargo test -p nsh-core --lib platform::tests::tizen` | `t17-os` | passed, 4 tests, 2026-11-10 13:09 UTC (s17) |
| `chk-nsh-os-tizen-js` | `cargo test -p nsh-js --lib env::tests::tizen` (at least 4) | `t17-os` | passed, 4 tests, 14:28 UTC (s17) |
| `chk-nsh-tv-env` | `cargo test -p nsh-js --lib env::tests::tv` | `t16-category`, `t17-os` | passed, 3 tests, 13:09 UTC (s17) |
| `chk-nsh-compat-tizen` | the compat run ([../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)), nsh-compat `nsh-tizen` | `t17-os` | passed, 14:42 UTC (s17) |

**On this branch alone** (no result above is from it): the three test commands run as written. `chk-nsh-compat-tizen`
ran with the other branches' compat cases and nsh-compat's `nsh-tizen`; on this branch, clone nsh-compat's
`tizen-platform` (which holds `tv-idiom`, for 02's `tv` cases) in its place. Not run: the pre-open checks.

## Updating and opening

After `pr/tv/02-category` and nsh-compat's `tizen-platform` have merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tizen/01-os pr/tizen/01-os
git rebase --onto upstream/main pr/tv/02-category pr/tizen/01-os
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tizen/01-os
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/01-os
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen/01-os \
  --title "core: Add Os::Tizen" --body "$(sed '1,2d' PR.md)"
```
