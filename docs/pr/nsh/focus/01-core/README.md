# pr/focus/01-core

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/focus/01-core`, head `ce993fd21`, local in `nsh-wt/tizen`, not pushed |
| Base | `upstream/main` `41100f2bf` (#1158, 2026-11-09). `upstream/main` is `c16a70096` (162 commits on); the branch merges onto it without conflict (`git merge-tree`). Not rebased |
| Depends on | nothing; the first of the `focus` stack (02, 03, 05 build on it) |
| Size | 8 files, +1353 −294: `crates/nsh-core/src/focus.rs` (+533) and `focus/tests.rs` (+464), `lib.rs` (+1); `nsh-platform-tui`'s `keyboard.rs` (+181 −286), `navigation.rs` (+2 −6), the new `tests/tui/focus_cell_size.rs` (+165); `docs/dev/tui.md` |
| Its commits | `ccbd31d9d` core: Decide directional focus moves in nsh_core::focus; `b34fd610e` tui: Take focus moves from nsh_core::focus; `ad514bb85` tui: Give the focus core its Cells one point each; `ce993fd21` core: Let focus frames be in any unit scaled per axis |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `173386011`; `feat/tizen` is at `7c5cd1c13`) |
| Review findings | none: the s15 review (`review-s15*.json`) reviewed the branches built on it, and raised nothing in `nsh_core::focus` itself |
| Needs | the focus issue ([../issue.md](../issue.md)) posted; no nsh-compat change; no behaviour change |

The TUI's rules move from its `keyboard.rs` (over Cells) into `nsh_core::focus` (over frames), and the TUI becomes its
first user with no change in behaviour: its `focus_navigation`, `modal_focus`, `keyboard`, `popup` and `navigation`
tests pass unchanged. `ad514bb85` gives the core each Cell as one point (the first conversion by Cell size missed by
a float on sizes like 8.4×16.8), and `ce993fd21` makes "frames in any unit scaled per axis" the module's contract.

## Before opening

- Squash or keep the four commits: the second and third are a fix of the first conversion; the maintainers squash
  anyway.
- `nsh_core::focus` is new public API in a published crate; its items are listed in `PR.md`.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-core` | `cargo test -p nsh-core --lib focus::tests` | `t14-focus-core` | passed, 13 tests, 2026-11-09 13:15 UTC (s14) |
| `chk-nsh-focus-tui` | `cargo test -p nsh-platform-tui --features dev,run-app --test tui -- focus_navigation:: modal_focus:: keyboard:: popup:: navigation:: focus_cell_size::` | `t14-focus-tui` | passed, 89 tests, 2026-11-09 13:15 UTC (s14) |

The s14 run (13:15 UTC) came after `ce993fd21` (13:13 UTC), so its 13 tests include the scale tests. Not run on this
branch alone, and not run: the pre-open checks ([../../README.md](../../README.md#pre-open-checks-every-branch)).

## Updating and opening

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/focus/01-core pr/focus/01-core
git rebase --onto upstream/main 41100f2bf pr/focus/01-core
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/focus/01-core
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/01-core
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/focus/01-core \
  --title "core: Decide directional focus moves in nsh_core::focus" --body "$(sed '1,2d' PR.md)"
```

Then put its number at the top of 02's, 03's and 05's `PR.md`.
