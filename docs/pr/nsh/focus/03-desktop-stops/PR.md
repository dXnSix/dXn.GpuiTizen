# desktop: Stop Tab at pressables in tree order, and move focus with the arrows

Closes #(desktop issue: Tab in tree order and the arrows by default). Part of #(focus issue). **Depends on:**
#(pr/focus/01-core), merged. No nsh-compat change.

## Description

Tab and the arrows did different things on each desktop: macOS went through AppKit's key view loop and Linux through
GTK's focus chain, both ordered by place on screen; Windows stopped at text fields only; the arrows only scrolled. The
headless driver, the web and (with #(pr/focus/02-gpui)) GPUI go through the views in tree order and move the focus
with the arrows. The three desktops now do the same, through `nsh_core::focus`.

**Tab and Shift+Tab, in tree order** (a scroll view's content in the order it shows, as the headless driver's
`Screen::tab_stops` and the web), around from the last, inside the newest modal:

- macOS: `text_input::tab` takes the scope's stops (the views `is_key_stop` takes, the inputs' fields and text views,
  the first key view of each hosted AppKit view) and moves with `nsh_core::input::tab_stop`. The root's Tab and a
  field's go the same way.
- Linux: `key_focus::tab` takes the window's move-focus binding, after the app's bindings and the focused widget's
  keys, over the stops `is_stop` takes, each input's editor and each native component's first focusable widget. GTK's
  own chain still moves inside a native component. A field's blur is now reported before the next view's focus.
- Windows: views that are not fields get a focus of their own (`focus.rs`), and Tab moves through fields that edit,
  pressables and `focusable` views through `nsh_core::focus`. A field reached has its text selected as before;
  another stop reports `FOCUSED`, `onFocus` / `onBlur` and the accessibility focus and is scrolled into view; Return
  and Space press it. `focus()` and `blur()` on a view follow it; modals and layers give the focus back. Native
  components keep the keyboard, as before. The headless driver's Tab stops at views on Windows too.

**The arrows**, on all three, over the stops Tab goes through and their frames as they show: Left / Right to the
nearest on an overlapping row, Up / Down to the nearest below or above, an overlapping column first. With nothing
focused, Left / Right take the first stop and Up / Down the first or last in sight, where they used to scroll; a menu
item's Left / Right stay in its popup. The stop reached is scrolled into view; a field takes the typing. The arrows
come after the app's bindings, the focused field's own (Left / Right, and Up / Down when multiline) and selectable
text; one that moves no focus scrolls as before. On macOS a single-line field hands Up / Down over from `moveUp:` /
`moveDown:`; on Linux they run in the window's scroll key controller, before GTK's directional focus.

`tests/apps/pressable-focus` (a field, three pressables, two of them in a row laid out in reverse, and a view that is
no stop) checks Tab, Shift+Tab, the arrows and Return / Space with `nsh test` on each desktop. `docs/dev/` and the
website (forms and keyboard, the macOS, Linux and Windows pages, elements; English and zh-CN) say what Tab and the
arrows do.

## Public API

None.

## Breaking Changes

No API breaks, but what the keys do changes, as #(desktop issue) records:

- macOS and Linux: Tab visits the stops in tree order, no longer by place on screen (a `row-reverse` row, a view
  placed above an earlier sibling).
- Windows: Tab now stops at pressables and `focusable` views; `focus:` styles and `onFocus` / `onBlur` fire there.
- macOS, Linux and Windows: an arrow moves the focus to a stop in its direction, and with nothing focused it focuses
  one, before it scrolls.

## How to Test

- `cargo build -p nsh-macos-host && nsh test tests/apps/pressable-focus --target macos`, and the same with
  `--target linux` (`nsh-linux-host`) and `--target windows`.
- `cargo test -p nsh-platform-macos --lib focus::tests`, `cargo test -p nsh-platform-linux --lib key_focus::` (Tab
  order, the way around, Shift+Tab, blur before focus, the arrows).
- Windows clippy (`x86_64-pc-windows-msvc`, `-p nsh-platform-windows --all-targets`).

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): no API; the keys' behaviour as #(desktop issue) decided.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (desktops only: the gallery on macOS, Linux and Windows instead).
