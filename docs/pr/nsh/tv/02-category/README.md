# pr/tv/02-category

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tv/02-category`, head `a8f7ad63c`, local in `nsh-wt/tizen`, not pushed |
| Base | `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `crates/nsh-core/src/prelude.rs` and `crates/nsh-platform-web/src/browser/engine.rs`. Not rebased |
| Depends on | nothing in NSH; nsh-compat [`tv-idiom`](../../compat/tv-idiom/) merged first (`platform android tv` in `tests/compat/cases/tv/script.txt`) |
| Size | 42 files, +957 −106: `nsh-core` (`platform.rs`, `element/ambient.rs`, `js/host.rs`, `prelude.rs`), `nsh-js` (`declarations.rs`, `dom.rs`, `env.rs` and the new `env/tests.rs` +131, `host.rs`), `nsh-platform-web` (`src/lib.rs`, `src/browser/engine.rs`, `fixtures.rs`, `web/browser/{bootstrap,dom,environment}.js`, the new `web/browser/tv.test.mjs` +237, `web/index.html`, `web/nsh.js`), `nsh-base`'s new `tests/base/tv_props.rs` (+183), the compat domain `tests/compat/cases/tv/` (new), `docs/dev/{api-naming,compatibility,components,elements,js-modules,native-pipeline,web}.md`, the website's device-and-environment, platform-specific, web, elements index and `nsh` module pages (English and zh-CN) |
| Its commits | `99ec7a52d` js: Add platform.tv, the tv prop group and the tv select key; `f2eccf9c3` web: Let the page set the TV idiom; `532345852` web: Check the TV idiom on the module itself in tv.test.mjs; `a8f7ad63c` js: Drop the host-prop platform `tv`, which no prop uses |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; held at its head by `pr/tv/03-dev-idiom`, `pr/tizen/01-os` (merge `5c1bfe2fe`) and `pr/tizen-web/01-package` (and through it `pr/tizen/07-cli` and `pr/tizen-web/03-module`); `pr/tizen/04-platform` (and 05, 06) through `pr/tizen/01-os` since `a892c2d4c` |
| Review findings | `f-s16-tv-web-test-reads-source` (minor): `tv.test.mjs` read the Rust source; `532345852` sends the page's settings to the module's own `nsh_environment` and checks what it sends back (`nsh_browser_test_environment`), run on the real browser-tests Wasm by `chk-nsh-tv-web`. `f-s16-tv-host-props-undeclared` (minor): `a8f7ad63c` drops `#[prop(tv = …)]`, `Platform::Tv` and its `declarations()` entry, which no prop used. Passed in the re-review (`review-s16-2.json`) |
| Needs | the TV issue answered: a TV is never desktop on any OS (a desktop OS in the TV idiom included), `tv` in `platform.select` after the OS's key |

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

After nsh-compat's `tv-idiom` has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tv/02-category pr/tv/02-category
git rebase --onto upstream/main 8c733731f pr/tv/02-category      # squash 99ec7a52d with a8f7ad63c's removal
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tv/02-category
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tv/02-category
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tv/02-category \
  --title "js: Add platform.tv, the tv prop group and the tv select key" --body "$(sed '1,2d' PR.md)"
```
