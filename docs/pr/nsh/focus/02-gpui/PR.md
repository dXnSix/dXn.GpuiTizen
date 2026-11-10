# gpui: Move focus with the arrows through nsh_core::focus

Part of #(focus issue). **Depends on:** #(pr/focus/01-core), merged. Builds on #1545's view focus. No nsh-compat
change.

## Description

The GPUI backend moved the keyboard focus between fields and native components with Tab only, so a pressable never
had it, and an app on a TV remote (whose arrows reach GPUI as `up`, `down`, `left` and `right`) could not be used
without a pointer.

The arrows now move the focus through `nsh_core::focus`, as the terminal backend's do:

- The stops are those of the focus scope: pressables, `focusable` views, fields that edit and native components that
  take the keyboard; inside the newest modal, less what a backdrop covers; with their frames in the window's points as
  drawn.
- Left / Right go along an overlapping row, Up / Down to the nearest below or above; with nothing focused an open
  anchored popup comes first; a menu item's Left / Right stay in its popup.
- The stop reached is scrolled into view as a focused field is, reports `FOCUSED` (`focus:` styles), `onFocus` /
  `onBlur` and the accessibility focus; Enter and Space press it when it takes presses. A ref's `blur()` takes the
  focus off a focused view.
- The arrows come after the app's bindings, the menus, the focused field (which keeps Left / Right, and Up / Down when
  multiline) and selectable text. An arrow that moves no focus scrolls as before.
- Modals blur and give back the focused view as they do a field; a layer that goes (a popup, a sheet) gives the focus
  back to the stop it came from.
- Tab is unchanged here (#(pr/focus/06-gpui-tab-stops) makes it stop at the same stops).

`docs/dev/gpui.md` and the website's GPUI page (English and zh-CN) describe it.

## Public API

None. Behaviour: on the GPUI backend the arrow keys now move the focus where a stop is in their direction, before
they scroll (#(desktop issue) asks whether that should stay the default on desktops).

## How to Test

- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib directional_focus` (GPUI's test app,
  no engine): rows, columns, presses, revealing in a scroll view, focus and blur with the focused state, fields,
  modals, menu popups, layers and backdrops, `blur()` on the focused view.
- By hand: `nsh run --target gpui` on an app with a row of buttons and a field; the arrows move between them, Enter
  presses.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): no API; the arrows' default is the open question of #(desktop issue).
- [ ] Ran the related `examples/gallery` demos on iOS and Android (GPUI only: not applicable).
