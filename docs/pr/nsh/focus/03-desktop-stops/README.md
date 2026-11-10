# pr/focus/03-desktop-stops

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/focus/03-desktop-stops`, head `8d563c338`, local, checked out in `nsh-wt/tizen-desktop`, not pushed |
| Base | `pr/focus/01-core` (`ce993fd21`), on `upstream/main` `41100f2bf`. Against `c16a70096` it conflicts in `docs/dev/elements.md` and `docs/dev/windows.md`; `nsh-platform-windows/src/text_input.rs` merges textually with #1475 (RichEdit text inputs) but must be rebuilt and checked. Not rebased |
| Depends on | 01 merged; the desktop issue ([../issue-desktop-tab.md](../issue-desktop-tab.md)) answered: it changes Tab's order on macOS and Linux, Tab on Windows, and the arrows on all three |
| Size | own: 36 files, +2501 −170: `nsh-platform-macos` (`focus.rs` +389 new, `input.rs`, `text_input.rs`, `view.rs`), `nsh-platform-linux` (`key_focus.rs` +855, `navigation.rs`, `scroll.rs`, `text_input.rs`), `nsh-platform-windows` (`focus.rs` +692 new, `input.rs`, `mount.rs`, `navigation.rs`, `surface.rs`, `text_input.rs`), `nsh-testing`'s headless keyboard, the app spec `tests/apps/pressable-focus/`, `AGENTS.md`, `docs/dev/{elements,linux,macos,windows}.md`, the website's forms-and-keyboard, linux, macos, windows and elements pages (English and zh-CN). With 01: 44 files, +3854 −464 |
| Its commits | `9f79c06c4` macos: Move Tab through the stops in tree order; `07b9bd44f` linux: Move Tab through the stops in tree order; `4fdbdf9bb` windows: Stop Tab at pressables, in tree order; `0c6f760ad` windows: Move focus with the arrows through nsh_core::focus; `8332c5ba2` linux: Report a field's blur before the next view's focus; `07eec85a1` macos: Move focus with the arrows through nsh_core::focus; `48fed3f34` linux: Move focus with the arrows through nsh_core::focus; `8d563c338` windows: Name the app spec that checks the keyboard focus |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes |
| Review findings | `f-s15-desktop-arrows` (major): the arrows moved the focus on Windows only, while macOS and Linux still scrolled. `07eec85a1` (macOS: `focus::arrow`, a single-line field hands Up / Down over from `moveUp:` / `moveDown:`) and `48fed3f34` (Linux: `key_focus::arrow`, in the window's scroll key controller) move it through `nsh_core::focus` there too, and `tests/apps/pressable-focus` gained two arrow tests ("The arrows move the focus to the nearest stop in their direction"; "With nothing focused Down takes the first stop, and a field keeps Left and Right"), run by `chk-nsh-focus-macos` and `chk-nsh-focus-linux`; `8d563c338` names the spec in `docs/dev/windows.md`. Part of `f-s15-docs-platform-claims`: `docs/dev/{macos,linux,windows}.md` say what the arrows do. Passed in the re-review |
| Needs | no nsh-compat change; no new API |

## Before opening

- It is large for one PR (three backends, Tab and arrows). If the maintainers ask, split it: Tab (`9f79c06c4`,
  `07b9bd44f`, `4fdbdf9bb`, `8332c5ba2`) and the arrows (`0c6f760ad`, `07eec85a1`, `48fed3f34`), each with its
  docs; the app spec covers both.
- Upstream #1475 rewrote Windows' text inputs on RichEdit (`ad481faa4`: a new `richedit.rs`, 1911 lines of `text_input.rs` changed); re-check that
  a field reached by Tab still selects its text, and rebuild with the Windows clippy.
- Windows: only `clippy` for `x86_64-pc-windows-msvc` ran here; `nsh test tests/apps/pressable-focus --target windows`
  needs a Windows machine or the fork's Windows CI (`fork-ci.yml`) once the branch is in the fork's `main`.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-focus-macos` | `bash -c 'cargo build -p nsh-macos-host && cargo run -p nsh-cli -- test tests/apps/pressable-focus --target macos'` | `t15-focus-desktops` | passed, 4 tests, 2026-11-10 10:41 UTC (s15) |
| `chk-nsh-focus-macos-unit` | `cargo test -p nsh-platform-macos --lib focus::tests` | `t15-focus-desktops` | passed, 2 tests (s15) |
| `chk-nsh-focus-linux` | `../../../dXn.GpuiTizen/scripts/lima-run.sh bash -c 'cargo build -p nsh-linux-host && cargo run -p nsh-cli -- test tests/apps/pressable-focus --target linux'` | `t15-focus-desktops` | passed, 4 tests, in the `gpuitizen` Lima VM (s15) |
| `chk-nsh-focus-linux-tab` | `../../../dXn.GpuiTizen/scripts/lima-run.sh cargo test -p nsh-platform-linux --lib key_focus::` | `t15-focus-desktops` | passed, 7 tests, Lima (s15) |
| `chk-nsh-focus-windows-clippy` | `../../../dXn.GpuiTizen/scripts/check-only-clippy.sh x86_64-pc-windows-msvc -p nsh-platform-windows --all-targets -- -D warnings` | `t15-focus-desktops` | passed (clippy, check-only from the Mac) (s15) |

The `lima-run.sh` paths are relative to `nsh-wt/tizen`; from `nsh-wt/tizen-desktop` they are the same depth. Not
run on this branch alone; not run: Windows' tests, the pre-open checks.

## Updating and opening

After 01 has merged and the desktop issue is answered:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen-desktop
git fetch upstream origin
git tag epic/focus/03-desktop-stops pr/focus/03-desktop-stops
git rebase --onto upstream/main pr/focus/01-core pr/focus/03-desktop-stops
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/focus/03-desktop-stops
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/focus/03-desktop-stops
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/focus/03-desktop-stops \
  --title "desktop: Stop Tab at pressables in tree order, and move focus with the arrows" \
  --body "$(sed '1,2d' PR.md)"
```
