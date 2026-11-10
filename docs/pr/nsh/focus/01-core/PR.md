# core: Decide directional focus moves in nsh_core::focus

Part of #(focus issue). The first of a series: the GPUI backend, the desktops and the web take their moves from this
module in their own PRs. No nsh-compat change.

## Description

The terminal backend is the only one that moves the keyboard focus with the arrows. Its rules (which views are stops
in the newest modal, which layers a backdrop takes out of reach, where Left / Right and Up / Down go, how a scroll
view reveals the stop, where the focus goes back when a modal or a layer closes) live in its `keyboard.rs`, over
Cells. Every backend that moves the focus with a keyboard's or a TV remote's arrows needs the same rules.

`nsh_core::focus` decides them over plain data every backend has, as `nsh_core::input` does for presses, hover and
wheel latching:

- the backend describes its stops (`Candidate`: id, frame, focus stop, input scope, layer, popup, tree order), the root
  view's layers (`Layer`) and the modals that show;
- the core answers the scope the focus moves in (`focus_scope`: the newest modal that shows), its stops in focus order
  less what a backdrop covers (`candidates`, `covering_layer`, `covered_by`), where Tab, Left / Right and Up / Down go
  (`tab`, `horizontal`, `vertical`), how far a scroll view moves to reveal the stop (`reveal`, nearest or centered),
  and which stop takes the focus back when a modal or a layer goes (`Modals`, `Layers`);
- the backend applies it: moves the focus, reports `focused` / `focus-within` and `onFocus` / `onBlur`, scrolls.

Frames are in the unit the backend lays out in, scaled along each axis on its own: every rule compares one axis at a
time and takes no length of its own, so the terminal can give the core each Cell as one point (8 × 16 points apiece
by default) and move the focus exactly as GPUI and the web will over points. `reveal` takes the layout's points.

The terminal backend is the first user: `keyboard.rs` describes the last frame's stops (each Cell one point, so frames
touch, overlap and sort as their Cells do) and applies what the core decides; `navigation::focus_scope` asks the core
for the newest modal. Its behaviour does not change.

## Public API

- `nsh_core::focus` (new module): `Order` (`view`, `link`), `Candidate` (`takes_focus`), `Layer`, `Align`, `Move`;
  `focus_scope(modals, shows) -> NodeId`, `covering_layer(layers)`, `covered_by(layers, own)`,
  `candidates(...)`, `tab(...)`, `horizontal(...)`, `vertical(...)`,
  `reveal(start, len, viewport, align) -> Option<Points>`; `Modals` (`update`, `first_stop`) and `Layers` (`entered`,
  `restore`, `retain`). Structs are `#[non_exhaustive]`.

## How to Test

- `cargo test -p nsh-core --lib focus::tests`: each rule on frames in points, and the same decisions at 1 × 1, 8 × 16,
  7.3 × 15.3 and 15.3 × 7.3 per axis (candidates, Tab, horizontal and vertical with nothing focused, with and without a
  popup, under a covering layer), a centered reveal scaling with its lengths.
- `cargo test -p nsh-platform-tui --features dev,run-app --test tui -- focus_navigation:: modal_focus:: keyboard:: popup:: navigation:: focus_cell_size::`:
  the TUI's existing groups unchanged, and `focus_cell_size` (a card and a button at 8 × 16, 8.4 × 16.8 and
  7.3 × 15.3, also with the card partly above the surface).

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): nothing apps use changes.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (terminal only: not applicable).
