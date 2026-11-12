# pr/focus/05-web

| | |
|---|---|
| Upstream PR | ready, not opened: nothing goes to `longbridge/nsh` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Fork PR | none of its own: in dXnSix/nsh#200 (its stack tops'), into the fork's `main`, on the owner's go |
| Upstream branch | `up/focus/05-web` on `dXnSix/nsh`, head `9a20206bc`, pushed 2026-11-12: `up/focus/01-core`, on `upstream/main` `ca7e70e51` (#1789); see below |
| Branch | `pr/focus/05-web`, head `55b534f2c`, pushed to `dXnSix/nsh` |
| Base | `pr/focus/01-core` (`ce993fd21`), on `upstream/main` `41100f2bf`. Against `c16a70096` it conflicts in `crates/nsh-platform-web/web/nsh.js`, `docs/dev/web.md`, `website/docs/web.md` and its zh-CN page. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `0d23c931c`: `55b534f2c` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 01 merged |
| Size | own: 10 files, +994 −10: `nsh-platform-web`'s new `src/focus.rs` (+189), `src/lib.rs` (+24: the module's exports), `src/encode.rs` (+5: a popup's `anchored`), `web/nsh.js` (+153), the new `tests/text/focus_moves.rs` (+234) and `web/browser/focus.test.mjs` (+359), `docs/dev/web.md`, the website's web page (English and zh-CN). With 01: 18 files, +2347 −304 |
| Its commits | `0d23c931c` web: Move focus with the arrows through nsh_core::focus |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; inside 07 |
| Review findings | none of its own. `f-s15-arrow-default-change` (minor: with nothing focused an arrow now focuses a stop where it scrolled, `arrowFocus` before `scrollKey` in `nsh.js`) is answered by the desktop issue ([../issue-desktop-tab.md](../issue-desktop-tab.md)), which names the web too |
| Needs | the desktop issue's answer on the arrows by default (the web does the same); no nsh-compat change |


## The upstream branch (2026-11-12)

`up/focus/05-web` holds this branch's own commits on `up/focus/01-core`, cherry-picked (the `pr/*` branch, which carries
merges of the fork's `main`, stays as it is): `9a20206bc` web: Move focus with the arrows through nsh_core::focus.

Picked cleanly; its `focus.test.mjs` events gain `composedPath`, which `keyDown` reads since upstream's #1784.

Checks on its tip `9a20206bc` (logs under `artifacts-temp/agentx/tizen-gpui-platform/t25-nsh/`): `focus.test.mjs` 8 passed, `focus_moves` 7, fmt.

## Before opening

- Upstream #1519 reports the input modality from the web too (`focus-visible:`); an arrow is a key, so a stop the
  arrows reach should show `focus-visible:`: check in `focus.test.mjs`.
- Sixteen upstream commits since the base touch `nsh.js` (among them #1519 `focus-visible:`, #1547 / #1549
  `MODIFIER_KEYS`, #1482 the touch keyboard, #1567 `onWheel`, #1447 / #1448 the address history): keep both sides.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-web` | `bash -c '$WASM_BUILD && NSH_BROWSER_WASM=target/wasm32-unknown-unknown/web/nsh_platform_web.wasm node --expose-gc --test crates/nsh-platform-web/web/browser/focus.test.mjs'` (`$WASM_BUILD`: [../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)) | `t15-focus-web` | passed, 14 tests, 2026-11-10 10:38 UTC (s15) |
| `chk-nsh-focus-web-core` | `cargo test -p nsh-platform-web --test text focus_moves` | `t15-focus-web` | passed, 14 tests (s15) |

On `feat/tizen` both files also hold 07's TV cases, so the counts are 05's and 07's together.

**On this branch alone** (no result above is from it): both commands run as written, but the recorded runs did not test
05's code alone: they ran with 07's rewritten `src/focus.rs` (+318 −62), `web/nsh.js` and `focus.test.mjs` (+181) and
07's `focus_moves::tv`. On this branch `focus.test.mjs` has 8 tests and `focus_moves` 7. Not run: the pre-open checks.

## Updating and opening

After 01 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/focus/05-web pr/focus/05-web
git rebase --onto upstream/main pr/focus/01-core pr/focus/05-web
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/focus/05-web
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/05-web
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/focus/05-web \
  --title "web: Move focus with the arrows through nsh_core::focus" --body "$(sed '1,2d' PR.md)"
```
