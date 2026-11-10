# TVs: the remote's keys in nsh/keymap, and TVs as a category on any OS

> Draft for longbridge/nsh, not posted. Post only with the owner's go, before `pr/tv/01-keys` and
> `pr/tv/02-category`. The branches are built ahead of the answer and change in place if it differs.

NSH knows `Idiom::Tv` (Android and tvOS report it) but an app cannot do much with it: it cannot name a remote's keys,
cannot style for a TV, and on a desktop OS the TV idiom counts as desktop. We are bringing NSH to Samsung TVs, and
these parts are not Samsung's: they hold for Android TV, a TV's browser, and any TV after. Directional focus (the
arrows) is its own issue, #(focus issue).

## 1. The remote's keys

```tsx
import { keymap } from "nsh";
keymap.bind("red", () => toggleSubtitles());
keymap.bind("mediaplaypause", () => player.toggle());
keymap.label("channelup"); // "Channel Up"
```

`nsh_core::keymap::Key` would gain 15 keys: `MediaPlayPause`, `MediaPlay`, `MediaPause`, `MediaStop`, `MediaRewind`,
`MediaFastForward`, `MediaNext`, `MediaPrevious`, `ColorRed`, `ColorGreen`, `ColorYellow`, `ColorBlue`, `ChannelUp`,
`ChannelDown` and `Info`. Names `mediaplaypause` … `mediaprevious`, `red`, `green`, `yellow`, `blue`, `channelup`,
`channeldown`, `info`; the web's `KeyboardEvent.key` names (`MediaTrackNext`, `ColorF0Red`, …) as aliases; labels in
words. They edit no text, so their bindings fire in text inputs.

Mapped from Windows' `VK_MEDIA_*`, Android's `KEYCODE_MEDIA_*` / `PROG_*` / `CHANNEL_*` / `INFO`, the web's key names
and, where an engine reports `Unidentified` (or in the TV idiom, any unnamed key), Samsung TVs' key codes; and from the
names a Tizen platform gives GPUI. macOS cannot deliver media keys to an app (they go to Now Playing).

Tests press them with `app.keyboard.remote(key)` (the arrows, `ok`, `back` and the 15), which works headless and on
the terminal, GPUI, the web, Android and Linux, and fails clearly where the platform cannot press a key.

## 2. TVs as a category

```tsx
<View style={{ padding: 16 }} tv={{ style: { padding: 48 }, accessibilityLabel: "Shelf" }} />
const columns = platform.select({ tv: 5, desktop: 4, mobile: 2 });
if (platform.tv) { … }
```

- `platform.tv` (Rust `is_tv()`, `Environment::tv()`): the TV idiom, on any OS.
- `tv={{…}}` props on every element, applied only there, as `ios={{…}}` and `desktop={{…}}` are.
- `platform.select` looks for `tv` after the OS's own key and before `desktop` / `mobile`.
- **A TV is never desktop**, on any OS: `Idiom::desktop_on` is false for `Idiom::Tv`, a desktop OS included, so
  `desktop={{…}}` does not apply and `nsh/base` presents itself as on mobile.
- The web player's `idiom` option takes `"tv"`, so a page on a TV (a TV's web app) says so; the pointer heuristic
  cannot tell.
- `nsh run` / `nsh dev --idiom tv` (and `nsh.json` `dev.idiom`) run an app in the TV idiom on the GPUI host and the web
  dev page, so TV layouts are built on a Mac with hot reload.

## Questions

1. **The names**: `red` rather than `colorred` / `colorf0red`, `mediaplaypause` as one word, `info`, with the web's
   names as aliases. OK?
2. **Bindings in text inputs**: media and TV keys fire even while a field has the focus (they type nothing). OK?
3. **`Key` without `#[non_exhaustive]`**: adding variants breaks exhaustive Rust matches (NSH's own backends are
   updated). Acceptable, or make it `#[non_exhaustive]` first?
4. **Never desktop**: today a desktop OS is desktop "whatever idiom it was given". We would make the TV idiom the
   exception on every OS. No released backend reports the TV idiom on a desktop OS; with `--idiom tv` the GPUI host
   would.
5. **The select order**: `os` key, then `tv`, then `desktop` / `mobile`. For Samsung's own OS (#(tizen issue)) the OS
   key `tizen` comes before `tv`.
6. **`tv={{…}}` takes every element's props**, as no prop is a TV's alone; there are no TV-only host props.
7. **Compat**: the new `tv` domain runs on `platform android tv`, which nsh-compat learns in a small PR first.

## The PRs

| # | Branch | What | Needs |
|---|---|---|---|
| 1 | `pr/tv/01-keys` | the 15 keys, their names and mappings; compat `modules/mod-keymap-tv` | – |
| 2 | `pr/tv/02-category` | `platform.tv`, `tv={{…}}`, the `tv` select key, never desktop, the page's `idiom: "tv"`; compat domain `tv` | nsh-compat `platform <os> tv` |
| 3 | `pr/tv/03-dev-idiom` | `--idiom tv` for `nsh run` / `nsh dev` | 2 |
| 4 | `pr/tv/04-remote-keys` | `app.keyboard.remote(key)` | 1 |
