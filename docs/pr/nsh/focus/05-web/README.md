# pr/focus/05-web

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/focus/05-web`, head `0d23c931c`, local in `nsh-wt/tizen`, not pushed |
| Base | `pr/focus/01-core` (`ce993fd21`), on `upstream/main` `41100f2bf`. Against `c16a70096` it conflicts in `crates/nsh-platform-web/web/nsh.js`, `docs/dev/web.md`, `website/docs/web.md` and its zh-CN page. Not rebased |
| Depends on | 01 merged |
| Size | own: 10 files, +994 −10: `nsh-platform-web`'s new `src/focus.rs` (+189), `src/lib.rs` (+24: the module's exports), `src/encode.rs` (+5: a popup's `anchored`), `web/nsh.js` (+153), the new `tests/text/focus_moves.rs` (+234) and `web/browser/focus.test.mjs` (+359), `docs/dev/web.md`, the website's web page (English and zh-CN). With 01: 18 files, +2347 −304 |
| Its commits | `0d23c931c` web: Move focus with the arrows through nsh_core::focus |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; inside 07 |
| Review findings | none of its own. `f-s15-arrow-default-change` (minor: with nothing focused an arrow now focuses a stop where it scrolled, `arrowFocus` before `scrollKey` in `nsh.js`) is answered by the desktop issue ([../issue-desktop-tab.md](../issue-desktop-tab.md)), which names the web too |
| Needs | the desktop issue's answer on the arrows by default (the web does the same); no nsh-compat change |

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

On `feat/tizen` both files also hold 07's TV cases, so the counts are 05's and 07's together. Not run on this branch
alone; not run: the pre-open checks.

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
