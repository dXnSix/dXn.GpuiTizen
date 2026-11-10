# web: Honor the TV's focus props and focus groups

Part of #(focus issue). **Depends on:** #(pr/focus/04-tv) and #(pr/focus/05-web), merged. No nsh-compat change
beyond #(pr/focus/04-tv)'s.

## Description

The arrows on the web decided with `focus::horizontal` and `vertical` only, so a TV's web app ignored `autoFocus` on
views, `preferredFocus`, the `nextFocus*` routes, `FocusGroup` and the rows of a list past those mounted, which
#(pr/focus/04-tv) added and GPUI honours.

- `props_json` now sends a view's `autoFocus`, `preferredFocus`, `nextFocus*` and `focusGroup`
  (`focus::write_props`).
- `focusStops` (`nsh.js`) hands each stop those of the views around it, its innermost focus group and the scroll views
  around it, and describes the groups, the scroll views (axis, and whether each can scroll on toward its start and its
  end) and the stops inside each view a route or a destination names.
- `nsh_focus_move` decides with `focus::directional`, which is the earlier rules when none of those props is set.
  When the scroll view around the focus scrolls on instead, the page scrolls it by a key's step and tells
  `nsh_focus_past` (`EngineControl::focus_past`), so a list scrolls to its next row and the core focuses that row's
  first stop once mounted.
- After a batch that brought a view with `autoFocus`, or a new focus scope, `nsh_focus_auto` (`focus::auto_focus`)
  picks the stop that takes the focus, once per stop.

`docs/dev/web.md` and the website's web and `FocusGroup` pages (English and zh-CN) now name the web among the
platforms where the props apply; the elements index already does, from #(pr/focus/04-tv).

## Public API

None new for apps: the props of #(pr/focus/04-tv) now apply on the web. The browser module exports `nsh_focus_auto`
and `nsh_focus_past` to `nsh.js`.

## How to Test

- `cargo test -p nsh-platform-web --test text focus_moves` (the new `focus_moves::tv` cases: `preferredFocus`, routes,
  groups entered at their destinations and trapping, `autoFocus` once per stop, a scroll view that keeps the focus).
- The browser-tests module and
  `NSH_BROWSER_WASM=target/wasm32-unknown-unknown/web/nsh_platform_web.wasm node --expose-gc --test crates/nsh-platform-web/web/browser/focus.test.mjs`.
- By hand: the website's `el-focus-group` demo with `nsh run --target web`, driven by the arrows.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): #(pr/focus/04-tv)'s cases.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (web only: not applicable).
