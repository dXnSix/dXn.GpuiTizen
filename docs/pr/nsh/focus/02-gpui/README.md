# pr/focus/02-gpui

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/focus/02-gpui`, head `a44434c87`, local in `nsh-wt/tizen`, not pushed |
| Base | `pr/focus/01-core` (`ce993fd21`), on `upstream/main` `41100f2bf`. Against `upstream/main` `c16a70096` it conflicts in `crates/gpui/nsh-platform-gpui/src/input.rs`, `mount.rs` and `text_input.rs` (#1545, below). Not rebased |
| Depends on | 01 (`pr/focus/01-core`) merged; then moved onto `upstream/main` alone |
| Size | own: 14 files, +1235 −23: `nsh-platform-gpui`'s new `focus.rs` (+664) and `focus/directional_focus.rs` (+500, the tests), hooks in `events.rs`, `input.rs`, `lib.rs`, `mount.rs`, `navigation.rs`, `surface.rs`, `text_input.rs`; `nsh-core/src/focus.rs` (+2 −1); `docs/dev/elements.md`, `docs/dev/gpui.md`, `website/docs/gpui.md`, `website/zh-CN/docs/gpui.md`. With 01: 21 files, +2587 −316 |
| Its commits | `602d87b9d` gpui: Move focus with the arrows through nsh_core::focus; `a44434c87` gpui: Take the focus off a focused view on blur() |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; also inside 04, 06 and 07 |
| Review findings | `f-s15-gpui-blur-command` (major): `blur()` on a pressable or `focusable` view that had GPUI's focus left it focused (`Command::Blur` only blurred fields). `a44434c87` adds `focus::blur_command` beside `focus::asked` in `Mounter::command`, with the test `blur_takes_the_focus_off_the_focused_view`. Passed in the re-review (`review-s15-2.json`) |
| Needs | the focus issue; the desktop issue's answer on arrows by default ([../issue-desktop-tab.md](../issue-desktop-tab.md)); no nsh-compat change |

## Before opening

- **Upstream #1545** ("gpui: Support programmatic focus on focusable views", `7b3e845af`, merged 2026-11-10, fixes
  #1411) added `crates/gpui/nsh-platform-gpui/src/key_focus.rs`: `Command::Focus` / `Command::Blur` on a focusable
  view that is no input, its `FOCUSED` state, `onFocus` / `onBlur`, reveal, and Return / Space pressing it. This
  branch's `focus.rs` gives such views their own focus too, for the arrows. When moving 02 onto `upstream/main`,
  build on `key_focus.rs` (one view focus, not two) and say so in `PR.md`; the `ref.focus()` part of 04
  (`e8de46ac9`) and 06's `b4a0181c9` (dropping the note that GPUI does not follow `focus()` on views) shrink with it.
- **Upstream #1519** (`focus-visible:` by input modality): the GPUI backend now reports key or pointer input; an arrow
  is a key, so check that a stop the arrows reach shows `focus-visible:`.
- `a44434c87` has no body; give it one or squash it into `602d87b9d`.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib directional_focus` | `t15-focus-gpui` | passed, 11 tests, 2026-11-10 10:41 UTC (s15) |

Not run on this branch alone; not run: the pre-open checks.

## Updating and opening

After 01 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/focus/02-gpui pr/focus/02-gpui
git rebase --onto upstream/main pr/focus/01-core pr/focus/02-gpui    # conflicts with #1545: see above
# the check above and the pre-open checks, then (owner runs this)
git push -u origin pr/focus/02-gpui
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/02-gpui
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/focus/02-gpui \
  --title "gpui: Move focus with the arrows through nsh_core::focus" --body "$(sed '1,2d' PR.md)"
```
