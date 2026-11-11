# pr/tv/04-remote-keys

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tv/04-remote-keys`, head `aa20e0e58`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-remote-keys` |
| Base | `pr/tv/01-keys` at its head (merged in by `a0b3f29da`), on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `crates/nsh-platform-web/web/nsh.test.mjs` (01's), `website/docs/testing-and-trace.md` and its zh-CN page. Not rebased |
| Depends on | 01 merged |
| Size | own: 21 files, +768 −11: `nsh-testing` (the new `remote.rs` +148, `agent.rs` +118, `driver.rs`, `headless/keyboard.rs`, `module.rs`, `lib.rs`, the new test app `tests/testing/remote/` and `tests/testing/remote_keys.rs`), the agents of GPUI (`agent.rs` +52), the terminal (`agent.rs`, `tests/tui/remote_keys.rs`), iOS and Windows; `nsh-cli/src/test_web.rs` (+125), `nsh-dev/src/protocol.rs`; `docs/dev/tooling.md`, the website's testing page (English and zh-CN). With 01: 41 files, +1508 −38 |
| Its commits | `7ea6f10ef` tui: Name the media and TV remote keys the test agent presses; `2b5417a15` test: Press TV remote keys in app tests; (merge `a0b3f29da` of 01); `aa20e0e58` test: Say where each TV remote key works, and test the agents' remote keys |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes |
| Review findings | `f-s16-remote-keys-every-platform` (minor): `keyboard.remote` was not supported everywhere and the docs did not say so. `aa20e0e58` documents which keys work where (`docs/dev/tooling.md`, the website's testing page in English and zh-CN, the `Keyboard::remote` comment the declarations come from), makes Windows' and iOS' agents fail an unsupported key naming it and the platform, and tests GPUI's (`remote_virtual_key`) and the web's (`tv_key_event`) remote paths. Passed in the re-review (`review-s16-2.json`) |
| Needs | no nsh-compat change |

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-remote-keys` | `cargo test -p nsh-testing --features bundle --test testing remote_keys::` | `t16-remote-keys` | passed, 1 test, 2026-11-10 12:25 UTC (s16) |
| `chk-nsh-remote-keys-agent` | `cargo test -p nsh-testing --features agent --lib agent::tests::remote_keys` | `t16-remote-keys` | passed, 1 test (s16) |
| `chk-nsh-remote-keys-tui` | `cargo test -p nsh-platform-tui --features dev,run-app --test tui remote_keys::` | `t16-remote-keys` | passed, 1 test (s16) |
| `chk-nsh-remote-keys-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --features dev --lib agent::tests::remote_keys` | `t16-remote-keys` | passed, 2 tests (s16) |
| `chk-nsh-remote-keys-web` | `cargo test -p nsh-cli --bin nsh test_web::tests::remote_keys` | `t16-remote-keys` | passed, 2 tests, 12:43 UTC (s16) |

**On this branch alone** (no result above is from it): every command runs as written. Not run: the pre-open checks, iOS
and Windows builds of their agents.

## Updating and opening

After 01 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen-remote-keys
git fetch upstream origin
git tag epic/tv/04-remote-keys pr/tv/04-remote-keys
git rebase --onto upstream/main pr/tv/01-keys pr/tv/04-remote-keys
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tv/04-remote-keys
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tv/04-remote-keys
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tv/04-remote-keys \
  --title "test: Press TV remote keys in app tests" --body "$(sed '1,2d' PR.md)"
```
