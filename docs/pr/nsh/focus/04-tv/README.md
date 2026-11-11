# pr/focus/04-tv

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/focus/04-tv`, head `a662232f0`, pushed to `dXnSix/nsh` |
| Base | `pr/focus/02-gpui` (`a44434c87`, merged in by `a7be4eb01`), on `upstream/main` `41100f2bf`. Against `c16a70096` it conflicts in `nsh-platform-gpui`'s `input.rs`, `mount.rs`, `text_input.rs` (02's, #1545), `nsh-core/src/core/tests.rs`, `element/builder.rs`, `element/mod.rs`, `website/elements/view.md` and its zh-CN page. Not rebased |
| Depends on | 02 merged (and so 01); nsh-compat [`focus-props`](../../compat/focus-props/) merged first |
| Size | own: 45 files, +3091 −88: `nsh-core` (`focus.rs` +367 and its tests +303, `core/directional.rs` +274 new, `core/mod.rs`, `core/list.rs`, `core/tests.rs` +487, `element/builder.rs`, `element/element_ref.rs`, `props.rs`, `ops.rs`, `js/host_values.rs`), `nsh-engine/src/lib.rs`, `nsh-base`'s `focus_group.rs` (new) and its tests, `nsh-platform-gpui` (`focus.rs`, the new `focus/tv_focus.rs` +310, `scroll.rs`, `mount.rs`, `surface.rs`), compat cases `props/props-view-focus.tsx`, `base/base-focus-group.tsx`, docs (`AGENTS.md`, `docs/dev/{api-naming,components,elements,gpui}.md`) and the website (the new `FocusGroup` page and demo, elements index, view, nsh-base, gpui; English and zh-CN). With 01 and 02: 56 files, +5626 −352 |
| Its commits | `252e0630a` core: Decide the TV's focus moves in nsh_core::focus; `e24810ce0` core: Let views name where the arrows take the focus; `e7b2d04ec` core: Scroll a list to the row the arrows reach before it is mounted; `ab1a3275d` focus_group: Add FocusGroup, a region the arrows enter at chosen stops; `d491ca17b` core: Focus the first stop of the list row the arrows reach; `e8de46ac9` gpui: Honor the TV's focus props, focus groups and list rows; `4485180bb` core: Let a focus group's destinations hold refs not set yet; `9476e21d5` compat: Check the TV's focus props and FocusGroup; `6fcc13829` focus_group: Name FocusGroup in the layout of AGENTS.md; (merge `a7be4eb01` of 02); `8b3d335d3` core: Bring a list's next row in at the edge the arrow goes toward; `b3a7934f0` gpui: Tell the engine through the surface when an arrow scrolls past a list's rows; `a662232f0` docs: Say where the arrows and the TV focus props apply |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; inside 06 and 07 at its head |
| Review findings | `f-s15-list-index-coverage` (minor): `Core::focus_past` was tested on a plain vertical list only and GPUI's path to the engine not at all. `8b3d335d3` brings a list's next row in at the edge the arrow goes toward (inverted and right-to-left lists included, with tests in `core::tests::directional_focus`), `b3a7934f0` sends GPUI's `Target::Scroll` through `Surface::focus_past` with the test `an_arrow_past_a_list_s_mounted_rows_asks_the_engine_for_the_next`. `f-s15-docs-platform-claims` (minor): `a662232f0` names where the arrows move the focus (macOS, Windows, Linux, the web, GPUI, TVs) and where the TV props apply (GPUI and the web) on the website's elements index (English and zh-CN) and in `docs/dev`. Both passed in the re-review. The s16 amendment of `f-s16-tv5-os-tizen-unmet` counts 04's compat cases `props/props-view-focus` and `base/base-focus-group` for TV-5 |
| Needs | the focus issue answered on the TV props' names and on `autoFocus` widening to every element |

## Before opening

- The size: core rules, props, `FocusGroup`, lists and the GPUI part in one PR, as nsh-root's rule asks (core plus each
  platform plus compat). If the maintainers ask for a split: the core and props (`252e0630a` … `d491ca17b`,
  `4485180bb`, `8b3d335d3`) with the compat cases, then GPUI (`e8de46ac9`, `b3a7934f0`).
- With #1545 upstream, `e8de46ac9`'s "Command::Focus on a stop that is no input moves the focus to it" is already
  upstream's: keep only what the arrows add.
- `autoFocus` moves from `TextInput` to every element (an input's keeps its meaning): `tests/compat` has
  `props-view-focus` for it; the props are documented as applying on GPUI and the web (TVs included).

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-tv-core` | `cargo test -p nsh-core --lib focus::tests::tv` | `t15-focus-tv` | passed, 11 tests, 2026-11-10 10:36 UTC (s15) |
| `chk-nsh-focus-tv-directional` | `cargo test -p nsh-core --lib core::tests::directional_focus` | `t15-focus-tv` | passed, 11 tests (s15) |
| `chk-nsh-focus-group` | `cargo test -p nsh-base --test base focus_group::` | `t15-focus-tv` | passed, 3 tests (s15) |
| `chk-nsh-focus-gpui-tv` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib tv_focus` | `t15-focus-tv` | passed, 8 tests, 10:41 UTC (s15) |
| `chk-nsh-focus-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib directional_focus` | `t15-focus-gpui` | passed, 11 tests (s15) |
| `chk-nsh-compat` | the compat run in [../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need) (nsh-compat `nsh-tizen`, which holds `focus-props`) | `t16-compat` | passed ("compatible with every recorded release"), 2026-11-10 13:00 UTC (s16) |

**On this branch alone** (no result above is from it): the five `cargo test` commands run as written
(`chk-nsh-focus-gpui`'s recorded run had 06's changes to `focus.rs` beside 04's). `chk-nsh-compat` ran with the other
branches' compat cases and nsh-compat's `nsh-tizen`; on this branch, clone nsh-compat's `focus-props` in its place. Not
run: the pre-open checks.

## Updating and opening

After 02 has merged and nsh-compat's `focus-props` PR has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/focus/04-tv pr/focus/04-tv
git rebase --onto upstream/main pr/focus/02-gpui pr/focus/04-tv
# the checks above and the pre-open checks (cargo xtask compat against nsh-compat main), then (owner runs this)
git push -u origin pr/focus/04-tv
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/04-tv
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/focus/04-tv \
  --title "core: Let views name where the arrows take the focus, with FocusGroup for TVs" \
  --body "$(sed '1,2d' PR.md)"
```
