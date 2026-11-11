# pr/tizen-web/01-package

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tizen-web/01-package`, head `0294d0204`, pushed to `dXnSix/nsh` |
| Base | `pr/tv/02-category` (head), with `pr/tv/01-keys` (head) merged in (`32bdc715b`); on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `nsh-cli`'s `package.rs` and `test_web.rs`, `crates/nsh-core/src/prelude.rs`, `crates/nsh-platform-web/src/browser/engine.rs`, `web/nsh.test.mjs`, the website's CLI and project-structure pages (English and zh-CN). Not rebased |
| Depends on | `pr/tv/01-keys` (Samsung's key codes in `nsh.js`) and `pr/tv/02-category` (`idiom: "tv"`) merged; the loader probe's results on a TV (`DEV-6`, [../README.md](../README.md#the-loader-probe-dev-6)) |
| Size | own: 35 files, +2863 −66: `nsh-cli` (the new `src/tizen_web.rs` +861, `tizen_web/dev.html`, `tizen_web/shim.js` +189 and `shim.test.mjs` +281, the new `tizen_web/preview-service.js` +62, the probe's `tizen_web/probe/` files with `csp.html`, `web.rs`, `web/dev.html`, `web/index.html`, `dev.rs`, `main.rs`, `package.rs`, `run.rs`, `test_host.rs`, `test_web.rs`, `Cargo.toml`, the new `tests/cli/tizen_web.rs` +718), `AGENTS.md`, `docs/dev/{compatibility,web}.md`, the website's CLI, platforms, project-structure and web pages (English and zh-CN). With `pr/tv/01` and `02`: 88 files, +4560 −199 |
| Its commits | (merge `32bdc715b` of `pr/tv/01-keys`); `5ef50b12e` cli: Package apps as Tizen web apps; `12834794f` cli: Run and test apps as Tizen web apps; `d74ce6fcd` cli: Probe how a TV loads a Tizen web app's files; `bfd4b7d9b` cli: Name the Tizen package id package in nsh.json; `6bfecf051` cli: Launch Tizen web apps by their package id; `6f0005235` cli: Open the Tizen development package to the dev server; `1fd94197c` cli: Probe CSP 'self' and Samsung's webapis on a TV; `0294d0204` cli: Declare Smart Hub's preview in Tizen web packages |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; merged at its head into `pr/tizen/07-cli` (`b126bf60f`) and `pr/tizen-web/03-module` (`a59cb3ada`) |
| Review findings | s20's (`review-s20.json` failed with 4, `review-s20-2.json` passed with a fifth withdrawn): `f-s20-preview-metadata` (major: no package declared Smart Hub's preview, so a TV never showed one) by `0294d0204`, `nsh.json` `tizen.smartHubPreview`: a URL writes the public preview's `use.preview` `endpoint_URL=<url>`, `"service"` a personal one (`bg_service`, `<tizen:service id="<package>.NshPreview">` with `service/preview.js`, the `application.launch` privilege, and the shim's `nshTizen.preview` naming the service), tested by `a_smart_hub_preview_is_declared_as_samsung_asks` and the shim's tests; `f-s20-preview-endpoint-value` (major) withdrawn, the `=` form is Samsung's; `f-s20-dev-package-policy` (minor: the development package's `<access>` left out the dev page's `ws://` socket, and `allow-navigation` named a port) by `6f0005235`, `<access origin="*" subdomains="true"/>` and navigation to the host alone; `f-s20-tz-run-id` (minor) by `6bfecf051`, `tz run -p` gets the package id for the app, the probe and the development package; `f-s20-probe-csp` (minor) by `1fd94197c`, the probe's `csp` row (`connect-src 'self'`, `probe/csp.html`) and `webapis` row |
| Needs | the Tizen issue; no nsh-compat change |

## Before opening

- `bfd4b7d9b` (no body) renames `nsh.json`'s `tizen.packageId` to `tizen.package`, which `5ef50b12e`'s message still
  calls `packageId`: squash it in, or give it a body.
- If the probe shows that a TV loads module scripts or Wasm differently, the package changes before the PR.
- Smart Hub's preview (both forms), the development package's policy, signing and `tz run -p` are unverified on a TV
  (`docs/dev/web.md`'s device checklist, `DEV-6`); tiles' `action_data` (`eden_resume`) does not reach the app yet,
  the documented limit. Say so in the PR.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-cli-tizen-web` | `cargo test -p nsh-cli --test cli tizen_web::` (at least 3) | `t20-package`, `t20-run-test` | passed, 8 tests, 2026-11-10 17:46 UTC (s20) |
| `chk-nsh-cli-tizen-web-unit` | `cargo test -p nsh-cli --bin nsh tizen_web::` | `t20-package` | passed, 7 tests, 17:46 UTC (s20) |
| `chk-nsh-tv-shim` | `node --test crates/nsh-cli/src/tizen_web/shim.test.mjs` (at least 2) | `t20-shim` | passed, 14 tests (with 03's), 17:46 UTC (s20) |
| `chk-nsh-test-tizen-web` | `bash -c '$WEB_RUNTIME && NSH_RUNTIME="$PWD/target/nsh-runtime" cargo run -p nsh-cli -- test examples/counter --target tizen-web'` (`$WEB_RUNTIME`: [../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)) | `t20-run-test` | passed, 1 test, 17:46 UTC (s20) |
| `chk-nsh-tv-idiom-tizen-web` | the same with `tests/apps/tv-idiom` (`pr/tv/03-dev-idiom`'s app, not on this branch: on `feat/tizen` only) | `t21-nsh-ci` | passed, 2 tests, 18:30 UTC (s21) |

The s20 runs (17:46 UTC) came after `0294d0204` (17:41 UTC) and `54f54bc36` (17:44 UTC).

**On this branch alone** (no result above is from it): `chk-nsh-test-tizen-web` and the two `cargo test` groups run as
written (8 and 7 tests there; the recorded runs had 03's changes to `tizen_web.rs` and `tests/cli/tizen_web.rs`, and
`pr/tizen/07-cli`'s to `tizen_web.rs`). `chk-nsh-tv-shim` runs as written but has 10 tests there: 4 of the recorded 14
are 03's. `chk-nsh-tv-idiom-tizen-web` cannot run: `tests/apps/tv-idiom` is `pr/tv/03-dev-idiom`'s, not on this branch;
here `chk-nsh-test-tizen-web` checks `--target tizen-web`. Not run: the pre-open checks, anything on a TV.

## Updating and opening

After `pr/tv/01-keys` and `pr/tv/02-category` have merged and the probe has run on a TV:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git switch pr/tizen-web/01-package
git tag epic/tizen-web/01-package pr/tizen-web/01-package
git reset --hard upstream/main
git cherry-pick 5ef50b12e 12834794f d74ce6fcd bfd4b7d9b 6bfecf051 6f0005235 1fd94197c 0294d0204
# the checks above and the pre-open checks, then (owner runs this)
git push --force-with-lease -u origin pr/tizen-web/01-package
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen-web/01-package
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen-web/01-package \
  --title "cli: Package, run and test apps as Tizen web apps" --body "$(sed '1,2d' PR.md)"
```
