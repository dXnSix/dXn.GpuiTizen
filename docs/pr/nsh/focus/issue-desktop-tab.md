# Tab in tree order and the arrows moving the focus, on the desktop backends

> Draft for longbridge/nsh, not posted. Post only with the owner's go, before `pr/focus/03-desktop-stops` and
> `pr/focus/06-gpui-tab-stops` are opened (and before 02 and 05 for question 2). It changes what existing apps do,
> so it asks for the meaning to be recorded first. Part of the directional focus work ([issue.md](issue.md)).

**Is it intended that Tab and Shift+Tab on the desktop backends visit every keyboard stop (fields, views that take
presses, `focusable` views) in tree order, the same order on macOS, Windows, Linux, GPUI, the web and the headless
driver? And should the arrows move the focus by default?** A series on the way does both. Before it lands we would
like the meaning recorded, because it changes what existing apps do.

## What changes for Tab

| Backend | Before | After |
|---|---|---|
| macOS | fields and pressables, in AppKit's key view loop order (by place on screen) | fields, pressables, `focusable` views and hosted key views, in tree order |
| Linux | fields and pressables, in GTK's focus chain order (GTK sorts children by position) | the same stops, in tree order (the window's `move-focus` binding, after the app's bindings) |
| Windows | text fields only | fields that edit, pressables and `focusable` views, in tree order; Return / Space press the focused view; `focus()` / `blur()` on views work |
| GPUI | fields and native components | the arrows' stops (pressables, `focusable` views, fields that edit, native components), in tree order |

"Tree order" is the order the headless driver's `Screen::tab_stops` and the web already use: the view tree, with a
scroll view's (and a `screen_order` list's) content in the order it shows (`nsh_core::scroll::reading_order`). All go
through `nsh_core::input::tab_stop` (Windows and GPUI through `nsh_core::focus`).

Why ask:

- A layout whose screen order differs from its tree order (`row-reverse`, absolute placement, a view placed above an
  earlier sibling) is now visited in tree order on macOS and Linux, where it followed the screen. This matches the web
  (DOM order) and the headless tests apps already write, but an app tuned to the old order sees a different one.
- On Windows a page of buttons has keyboard stops for the first time; `focus:` styles and `onFocus` / `onBlur` fire
  there for the first time.
- On GPUI, Tab now stops at pressables too.

## What changes for the arrows

On GPUI, the web, Windows, macOS and Linux, the arrows move the focus to the nearest stop in their direction, through
`nsh_core::focus`, before they scroll: GPUI's and Windows' key handling ask the focus first, the web's `nsh.js` asks
`arrowFocus` before `scrollKey`, macOS and Linux run them after the app's bindings and the focused field's keys. With
nothing focused an arrow now focuses a stop (the first `preferredFocus` stop in sight where that prop applies, else
Left / Right the first stop and Up / Down the first or last in sight) where it used to scroll the page. Inside a scroll
view that can still scroll toward the arrow, GPUI and the web scroll first and focus the stop once it is in reach. An
arrow with no stop that way still scrolls.

It is not opt-in: a reading app that scrolled its page with the arrows now moves between its links and buttons.
`docs/dev/compatibility.md` (rule 4) would ask for a switch once a release records the old meaning.

## Options for the arrows

1. Always on, everywhere (TV remotes need it; a keyboard user gets the same model on every platform).
2. Move the focus only once a view has it (after Tab, a click on a field, or `focus()`), and scroll otherwise; on a TV
   (the TV idiom) always.
3. An app-level setting (`nsh.json` or a root prop), on by default in the TV idiom.

The branches implement 1. 2 and 3 are small changes in each backend's key handling.

## Proposed record

- Tab / Shift+Tab on every desktop backend: the stops `ViewProps::focus_stop` makes (plus fields that edit and native
  components that take the keyboard), in tree order, around, within the newest modal.
- The arrows: as the answer to the options above.

Checked with `tests/apps/pressable-focus` (`nsh test tests/apps/pressable-focus --target macos`, `--target linux` in a
Linux VM; Windows by clippy only so far) and GPUI's `focus::tab_stops` and `directional_focus` test groups.

The PRs: `pr/focus/03-desktop-stops` (macOS, Linux, Windows), `pr/focus/06-gpui-tab-stops` (GPUI's Tab); the
arrows' default also covers `pr/focus/02-gpui` and `pr/focus/05-web`.
