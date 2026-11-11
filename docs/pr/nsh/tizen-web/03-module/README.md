# pr/tizen-web/03-module

| | |
|---|---|
| Upstream PR | not opened |
| Fork PR | dXnSix/nsh#202 (merge commit), into the fork's `main`, on the owner's go |
| Branch | `pr/tizen-web/03-module`, head `ff4735988`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-web` |
| Base | `pr/tizen/06-module` (head, merged in last by `661d69f2d`) and `pr/tizen-web/01-package` (head, merged in by `47a342f0d`, `b28080d75`, `a59cb3ada`); on `upstream/main` `8c733731f`. Against `c16a70096` it has both parents' conflicts (`.github/workflows/gpui.yml`, `nsh-platform-gpui`'s `app.rs` and `dev.rs`, `nsh-cli`'s `package.rs` and `test_web.rs`, `crates/nsh-core/src/prelude.rs`, `crates/nsh-platform-web/src/browser/engine.rs`, `web/nsh.test.mjs`, the website's CLI and project-structure pages and `website/zh-CN/docs/gpui.md`). Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `445c2e43e`: `4c148d064`, `7df59b2b7`, `ff4735988` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | `pr/tizen/06-module` (`nsh/tizen`) and `pr/tizen-web/01-package` (the shim, `tizen.smartHubPreview` and the preview service) merged |
| Size | own: 14 files, +427 −26 (against the 06 and 01 it holds): the new `crates/nsh-platform-web/web/browser/tizen.js` (+87) and `tizen.test.mjs` (+123), `web/browser/runtime.js`, `nsh-cli`'s `tizen_web.rs`, `tizen_web/shim.js` (+74 −3) and `shim.test.mjs` (+75), `web.rs` (`Variant::device_scripts`), `tests/cli/tizen_web.rs`; `docs/dev/{js-modules,web}.md`, the website's web and `nsh/tizen` pages (English and zh-CN). With everything below it: 150 files, +9418 −381 |
| Its commits | (merge `47a342f0d`); `3be0df576` web: Serve nsh/tizen in Tizen web apps from the TV's web APIs; (merges `b28080d75`, `a59cb3ada`); `54f54bc36` web: Set a Tizen web app's Smart Hub preview through its preview service; (merges `960323468`, `661d69f2d`) |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merges `189a8c56b`, `acc42e2a6`) |
| Review findings | `f-s20-preview-metadata` (major), its web half: `setPreviewData` called `webapis.preview.setPreviewData` from the page, which Samsung takes only from a background service of the package. `54f54bc36` makes the shim launch the package's preview service (`nshTizen.preview.service`, `tizen.application.launchAppControl` with the data under `preview`, which 01's service hands to `webapis.preview`); without a personal preview, or with a public one, the call rejects with `ERR_FEATURE_UNAVAILABLE_ON_PLATFORM` and says why. Tested in `shim.test.mjs`; passed in `review-s20-2.json` |
| Needs | no nsh-compat change |

## Before opening

- Its own commits are `3be0df576` and `54f54bc36`; after 06 and 01 have merged, cherry-pick them onto
  `upstream/main`.
- Smart Hub's personal preview through the service is unverified on a TV (`DEV-6`): when Smart Hub starts the service
  with no data, the service only exits (the s20 re-review's note for the device checklist).

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-tizen-module-web` | `node --test crates/nsh-platform-web/web/browser/tizen.test.mjs` | `t20-shim` | passed, 6 tests, 2026-11-10 17:46 UTC (s20) |
| `chk-nsh-tv-shim` | `node --test crates/nsh-cli/src/tizen_web/shim.test.mjs` | `t20-shim` | passed, 14 tests (01's and 03's), 17:46 UTC (s20) |
| `chk-nsh-cli-tizen-web` | `cargo test -p nsh-cli --test cli tizen_web::` | `t20-package`, `t20-run-test` | passed, 8 tests, 17:46 UTC (s20) |

The s20 runs came after `54f54bc36` (17:44 UTC).

**On this branch alone** (no result above is from it): every command runs as written; the branch holds 01 and
`pr/tizen/06-module` at their heads. Not run: the pre-open checks, anything on a TV.

## Updating and opening

After `pr/tizen/06-module` and `pr/tizen-web/01-package` have merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen-web
git fetch upstream origin
git tag epic/tizen-web/03-module pr/tizen-web/03-module
git reset --hard upstream/main
git cherry-pick 3be0df576 54f54bc36
# the checks above and the pre-open checks, then (owner runs this)
git push --force-with-lease -u origin pr/tizen-web/03-module
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen-web/03-module
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen-web/03-module \
  --title "web: Serve nsh/tizen in Tizen web apps from the TV's web APIs" --body "$(sed '1,2d' PR.md)"
```
