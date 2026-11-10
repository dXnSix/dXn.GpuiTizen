# web: End scrolls without scrollend

No issue (a fix for older engines). No nsh-compat change.

## Description

Engines without the `scrollend` event (Chromium before 114, such as the Chromium 108 of 2024's Samsung TVs) never end a
scroll the browser drives, so `onScrollEnd` and `onMomentumEnd` never fired for touch scrolls, their glides or the
browser's own smooth scrolls.

Where the window has no `onscrollend` (`SCROLLEND`), `nsh.js` now ends such a scroll once no scroll event came for
150 ms (`SCROLL_QUIET`): `scrollEnded` runs as the browser's `scrollend` would run it, a glide's `onMomentumEnd` first,
and a held scroll bar ends at its release. Engines with `scrollend` are unchanged.

`docs/dev/web.md` and the website's web page (English and zh-CN) say so.

## Public API

None.

## How to Test

- `node --expose-gc --test crates/nsh-platform-web/web/browser/scrollend.test.mjs`: with `scrollend` removed, a
  browser-driven scroll ends after the quiet interval, a glide reports `onMomentumEnd` first, a held scroll bar ends at
  its release; with `scrollend`, the browser's event ends the scroll, and only once.
- By hand: a scroll view with `onScrollEnd` in Chromium 108 (or a Samsung TV), with touch or the browser's smooth
  scrolling.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): nothing apps use changes.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (web only: the gallery's scroll demos in an old Chromium instead).
