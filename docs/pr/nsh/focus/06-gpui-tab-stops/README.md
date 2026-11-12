# pr/focus/06-gpui-tab-stops

| | |
|---|---|
| Upstream PR | ready, not opened: nothing goes to `longbridge/nsh` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Fork PR | dXnSix/nsh#199 (merge commit), into the fork's `main`, on the owner's go |
| Upstream branch | `up/focus/06-gpui-tab-stops` on `dXnSix/nsh`, head `0bea951a0`, pushed 2026-11-12: `up/focus/04-tv`, on `upstream/main` `ca7e70e51` (#1789); see below |
| Branch | `pr/focus/06-gpui-tab-stops`, head `5a21e0f54`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-gpui02` |
| Base | `pr/focus/02-gpui`, with `pr/focus/04-tv` merged in (`def3cb5d5`); both at their heads. On `upstream/main` `41100f2bf`; against `c16a70096` it has 04's conflicts. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `b4a0181c9`: `5a21e0f54` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 04 merged (which holds 02 and 01); the desktop issue answered (Tab on GPUI changes) |
| Size | own: 8 files, +258 −63: `nsh-platform-gpui`'s new `focus/tab_stops.rs` (+188, with its tests), `focus.rs`, `focus/directional_focus.rs`, `input.rs`, `app.rs`; `docs/dev/gpui.md`, the website's GPUI page (English and zh-CN). With 01, 02 and 04: 58 files, +5877 −408 |
| Its commits | `917c652c3` gpui: Stop Tab at pressables and focusable views, in tree order; (merge `8a1b3f284` of 02); `7382ccca5` gpui: Tab through the arrows' stops and let go of the focus past the ends; (merge `def3cb5d5` of 04); `b4a0181c9` docs: Drop the note that GPUI does not follow focus() on views |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes |
| Review findings | `f-s15-gpui-tab-out-keeps-focus` (major): when Tab moved the window's focus out of the view (`AppView::tab`, `leave`), only fields were blurred, so a pressable kept `FOCUSED` and never heard `onBlur`. `f-s15-gpui-tab-stop-set` (minor): Tab built its own stop list (stops under a backdrop, stops without an area, every enabled input). `7382ccca5` makes Tab go through `focus::tab_stops` (the arrows' `stops_in`) and lets go of the focus past the ends (`input::tab_past_ends`), with the tests `tab_skips_the_stops_under_a_backdrop` and `the_stop_tab_leaves_the_view_from_lets_go`; `b4a0181c9` drops the note that GPUI does not follow `focus()` on views (part of `f-s15-docs-platform-claims`). Passed in the re-review |
| Needs | no nsh-compat change; no API |


## The upstream branch (2026-11-12)

`up/focus/06-gpui-tab-stops` holds this branch's own commits on `up/focus/04-tv`, cherry-picked (the `pr/*` branch, which carries
merges of the fork's `main`, stays as it is): `0e10283ba` gpui: Stop Tab at pressables and focusable views, in tree order; `eaf4d2e05` gpui: Tab through the arrows' stops and let go of the focus past the ends; `0bea951a0` gpui: Wrap a test's Tab past the last stop around, as a window's FocusNext does.

Tab goes through the arrows' stops in `focus::tab_stops`, ordered by `nsh_core::input::tab_order` (#1616); the popover test passes over the trigger, a stop of this Tab, as in the fork; and it now carries the test agent's Tab wrap (`48389b3f9`, from `pr/tv/03-dev-idiom`), which needs `input::tab_past_ends`.

Checks on its tip `0bea951a0` (logs under `artifacts-temp/agentx/tizen-gpui-platform/t25-nsh/`): `focus::tab_stops` 6 passed, `directional_focus` 11, the GPUI tests with `shot` 254, fmt, the GPUI clippy.

## Before opening

- `up/focus/06-gpui-tab-stops` is built on `up/focus/04-tv` (which holds 02): its own commits are the three of its
  "upstream branch" section, the merges gone. Once 04 has merged upstream, rebuild it on the newest `upstream/main`
  by cherry-picking those three.
- `pr/focus/06-gpui-tab-stops`'s `b4a0181c9` (dropping the note that GPUI does not follow `focus()` on views) has no
  copy on the `up/` branch: upstream's text there no longer has the note, so the pick was empty.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-gpui-tab` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib focus::tab_stops` | `t15-focus-desktops` | passed, 6 tests, 2026-11-10 10:41 UTC (s15) |
| `chk-nsh-focus-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib directional_focus` | `t15-focus-gpui` | passed, 11 tests (s15) |

**On this branch alone** (no result above is from it): both commands run as written; the branch holds 02 and 04 at their
heads, and its `focus` module is `feat/tizen`'s. Not run: the pre-open checks.

## Updating and opening

Nothing is opened before the owner's agreement with NSH's owners ([../../../README.md](../../../README.md#prepared-not-opened-2026-11-12)).
The branch that opens is `up/focus/06-gpui-tab-stops`, never `pr/focus/06-gpui-tab-stops`, which stays as the epic's records name it.
Before opening, rebuild `up/focus/06-gpui-tab-stops` on the newest `upstream/main` (or its parent's `up/` branch, once that has
merged upstream) as [../../README.md](../../README.md#updating-a-branch-and-opening-it) says, and run the checks
above on its tip. Then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git push --force-with-lease origin up/focus/06-gpui-tab-stops
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/06-gpui-tab-stops
gh pr create --repo longbridge/nsh --base main --head dXnSix:up/focus/06-gpui-tab-stops \
  --title "gpui: Stop Tab at pressables and focusable views, in tree order" --body "$(sed '1,2d' PR.md)"
```
