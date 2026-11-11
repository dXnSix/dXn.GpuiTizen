# pr/tv/01-keys

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tv/01-keys`, head `f908ff626`, pushed to `dXnSix/nsh` (checked out nowhere; `nsh-wt/tizen-engine` has `pr/tv/03-dev-idiom` now) |
| Base | `upstream/main` `8c733731f` (#1135, 2026-11-09). Against `c16a70096` (161 commits on) it conflicts in `crates/nsh-platform-web/web/nsh.test.mjs`. Not rebased |
| Depends on | nothing |
| Size | 22 files, +740 −27: `nsh-core/src/keymap.rs` (+306), `nsh-js/src/keymap.rs` (the TS names), `nsh-platform-gpui`'s `keys.rs` (+113), `keymap.rs`, `agent.rs`; `nsh-platform-web`'s `src/lib.rs`, `web/nsh.js`, `web/nsh.test.mjs`; `nsh-platform-linux`'s `agent.rs`, `keymap.rs`, `xtest.rs`; `nsh-platform-macos/src/agent.rs`; `nsh-platform-tui/src/app.rs`; `nsh-cli/src/test_web.rs`; compat case `modules/mod-keymap-tv.tsx`; `docs/dev/{elements,gpui,web}.md`; `website/modules/nsh.md` (English and zh-CN) |
| Its commits | `99a05bf7b` keymap: Name the media and TV remote keys; `4e2fe610d` web: Map the media and TV remote keys, by Samsung TVs' key codes too; `2b8bc88a9` gpui: Map the Tizen platform's TV remote key names; `b8bc8dee1` macos: Fail clearly on the media and TV remote keys in the test agent; `c621bc72b` web: Read TV remote keys by Samsung's key codes whenever the TV idiom's key is unnamed; `f908ff626` gpui: Say the Tizen platform names Info `info`, with `xf86info` an alias |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; merged at its head into `pr/tv/04-remote-keys`, `pr/tizen/04-platform` (since `39e6b0c8e`) and `pr/tizen-web/01-package` |
| Review findings | `f-s16-macos-agent-key-match` (major): macOS' test agent matched the keys exhaustively and could not press the new ones; `b8bc8dee1` fails the 15 media and TV keys with an error naming the key and macOS (on a Mac they are NX events, never app key events), checked by `chk-nsh-keys-macos-build`. `f-s16-tv-keycode-fallback-narrow` (minor): `c621bc72b` applies Samsung's key codes in the TV idiom to any key `nsh.js` does not name (empty, missing, `Pause` for 19, `XF86PlayBack`), with two "TV remote" tests. `f-s16-gpui-info-key-name` (minor): `f908ff626` says Info is `info`, `xf86info` an alias (dXn.GpuiTizen `e8e5693` names it so). Passed in the re-review (`review-s16-2.json`) |
| Needs | the TV issue ([../issue.md](../issue.md)); no nsh-compat change (the compat case uses existing `key` steps) |

## Before opening

- `c621bc72b` reads keys by Samsung's key codes in the TV idiom (`nsh.js`: the player's `idiom` option is `"tv"`).
  That value is 02's to define (`nsh_environment` reads it as `Idiom::Tv` from 02 on); on this branch alone the key
  codes apply when a page passes it, and the app's environment does not know it yet. Say so in the PR, or open it
  after 02. Its tests passed on `feat/tizen`, which has both; check them on the branch alone.
- `2b8bc88a9` and `f908ff626` map the names the Tizen platform (dXn.GpuiTizen) gives GPUI; upstream has no Tizen
  platform yet. Say so in the PR (they are the names, not a dependency), or move them to the `tizen` area if the
  maintainers prefer.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-keymap-tv` | `cargo test -p nsh-core --lib keymap::tests::tv` | `t16-keys` | passed, 7 tests, 2026-11-10 12:15 UTC (s16) |
| `chk-nsh-keys-gpui` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib -- keys::tests::tv keymap::tests::tv_keys` | `t16-keys` | passed, 3 tests (s16) |
| `chk-nsh-keys-web` | `node --test --test-name-pattern='TV remote' crates/nsh-platform-web/web/nsh.test.mjs` | `t16-keys` | passed, 2 tests (s16) |
| `chk-nsh-keys-macos-build` | `cargo clippy -p nsh-platform-macos --all-targets --features dev -- -D warnings` | `t16-keys` | passed (s16) |
| `chk-nsh-compat` | the compat run ([../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)) | `t16-compat` | passed, 13:00 UTC (s16) |

**On this branch alone** (no result above is from it): the four `cargo` and `node` commands run as written (the recorded
`chk-nsh-keys-web` loaded `feat/tizen`'s `nsh.js`, with the focus branches' changes and 02's TV idiom: see above).
`chk-nsh-compat` ran with the other branches' compat cases and nsh-compat's `nsh-tizen`; this branch needs no companion,
so clone nsh-compat `main` in its place. Not run: the pre-open checks, the Windows and Android builds (they map keys
too).

## Updating and opening

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tv/01-keys pr/tv/01-keys
git rebase --onto upstream/main 8c733731f pr/tv/01-keys
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tv/01-keys
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tv/01-keys
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tv/01-keys \
  --title "keymap: Name the media and TV remote keys" --body "$(sed '1,2d' PR.md)"
```
