# core: Let views name where the arrows take the focus, with FocusGroup for TVs

Part of #(focus issue). **Depends on:** #(pr/focus/02-gpui), merged.

> [!NOTE]
> Needs longbridge/nsh-compat#(focus-props) ("compat: Report views' focus props and focus groups"), so that `prop`
> steps can check `autoFocus`, `preferredFocus`, the `nextFocus*` routes and `focusGroup`. Merge it first.

## Description

A TV is driven by a remote's arrows alone, so the nearest stop is not enough: a stop that takes the focus as it
appears or where the focus enters, the app's own routes between rows that do not line up, groups entered at chosen
stops that may keep the arrows inside, and lists whose next row is not mounted yet.

**Props.** `autoFocus` now applies to every element (a view takes the focus when it appears; an input's keeps its
meaning), `preferredFocus` makes a stop win where the focus enters, and `nextFocusUp` / `Down` / `Left` / `Right` send an
arrow to another view past geometry (a ref, as an overlay's anchor takes; it may name a view rendered later). The
refs are looked up again in every commit round, as `follow()` does; nothing runs while no view names one.

**`FocusGroup`** (`nsh/base`): a region the arrows treat as one, a shelf of tiles or a side menu. The focus enters it
at the first of its `destinations` in reach, else at its `preferredFocus` stop, else at the nearest; with `trap` the
arrows never leave it. Groups nest. It is a view carrying `Prop::FocusGroup`; its destinations may be its own
children (a `FocusTarget` takes `null` / `undefined` until their refs are set). Where the arrows do not move the focus
between views, it is a plain view.

**Rules.** `nsh_core::focus::directional` adds these over `horizontal` and `vertical`, on new `Candidate` data
(`auto_focus`, `preferred`, `next`, `group`, `scroller`) and `Group` descriptions; `auto_focus` and `first_stop` pick
the stop that takes the focus when a view or a scope appears. Without the new data it moves as before.

**Lists.** A list mounts only the rows near its viewport. When an arrow finds no stop inside the list around the
focus, the platform tells the engine (`EngineControl::focus_past`); the core works out the next row along the list
(`focus::next_row`: a row of columns in a grid, inverted and right-to-left lists the other way), brings it in at the
edge the arrow goes toward (`Command::ScrollToIndex`), and focuses that row's first stop once it is mounted
(`Command::Focus`). The row is forgotten when the focus is reported elsewhere first.

**GPUI** honours all of it: `preferredFocus` with nothing focused, the routes (a stop's own or a view's around it), the
groups, and a scroll view that keeps the focus while it can scroll on toward the arrow (it scrolls, and the engine is
told). `autoFocus` stops take the focus after the batch that brings them or a new focus scope, once each.

The website's "Arrow keys and TVs" section and the new `FocusGroup` page (English and zh-CN) say where the props apply:
the GPUI backend and the web (TVs included; the web's part is #(pr/focus/07-web-tv)).

## Public API

- TSX props on every element: `autoFocus?: boolean` (was `TextInput`'s only), `preferredFocus?: boolean`,
  `nextFocusUp` / `nextFocusDown` / `nextFocusLeft` / `nextFocusRight?: ViewRef | null`.
- Rust `Interactive`: `auto_focus(impl IntoReactive<bool>)`, `preferred_focus(impl IntoReactive<bool>)`,
  `next_focus_up(&ElementRef)` (and `_down`, `_left`, `_right`), `focus_group(trap, destinations)`.
- `nsh_core::element::FocusTarget` (a ref JS passes or a Rust `ElementRef`; `node()`), `FocusGroupTargets`.
- `nsh/base`: `FocusGroup` (`destinations?: FocusTarget[]`, `trap?: boolean`, children); Rust `focus_group()`,
  `FocusGroupProps`, `FocusGroupElement`.
- `nsh_core::props::ViewProps`: `auto_focus`, `preferred_focus`, `next_focus: NextFocus`, `focus_group:
  Option<GroupConfig>`.
- `nsh_core::focus`: `Direction`, `NextFocus`, `GroupConfig`, `Group`, `in_group`, `Target`, `directional`,
  `auto_focus`, `first_stop`, `Row`, `next_row`, `PendingRow`; `Candidate` gains `auto_focus`, `preferred`, `next`,
  `group`, `scroller`.
- `nsh_engine::EngineControl::focus_past(from, direction)` (and `Core::focus_past`).

## How to Test

- `cargo test -p nsh-core --lib focus::tests::tv` (the rules), `cargo test -p nsh-core --lib core::tests::directional_focus`
  (refs found in later commits, list rows scrolled to and focused, inverted, right-to-left and grid lists followed
  until they rest), `cargo test -p nsh-base --test base focus_group::`.
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib tv_focus` and `directional_focus`.
- `cargo xtask compat` with longbridge/nsh-compat#(focus-props): `props-view-focus` (the props, one route to a view
  rendered after the one naming it) and `base-focus-group` (trap and destinations following the app's state).
- The website's `el-focus-group` demo, on `nsh run --target gpui` with the arrows.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): `props-view-focus`, `base-focus-group`.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (the props apply on GPUI and the web: not applicable).
