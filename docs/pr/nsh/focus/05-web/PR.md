# web: Move focus with the arrows through nsh_core::focus

Part of #(focus issue). **Depends on:** #(pr/focus/01-core), merged. No nsh-compat change.

## Description

In the browser the arrows only scrolled the scroll view around the focus or under the mouse, so a page driven by a
keyboard's or a TV remote's arrows could not reach its controls without Tab. The web now moves the focus with them by
`nsh_core::focus`'s rules, compiled into the module, as the terminal and GPUI backends do.

- `nsh.js` (`arrowFocus`) describes what the page renders: each focus stop's `getBoundingClientRect` in CSS pixels,
  whether some of it is in sight in its root view and the scroll views around it, its scope (the `Dialog` it is in,
  or the root), its layer, the anchored overlay around it (the props' new `anchored`) and whether it is a menu item;
  each root view's layers, what of them is in sight and whether they take presses; the modals and whether each shows.
- `nsh_focus_move` (`src/focus.rs`) picks the newest modal that shows, leaves out the layers under a backdrop that
  covers the page, keeps a focused menu item's arrows in its popup, and answers `focus::horizontal` / `vertical`'s
  stop.
- The page focuses it without the browser's scrolling and reveals it: each scroll view around it glides by
  `nsh_focus_reveal`'s delta (`focus::reveal`, centered vertically, nearest horizontally, as on the terminal). With no
  stop that way the key scrolls as before.
- Fields keep their arrows, and so do components that bind them in their keymap (shortcuts come first); Shift and the
  modifiers leave the arrows alone; `focus:` classes, `onFocus` / `onBlur` and Return pressing the focused pressable
  work as with Tab.

`src/focus.rs` depends on nothing of the browser, so its decisions are tested on the host.

## Public API

None for apps. The browser module exports `nsh_focus_move` and `nsh_focus_reveal` to `nsh.js`. Behaviour: on the web
the arrows now move the focus where a stop is in their direction, before they scroll (#(desktop issue)).

## How to Test

- `cargo test -p nsh-platform-web --test text focus_moves`: the decisions, on the host.
- Build the browser-tests module (`cargo build -p nsh-platform-web --target wasm32-unknown-unknown
  --no-default-features --features browser-tests --profile web`), then
  `NSH_BROWSER_WASM=target/wasm32-unknown-unknown/web/nsh_platform_web.wasm node --expose-gc --test crates/nsh-platform-web/web/browser/focus.test.mjs`:
  the page's description and how it applies the answer, on a fake DOM, with the module's own exports (without
  `NSH_BROWSER_WASM`, with a recording fake of the module).
- By hand: `nsh run --target web` on an app with buttons in rows; the arrows move between them, Return presses.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): no API; the arrows' default as #(desktop issue) decides.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (web only: the gallery in a browser instead).
