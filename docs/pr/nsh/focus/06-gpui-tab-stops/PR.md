# gpui: Stop Tab at pressables and focusable views, in tree order

Part of #(focus issue), as #(desktop issue) decided for the desktops. **Depends on:** #(pr/focus/04-tv), merged. No
nsh-compat change.

## Description

On the GPUI backend Tab and Shift+Tab moved between fields and native components only, while the arrows
(#(pr/focus/02-gpui)) reach every focus stop, and macOS, Linux and Windows now stop Tab at pressables in tree order
(#(pr/focus/03-desktop-stops)).

- Tab and Shift+Tab go through the stops the arrows do (`focus::tab_stops` over `stops_in` and
  `nsh_core::focus::candidates`): pressables, `focusable` views, fields that edit and native components that take the
  keyboard, in tree order through `nsh_core::input::tab_stop`, within the newest modal. A stop under a sheet's or a
  dialog's backdrop, or one with no area, is skipped.
- A view Tab reaches takes the view focus the arrows give (`FOCUSED`, `onFocus` / `onBlur`, the accessibility focus;
  Enter and Space press it) and is revealed in its scroll views.
- Past the last stop the window's focus moves on as before (`nsh_core::input::tab_stop` and `tab_to_edge` still decide
  the ends for embedding). When Tab moves the window's focus out of the view, or wraps around with no stop left, the
  stop that had the focus lets go of it, a view as a field does (`FOCUSED` off, `onBlur`).

`docs/dev/gpui.md` and the website's GPUI page (English and zh-CN) say so.

## Public API

None.

## Breaking Changes

No API breaks. What Tab does on GPUI changes: it now stops at pressables and `focusable` views, in tree order, as on
the other desktops (#(desktop issue)).

## How to Test

- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib focus::tab_stops`: the order, the
  stops skipped under a backdrop or with no area, a field as a stop when it edits, letting go past the ends.
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib directional_focus`: the arrows,
  unchanged.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): no API; Tab's behaviour as #(desktop issue) decided.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (GPUI only: not applicable).
