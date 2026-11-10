# keymap: Name the media and TV remote keys

Part of #(TV issue). No nsh-compat change.

## Description

A TV remote has keys a keyboard does not: play / pause and the other media keys, the four color keys, channel up and
down, and Info. Apps need them as shortcuts, and today `nsh/keymap` cannot name them.

- `nsh_core::keymap::Key` gains `MediaPlayPause`, `MediaPlay`, `MediaPause`, `MediaStop`, `MediaRewind`,
  `MediaFastForward`, `MediaNext`, `MediaPrevious`, `ColorRed`, `ColorGreen`, `ColorYellow`, `ColorBlue`,
  `ChannelUp`, `ChannelDown` and `Info`, named `mediaplaypause`, `mediaplay`, `mediapause`, `mediastop`,
  `mediarewind`, `mediafastforward`, `medianext`, `mediaprevious`, `red`, `green`, `yellow`, `blue`, `channelup`,
  `channeldown` and `info` (the web's `mediatracknext`, `colorf0red`, … as aliases) and labelled in words ("Play/Pause",
  "Red", "Channel Up"). They edit no text, so their bindings fire in text inputs.
- Where the platforms report them: Windows' `VK_MEDIA_*` keys; Android's `KEYCODE_MEDIA_*`, `KEYCODE_PROG_*`,
  `KEYCODE_CHANNEL_*` and `KEYCODE_INFO`; the web's `KeyboardEvent.key` (`MediaPlayPause`, `ColorF0Red`,
  `ChannelUp`, `Info`, …), and on engines that report `Unidentified` (and, in the TV idiom, any key that is none of
  the named ones, such as `Pause` for keyCode 19) Samsung TVs' remote key codes (`tizen.tvinputdevice`); GPUI, the key
  names a Tizen platform gives it (`red`, `mediaplaypause`, `channelup`, `info`, Ecore's `xf86info` as an alias), by
  codes of the backend's own past `0xFF` so Windows' input code never takes them for a Windows key. GTK and menu
  shortcuts take none.
- macOS: a Mac's media keys reach the system (Now Playing) as system-defined events, never an app's key events, so the
  test agent cannot press them and says so, naming the key and macOS.

`website/modules/nsh.md` (English and zh-CN) lists the names; `docs/dev/` the mappings.

## Public API

- `nsh_core::keymap::Key`: 15 variants, above; their names in the `NAMED` table and the TS key names of `nsh/keymap`
  (`Keystroke` strings such as `"red"`, `"mod+channelup"`).

## Breaking Changes

None for apps: new names only. `Key` is not `#[non_exhaustive]`, so Rust code that matches it exhaustively needs an arm
for the new keys (NSH's own backends are updated here).

## How to Test

- `cargo test -p nsh-core --lib keymap::tests::tv`: names, aliases, labels, the platforms' codes.
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib -- keys::tests::tv keymap::tests::tv_keys`.
- `node --test --test-name-pattern='TV remote' crates/nsh-platform-web/web/nsh.test.mjs`: the DOM names and Samsung's
  key codes, in the TV idiom and outside it (a desktop's Pause/Break stays no media key).
- `cargo xtask compat`: `modules/mod-keymap-tv` (15 bindings, the labels, `key red` … `key info` firing).
- `cargo clippy -p nsh-platform-macos --all-targets --features dev -- -D warnings`.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): `modules/mod-keymap-tv`.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (Android maps the keys: an Android TV or a remote's media keys).
