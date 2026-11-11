# focus: the arrows move the keyboard focus, on every backend

Directional focus for NSH: the terminal backend's rules (which views are stops, where Left / Right and Up / Down go,
how a scroll view reveals the stop, where the focus goes back when a modal or a layer closes) move into
`nsh_core::focus`, decided over plain data every backend has; the TUI, GPUI, the three desktops and the web take
their moves from it; Tab stops at pressables in tree order on the desktops; and what a TV remote needs on top
(`autoFocus` on views, `preferredFocus`, `nextFocus*`, `FocusGroup`, lists scrolled to rows not mounted yet).

Two issues come first: [issue.md](issue.md) (the design and the TV props) and
[issue-desktop-tab.md](issue-desktop-tab.md) (Tab in tree order and the arrows moving the focus by default: a change
of recorded meaning on every desktop, the maintainers' call before 03 and 06 are opened).

## The PRs, in order

All pushed to `dXnSix/nsh` (2026-11-10), worked on in `nsh-wt/tizen`. Own size: the branch against the parents it holds; total: against
`41100f2bf`.

| # | Folder | Branch | Built on | Title | Own size | Needs | Status |
|---|---|---|---|---|---|---|---|
| 01 | [01-core/](01-core/) | `pr/focus/01-core` | `upstream/main` `41100f2bf` | core: Decide directional focus moves in nsh_core::focus | 8 files, +1353 −294 | the issue | not opened |
| 02 | [02-gpui/](02-gpui/) | `pr/focus/02-gpui` | 01 | gpui: Move focus with the arrows through nsh_core::focus | 14 files, +1235 −23 | 01 merged; the desktop issue (arrows by default) | not opened |
| 03 | [03-desktop-stops/](03-desktop-stops/) | `pr/focus/03-desktop-stops` | 01 | desktop: Stop Tab at pressables in tree order, and move focus with the arrows | 36 files, +2501 −170 | 01 merged; the desktop issue answered | not opened |
| 04 | [04-tv/](04-tv/) | `pr/focus/04-tv` | 02 | core: Let views name where the arrows take the focus, with FocusGroup for TVs | 45 files, +3091 −88 | 02 merged; nsh-compat `focus-props` | not opened |
| 05 | [05-web/](05-web/) | `pr/focus/05-web` | 01 | web: Move focus with the arrows through nsh_core::focus | 10 files, +994 −10 | 01 merged; the desktop issue (arrows by default) | not opened |
| 06 | [06-gpui-tab-stops/](06-gpui-tab-stops/) | `pr/focus/06-gpui-tab-stops` | 02, merges 04 | gpui: Stop Tab at pressables and focusable views, in tree order | 8 files, +258 −63 | 04 merged; the desktop issue answered | not opened |
| 07 | [07-web-tv/](07-web-tv/) | `pr/focus/07-web-tv` | 04 + 05 | web: Honor the TV's focus props and focus groups | 12 files, +984 −92 | 04 and 05 merged | not opened |

```
01-core ──┬── 02-gpui ──── 04-tv ──┬── 06-gpui-tab-stops (also merges 02)
          │                        └──┐
          ├── 05-web ─────────────────┴── 07-web-tv (merges 05 and the latest 04)
          └── 03-desktop-stops
```

Open 01 first. Once it merges, 02, 03 and 05 can be open at the same time (they touch different backends; each is
moved onto `upstream/main` alone). 04 waits for 02, 06 for 04, 07 for 04 and 05.

## Stack state (2026-11-10, 19:30 CET)

- **Every branch holds its parents at their heads.** 06 merges 02 (`8a1b3f284`) and the latest 04 (`def3cb5d5`);
  07 merges 05 (`0bd2092b9`) and, since 18:45, the latest 04 (`f69c8f57c`, which took 04's wording of the website's
  "Arrow keys and TVs" in `website/elements/index.md` and its zh-CN page). 07's own change is `6f396c490` alone. All
  seven are in `feat/tizen` (head `7c5cd1c13`).
- The heads of 01 … 06 have not moved since the s15 repairs; 07 moved by the merge only.
- Against `upstream/main` `c16a70096` (unchanged, 162 commits after `41100f2bf`; `git ls-remote` gives the same):
  01 merges cleanly; 02 conflicts in `nsh-platform-gpui`'s `input.rs`, `mount.rs`, `text_input.rs`; 03 in
  `docs/dev/elements.md`, `docs/dev/windows.md`; 04 and 06 in 02's three files and `nsh-core`'s `core/tests.rs`,
  `element/builder.rs`, `element/mod.rs`, `website/elements/view.md` (English and zh-CN); 05 in `web/nsh.js`,
  `docs/dev/web.md`, `website/docs/web.md` (English and zh-CN); 07 in 04's and 05's files together.
- Upstream since the base: #1545 (`7b3e845af`, 2026-11-10) gives GPUI's `ref.focus()` on focusable views in a new
  `key_focus.rs`, with Return / Space pressing the focused stop: the part of 02 and 04 that does the same must be
  folded onto it when 02 is moved (02, 04, 06 and 07 conflict in `nsh-platform-gpui`'s `input.rs`, `mount.rs`,
  `text_input.rs`). #1519 (`fe8add536`) makes `focus-visible:` follow the input modality, which every backend now
  reports: an arrow is a key, so a focus the arrows move should show `focus-visible:`; check it on each backend.
  #1551 (`a72f761a0`) moves `onOpen` of popups a commit later, so `focus()` there works; #1475 (`ad481faa4`)
  replaces Windows' text inputs with RichEdit, which 03's `text_input.rs` change merges with textually but must be
  rebuilt and checked against.

## The reviews and their repairs

s14 reviewed 01 alone: `review-s14.json` failed with `f-s14-cell-points-inexact` (major), `review-s14-2.json` with
`f-s14-frame-unit-contract` (minor), and `review-s14-3.json` passed; the repairs are `ad514bb85` and `ce993fd21`
([01-core/README.md](01-core/README.md)).

s15 reviewed 02 … 07 (`review-s15.json`, failed with 8 findings; re-review `review-s15-2.json` and
`review-s15-full.json`, passed at snapshot `a5295e00`, all 13 checks of `check-s15.json` passing). Where each finding
was repaired:

| Finding | Branch | What changed |
|---|---|---|
| `f-s15-desktop-arrows` (major) | 03 | the arrows move the focus on macOS (`07eec85a1`) and Linux (`48fed3f34`) too, not only on Windows; `tests/apps/pressable-focus` gained two arrow tests, run on macOS and Linux |
| `f-s15-gpui-blur-command` (major) | 02 | a ref's `blur()` on a view that has GPUI's focus takes it off (`a44434c87`, `focus::blur_command`) |
| `f-s15-gpui-tab-out-keeps-focus` (major) | 06 | Tab past the ends, or out of the view, lets go of the stop's focus (`7382ccca5`) |
| `f-s15-gpui-tab-stop-set` (minor) | 06 | Tab goes through the arrows' stops (`focus::tab_stops`: none under a backdrop, none without an area) (`7382ccca5`) |
| `f-s15-arrow-default-change` (minor) | the desktop issue | the arrows' new default on GPUI, Windows, macOS, Linux and the web is raised in [issue-desktop-tab.md](issue-desktop-tab.md) |
| `f-s15-docs-platform-claims` (minor) | 03, 04, 06 | the docs name where the arrows and the TV props apply (`a662232f0`, the desktops' `docs/dev` pages in 03, `b4a0181c9`) |
| `f-s15-web-tv-rules` (minor) | 07 | the web honours `preferredFocus`, `nextFocus*`, `FocusGroup`, view `autoFocus` and lists past their mounted rows (`6f396c490`) |
| `f-s15-list-index-coverage` (minor) | 04 | a list's next row comes in at the edge the arrow goes toward, inverted and right-to-left included (`8b3d335d3`); GPUI asks the engine through the surface, with a test (`b3a7934f0`) |

## Rules for this area

nsh-root's ([../README.md](../README.md#rules)): one PR at a time per stack line, each moved onto `upstream/main`
with only its own commits before it is opened; docs and compat cases in the same PR; the owner pushes and opens.
