# pr/focus/07-web-tv

| | |
|---|---|
| Upstream PR | not opened |
| Fork PR | dXnSix/nsh#200 (merge commit), into the fork's `main`, on the owner's go |
| Branch | `pr/focus/07-web-tv`, head `eb64d0a7f`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-web-tv` |
| Base | `pr/focus/04-tv` at its head (`a662232f0`, merged in by `f69c8f57c`) with `pr/focus/05-web` merged in (`0bd2092b9`); on `upstream/main` `41100f2bf`. Against `c16a70096` it has 04's and 05's conflicts (`nsh-platform-gpui`'s `input.rs`, `mount.rs`, `text_input.rs`; `nsh-core`'s `core/tests.rs`, `element/builder.rs`, `element/mod.rs`; `nsh-platform-web/web/nsh.js`; `docs/dev/web.md`; the website's web and view pages, English and zh-CN). Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `f69c8f57c`: `c85812a08`, `eb64d0a7f` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 04 and 05 merged |
| Size | own: 12 files, +984 −92 (against the 04 and 05 it holds): `nsh-platform-web`'s `src/focus.rs` (+318 −62), `src/lib.rs` (+44: `nsh_focus_auto`, `nsh_focus_past`), `src/encode.rs`, `web/nsh.js` (+183 −27), the new `tests/text/focus_moves/tv.rs` (+288), `focus_moves.rs`, `web/browser/focus.test.mjs` (+181); `docs/dev/web.md`, the website's web and `FocusGroup` pages (English and zh-CN). With 01, 02, 04 and 05: 67 files, +7512 −362 |
| Its commits | (merge `0bd2092b9` of 05); `6f396c490` web: Honor the TV's focus props and focus groups; (merge `f69c8f57c` of 04, whose conflict in `website/elements/index.md` and its zh-CN page took 04's wording of "Arrow keys and TVs") |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `01be268d4`) |
| Review findings | `f-s15-web-tv-rules` (minor): the web decided with `focus::horizontal` / `vertical` only, so on a `tizen-web` TV `preferredFocus`, `nextFocus*`, `FocusGroup`, view `autoFocus` and lists past their mounted rows did nothing. `6f396c490` sends the props (`focus::write_props`), decides with `focus::directional`, scrolls on and tells `nsh_focus_past`, and runs `nsh_focus_auto` once per stop, with `focus_moves::tv` and browser cases in `focus.test.mjs`. Passed in the re-review (`review-s15-2.json`) |
| Needs | no nsh-compat change (04's `focus-props` covers the props) |

## Before opening

- 07 holds 04 and 05 at their heads; its own change is `6f396c490` alone. When 04 and 05 have merged, move it by
  cherry-picking that commit. `6f396c490` still carries its older "Arrow keys and TVs" paragraph in
  `website/elements/index.md` (English and zh-CN): on `upstream/main` with 04 merged it conflicts there again; keep
  04's wording, as `f69c8f57c` did:

  ```bash
  git switch pr/focus/07-web-tv
  git tag epic/focus/07-web-tv pr/focus/07-web-tv
  git reset --hard upstream/main
  git cherry-pick 6f396c490          # website/elements/index.md (en, zh-CN): keep upstream's (04's) paragraph
  ```

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-web-core` | `cargo test -p nsh-platform-web --test text focus_moves` | `t15-focus-web` | passed, 14 tests (05's and 07's), 2026-11-10 10:39 UTC (s15) |
| `chk-nsh-focus-web` | `bash -c '$WASM_BUILD && NSH_BROWSER_WASM=target/wasm32-unknown-unknown/web/nsh_platform_web.wasm node --expose-gc --test crates/nsh-platform-web/web/browser/focus.test.mjs'` | `t15-focus-web` | passed, 14 tests (05's and 07's), 10:38 UTC (s15) |

**On this branch alone** (no result above is from it): both commands run as written; the branch holds 04 and 05 at their
heads, and its web focus files are `feat/tizen`'s. Not run: the pre-open checks.

## Updating and opening

After 04 and 05 have merged, the cherry-pick above, the checks and the pre-open checks, then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen-web-tv
git push --force-with-lease -u origin pr/focus/07-web-tv
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/07-web-tv
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/focus/07-web-tv \
  --title "web: Honor the TV's focus props and focus groups" --body "$(sed '1,2d' PR.md)"
```
