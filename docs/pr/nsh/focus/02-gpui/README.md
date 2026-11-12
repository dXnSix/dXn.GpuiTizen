# pr/focus/02-gpui

| | |
|---|---|
| Upstream PR | ready, not opened: nothing goes to `longbridge/nsh` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Fork PR | none of its own: in dXnSix/nsh#199, dXnSix/nsh#200 (its stack tops'), into the fork's `main`, on the owner's go |
| Upstream branch | `up/focus/02-gpui` on `dXnSix/nsh`, head `1829d9316`, pushed 2026-11-12: `up/focus/01-core`, on `upstream/main` `ca7e70e51` (#1789); see below |
| Branch | `pr/focus/02-gpui`, head `cf1919d5c`, pushed to `dXnSix/nsh` |
| Base | `pr/focus/01-core` (`ce993fd21`), on `upstream/main` `41100f2bf`. Against `upstream/main` `c16a70096` it conflicts in `crates/gpui/nsh-platform-gpui/src/input.rs`, `mount.rs` and `text_input.rs` (#1545, below). Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `a44434c87`: `cf1919d5c` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 01 (`pr/focus/01-core`) merged; then moved onto `upstream/main` alone |
| Size | own: 14 files, +1235 −23: `nsh-platform-gpui`'s new `focus.rs` (+664) and `focus/directional_focus.rs` (+500, the tests), hooks in `events.rs`, `input.rs`, `lib.rs`, `mount.rs`, `navigation.rs`, `surface.rs`, `text_input.rs`; `nsh-core/src/focus.rs` (+2 −1); `docs/dev/elements.md`, `docs/dev/gpui.md`, `website/docs/gpui.md`, `website/zh-CN/docs/gpui.md`. With 01: 21 files, +2587 −316 |
| Its commits | `602d87b9d` gpui: Move focus with the arrows through nsh_core::focus; `a44434c87` gpui: Take the focus off a focused view on blur() |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; also inside 04, 06 and 07 |
| Review findings | `f-s15-gpui-blur-command` (major): `blur()` on a pressable or `focusable` view that had GPUI's focus left it focused (`Command::Blur` only blurred fields). `a44434c87` adds `focus::blur_command` beside `focus::asked` in `Mounter::command`, with the test `blur_takes_the_focus_off_the_focused_view`. Passed in the re-review (`review-s15-2.json`) |
| Needs | the focus issue; the desktop issue's answer on arrows by default ([../issue-desktop-tab.md](../issue-desktop-tab.md)); no nsh-compat change |


## The upstream branch (2026-11-12)

`up/focus/02-gpui` holds this branch's own commits on `up/focus/01-core`, cherry-picked (the `pr/*` branch, which carries
merges of the fork's `main`, stays as it is): `1e156bbee` gpui: Move focus with the arrows through nsh_core::focus; `07a827030` gpui: Take the focus off a focused view on blur(); `1829d9316` gpui: Keep focus() on views, the pointer's blur and removed views in the focus module.

Upstream's GPUI keeps the keyboard focus on views in `key_focus.rs` (#1545, #1678 Return and Space before the keymap, #1795 removed views): the new commit `1829d9316` folds it into this branch's `focus.rs` (`focus_command`, `pointer_down`, `detached`, `on_control`), as the fork's merge did for #1545.

Checks on its tip `1829d9316` (logs under `artifacts-temp/agentx/tizen-gpui-platform/t25-nsh/`): `directional_focus` 11 passed, the GPUI tests with `shot` 240 (#1545's and #1795's included), fmt, the GPUI clippy.

## Before opening

- **Upstream #1545** ("gpui: Support programmatic focus on focusable views", `7b3e845af`, merged 2026-11-10, fixes
  #1411) added `crates/gpui/nsh-platform-gpui/src/key_focus.rs`: `Command::Focus` / `Command::Blur` on a focusable
  view that is no input, its `FOCUSED` state, `onFocus` / `onBlur`, reveal, and Return / Space pressing it. This
  branch's `focus.rs` gives such views their own focus too, for the arrows. When moving 02 onto `upstream/main`,
  build on `key_focus.rs` (one view focus, not two), then replace `PR.md`'s two `[OWNER: …]` markers (the text
  assumes that rework) with what the rebased branch adds on top of #1545; the `ref.focus()` part of 04
  (`e8de46ac9`) and 06's `b4a0181c9` (dropping the note that GPUI does not follow `focus()` on views) shrink with it.
- **Upstream #1519** (`focus-visible:` by input modality): the GPUI backend now reports key or pointer input; an arrow
  is a key, so check that a stop the arrows reach shows `focus-visible:`.
- `a44434c87` has no body; give it one or squash it into `602d87b9d`.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib directional_focus` | `t15-focus-gpui` | passed, 11 tests, 2026-11-10 10:41 UTC (s15) |

**On this branch alone** (no result above is from it): the command runs as written (11 tests there too), but the
recorded run had 04's and 06's changes to `focus.rs` beside 02's. Not run: the pre-open checks.

## Updating and opening

Nothing is opened before the owner's agreement with NSH's owners ([../../../README.md](../../../README.md#prepared-not-opened-2026-11-12)).
The branch that opens is `up/focus/02-gpui`, never `pr/focus/02-gpui`, which stays as the epic's records name it.
Before opening, rebuild `up/focus/02-gpui` on the newest `upstream/main` (or its parent's `up/` branch, once that has
merged upstream) as [../../README.md](../../README.md#updating-a-branch-and-opening-it) says, and run the checks
above on its tip. Then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git push --force-with-lease origin up/focus/02-gpui
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/02-gpui
gh pr create --repo longbridge/nsh --base main --head dXnSix:up/focus/02-gpui \
  --title "gpui: Move focus with the arrows through nsh_core::focus" --body "$(sed '1,2d' PR.md)"
```
