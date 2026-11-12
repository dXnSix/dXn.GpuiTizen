# pr/tv/02-category

| | |
|---|---|
| Upstream PR | ready, not opened: nothing goes to `longbridge/nsh` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Fork PR | none of its own: in dXnSix/nsh#196, dXnSix/nsh#201, dXnSix/nsh#202 (its stack tops'), into the fork's `main`, on the owner's go |
| Upstream branch | `up/tv/02-category` on `dXnSix/nsh`, head `666ddd4f2`, pushed 2026-11-12 (rebuilt from `40d27d136`): `upstream/main` `092438e52` (#1836); see below |
| Branch | `pr/tv/02-category`, head `fc8dc6ea8`, pushed to `dXnSix/nsh` |
| Base | `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `crates/nsh-core/src/prelude.rs` and `crates/nsh-platform-web/src/browser/engine.rs`. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `a8f7ad63c`: `fc8dc6ea8` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | nothing in NSH; nsh-compat [`tv-idiom`](../../compat/tv-idiom/) merged first (`platform android tv` in `tests/compat/cases/tv/script.txt`) |
| Size | 42 files, +957 −106: `nsh-core` (`platform.rs`, `element/ambient.rs`, `js/host.rs`, `prelude.rs`), `nsh-js` (`declarations.rs`, `dom.rs`, `env.rs` and the new `env/tests.rs` +131, `host.rs`), `nsh-platform-web` (`src/lib.rs`, `src/browser/engine.rs`, `fixtures.rs`, `web/browser/{bootstrap,dom,environment}.js`, the new `web/browser/tv.test.mjs` +237, `web/index.html`, `web/nsh.js`), `nsh-base`'s new `tests/base/tv_props.rs` (+183), the compat domain `tests/compat/cases/tv/` (new), `docs/dev/{api-naming,compatibility,components,elements,js-modules,native-pipeline,web}.md`, the website's device-and-environment, platform-specific, web, elements index and `nsh` module pages (English and zh-CN) |
| Its commits | `99ec7a52d` js: Add platform.tv, the tv prop group and the tv select key; `f2eccf9c3` web: Let the page set the TV idiom; `532345852` web: Check the TV idiom on the module itself in tv.test.mjs; `a8f7ad63c` js: Drop the host-prop platform `tv`, which no prop uses |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; held at its head by `pr/tv/03-dev-idiom`, `pr/tizen/01-os` (merge `5c1bfe2fe`) and `pr/tizen-web/01-package` (and through it `pr/tizen/07-cli` and `pr/tizen-web/03-module`); `pr/tizen/04-platform` (and 05, 06) through `pr/tizen/01-os` since `a892c2d4c` |
| Review findings | `f-s16-tv-web-test-reads-source` (minor): `tv.test.mjs` read the Rust source; `532345852` sends the page's settings to the module's own `nsh_environment` and checks what it sends back (`nsh_browser_test_environment`), run on the real browser-tests Wasm by `chk-nsh-tv-web`. `f-s16-tv-host-props-undeclared` (minor): `a8f7ad63c` drops `#[prop(tv = …)]`, `Platform::Tv` and its `declarations()` entry, which no prop used. Passed in the re-review (`review-s16-2.json`) |
| Needs | the TV issue answered: a TV is never desktop on any OS (a desktop OS in the TV idiom included), `tv` in `platform.select` after the OS's key |


## The upstream branch (2026-11-12)

`up/tv/02-category` holds this branch's own commits on `upstream/main` `092438e52` (#1836), cherry-picked (the `pr/*` branch, which carries merges of the
fork's `main`, stays as it is): `92742d65e` js: Add platform.tv, the tv prop group and the tv select key; `e914e6326` web: Let the page set the TV idiom; `27e51cb12` web: Check the TV idiom on the module itself in tv.test.mjs; `666ddd4f2` js: Drop the host-prop platform `tv`, which no prop uses.

Rebased from `40d27d136` (on `ca7e70e51`) without conflicts, so that `up/tv/03-dev-idiom` could be rebuilt on the newer `main`.

Checks on its tip `666ddd4f2` (`artifacts-temp/agentx/tizen-gpui-platform/t25-nsh/tv-02-category-final.log`): `env::tests::tv` 3 passed, `tv_props::` 2, the web module's `tv.test.mjs` 4, fmt, the `nsh-core`/`nsh-js`/`nsh-base` clippy, and `cargo xtask compat` with nsh-compat `up/tv-idiom` (compatible with every recorded release).

## Before opening

- `99ec7a52d` added a host-prop platform `tv` (`#[prop(tv = "…")]`, `Platform::Tv`, `TvHostProps`) and `a8f7ad63c`
  removed it again: squash them so the PR never shows it.
- `prelude.rs` conflicts with upstream's new exports (#1550 `data_dir`, …): keep both, add `is_tv`.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-tv-env` | `cargo test -p nsh-js --lib env::tests::tv` | `t16-category` | passed, 3 tests, 2026-11-10 12:17 UTC (s16) |
| `chk-nsh-tv-props` | `cargo test -p nsh-base --test base tv_props::` | `t16-category` | passed, 2 tests (s16) |
| `chk-nsh-tv-web` | `bash -c '$WASM_BUILD && NSH_BROWSER_WASM=target/wasm32-unknown-unknown/web/nsh_platform_web.wasm node --expose-gc --test crates/nsh-platform-web/web/browser/tv.test.mjs'` | `t16-category` | passed, 4 tests, 12:47 UTC (s16) |
| `chk-nsh-compat` | the compat run ([../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)) | `t16-compat` | passed, 13:00 UTC (s16) |

**On this branch alone** (no result above is from it): the three test commands run as written (`env::tests::tv` has the
same 3 tests there). `chk-nsh-compat` ran with the other branches' compat cases and nsh-compat's `nsh-tizen`; on this
branch, clone nsh-compat's `tv-idiom` in its place. Not run: the pre-open checks.

## Updating and opening

Nothing is opened before the owner's agreement with NSH's owners ([../../../README.md](../../../README.md#prepared-not-opened-2026-11-12)).
The branch that opens is `up/tv/02-category`, never `pr/tv/02-category`, which stays as the epic's records name it.
Before opening, rebuild `up/tv/02-category` on the newest `upstream/main` (or its parent's `up/` branch, once that has
merged upstream) as [../../README.md](../../README.md#updating-a-branch-and-opening-it) says, and run the checks
above on its tip. Then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git push --force-with-lease origin up/tv/02-category
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tv/02-category
gh pr create --repo longbridge/nsh --base main --head dXnSix:up/tv/02-category \
  --title "js: Add platform.tv, the tv prop group and the tv select key" --body "$(sed '1,2d' PR.md)"
```
