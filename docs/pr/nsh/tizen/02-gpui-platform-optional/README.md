# pr/tizen/02-gpui-platform-optional

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tizen/02-gpui-platform-optional`, head `c22f7a995`, pushed to `dXnSix/nsh` |
| Base | `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `website/zh-CN/docs/gpui.md`. Not rebased |
| Depends on | nothing |
| Size | 8 files, +35 −17: `nsh-platform-gpui`'s `Cargo.toml` (the `platform` feature), `src/app.rs`, `src/lib.rs`, `src/text.rs`; `nsh-gpui-host/Cargo.toml` (turns it on); `docs/dev/gpui.md`, the website's GPUI page (English and zh-CN) |
| Its commits | `768864617` gpui: Make gpui_platform an optional default feature; `2f7eb1758` gpui: Let AppView's example compile without the platform feature; `c22f7a995` gpui: Format AppView's example |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; the base of 03 |
| Review findings | `f-s17-appview-doctest-needs-platform` (minor): `AppView`'s doc example called `gpui_platform`, so the doctest failed without the `platform` feature. `2f7eb1758` writes it through the crate's own `nsh_platform_gpui::gpui` re-export, `c22f7a995` formats it. Passed in `review-s17-2.json` |
| Needs | the Tizen issue (it is "change 4" of dXn.GpuiTizen's gpui-fast research, made in NSH: [../../../gpui-fast/README.md](../../../gpui-fast/README.md)) |

It needs nothing of dXn.GpuiTizen and could be opened before the rest of the area, on the owner's call.

## Before opening

- Squash `c22f7a995` (formatting) into `2f7eb1758`.
- An embedder that depends on `nsh-platform-gpui` with `default-features = false` and calls `run_app` must now turn
  `platform` on (`PR.md`, Breaking Changes). The GPUI workspace's own `nsh-platform-gpui` entry is
  `default-features = false`; `nsh-gpui-host` turns `platform` on.

## Checks

| Check | Command | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-gpui-default` | `cargo check --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui` (in the branch's checkout) | `t17-gpui-optional` | passed, 2026-11-10 14:14 UTC (s17) |
| `chk-nsh-gate-armv7` | `cargo xtask cross --target armv7 --manifest-path ../nsh/nsh-wt/tizen/crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features` (from dXn.GpuiTizen) | `t17-gpui-optional` | passed, 14:13 UTC (s17) |
| `chk-nsh-gpui-tizen-deps` | `cargo xtask tizen-deps --target armv7 --manifest-path ../nsh/nsh-wt/tizen/crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features --features js` (from dXn.GpuiTizen; expects "no desktop platform crates") | `t17-gpui-optional`, `t17-gpui-tizen-cfg` | passed, 14:14 UTC (s17) |

The recorded runs are on `feat/tizen`, with 03. Before 03 existed, the armv7 build of `nsh-platform-gpui
--no-default-features` (with and without `--features js`) finished
(`artifacts-temp/agentx/tizen-gpui-platform/ntz1-gate-armv7.log`, `ntz1-gate-armv7-js.log`, written 10:53 UTC, four
minutes before `768864617` was committed, so from its working tree), where it had failed on 2026-11-08 in `freetype-sys`
and `yeslogic-fontconfig-sys` (`amend-nsh-gate-s17.json`).

**On this branch alone** (no result above is from it): `chk-nsh-gpui-default` runs as written; the two cross checks'
`--manifest-path` names `nsh-wt/tizen`, which is `feat/tizen`: point it at a worktree that has this branch. Not run: the
pre-open checks.

## Updating and opening

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tizen/02-gpui-platform-optional pr/tizen/02-gpui-platform-optional
git rebase --onto upstream/main 8c733731f pr/tizen/02-gpui-platform-optional
# the checks above (cargo test --doc -p nsh-platform-gpui --no-default-features --features js too) and the
# pre-open checks, then (owner runs this)
git push -u origin pr/tizen/02-gpui-platform-optional
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/02-gpui-platform-optional
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen/02-gpui-platform-optional \
  --title "gpui: Make gpui_platform an optional default feature" --body "$(sed '1,2d' PR.md)"
```
