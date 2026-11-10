# test: Press TV remote keys in app tests

Part of #(TV issue). **Depends on:** #(pr/tv/01-keys), merged. No nsh-compat change.

## Description

TV apps are driven by a remote, not a keyboard. App tests can now press a remote's key by name:

```tsx
await app.keyboard.remote("down");
await app.keyboard.remote("ok");
await app.keyboard.remote("mediaplaypause");
await app.keyboard.remote("back");
```

- The names: the arrows, `enter` (`ok`), `back`, and the media and TV keys `nsh_core::keymap` names
  (`mediaplaypause` … `info`). A key reaches the app as `keyboard.press` of the same name does; `back` goes back as a
  TV's Back does, the app's `onBackPress` first, and does nothing with nothing to go back from. An unknown name fails
  with the names there are.
- The step is `TestStep::PressRemoteKey`. Headless runs press the key as `keyboard.press` does; a real host's agent
  asks `Native::press_remote_key`, whose default presses the key's name (`press_keys`) or goes back.
- GPUI presses Back as a Tizen platform reports it (`VK_BROWSER_BACK`, `agent::remote_virtual_key`); the web agent
  sends the media and TV keys with Samsung TVs' key codes, as a TV's browser does (`test_web::tv_key_event`, held to
  the codes `nsh.js` reads); the terminal's agent names them as `nsh_core::keymap` does (`red`, `mediaplaypause`),
  which its key scheme parses (it used the key's `Debug` name, which parses as no keystroke for these).
- Where each works: every key headless and on the terminal, GPUI, the web, Android and Linux; on Windows the arrows,
  OK, Back and its four `VK_MEDIA_*` keys; on macOS and iOS the arrows, OK and Back. Windows', iOS' and macOS' agents
  fail a key they cannot press, naming it and the platform.

`docs/dev/tooling.md`, the website's testing page (English and zh-CN) and the `remote` declaration say so.

## Public API

- `nsh/test`: `app.keyboard.remote(key: string): Promise<void>`.
- `nsh_testing::remote::RemoteKey` (`parse`, `names`, `keystroke`), re-exported from `nsh_testing`;
  `TestStep::PressRemoteKey`; `press_remote_key` on the test driver and on `agent::Native` (with a default).
- `nsh_dev::protocol`: the dev protocol's `PressRemoteKey { key }` message, for real hosts.

## How to Test

- `cargo test -p nsh-testing --features bundle --test testing remote_keys::` (an app test pressing the remote's keys
  headless: every media and TV key firing its binding, the arrows whatever the name's case, `ok` and `enter` pressing
  the focused view, `back` going back and then doing nothing, unknown names failing with the names there are).
- `cargo test -p nsh-testing --features agent --lib agent::tests::remote_keys`,
  `cargo test -p nsh-platform-tui --features dev,run-app --test tui remote_keys::`,
  `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --features dev --lib agent::tests::remote_keys`,
  `cargo test -p nsh-cli --bin nsh test_web::tests::remote_keys`.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): test API only.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (`nsh test --target ios` / `android` with a remote step).
