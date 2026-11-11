# pr/tizen/03-gpui-tizen-cfg

| | |
|---|---|
| Upstream PR | not opened |
| Fork PR | none of its own: in dXnSix/nsh#201, dXnSix/nsh#202 (its stack tops'), into the fork's `main`, on the owner's go |
| Branch | `pr/tizen/03-gpui-tizen-cfg`, head `57862414c`, pushed to `dXnSix/nsh` |
| Base | `pr/tizen/02-gpui-platform-optional` at its head (merged in twice: `bcc0697ae`, `d0e064bbb`), on `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `website/zh-CN/docs/gpui.md` (02's); upstream's #1439 changes the same web view path (below). Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `5c719ccbf`: `57862414c` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 02 merged |
| Size | own: 18 files, +645 −62: `nsh-platform-gpui`'s `Cargo.toml` (target tables without the vendor), `src/lib.rs` (the `compile_error!`), `src/app.rs` (the new setters), the new `src/host_settings.rs` (+74), `src/system.rs` (+89: the TV's), `src/text.rs` (+105) and the new `src/text/tv.rs` (+158), `src/native.rs`, `src/native/video.rs`, `src/native/web.rs` (+112: the placeholder), `src/image.rs`, `src/surface.rs`, `src/synth.rs`; `crates/gpui/Cargo.toml` (`unexpected_cfgs`); `docs/dev/{embedding,gpui}.md`, the website's GPUI page (English and zh-CN). With 02: 19 files, +679 −78 |
| Its commits | `3411a7710` gpui: Build for Tizen; (merge `bcc0697ae`); `6325f81ea` gpui: Match a TV's UI family ignoring case and spaces for its line metrics; (merge `d0e064bbb`); `2e0e29dc5` gpui: Keep Tizen's UI family list for the docs only; `5c719ccbf` gpui: Stop a Tizen build that keeps the platform feature |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; the base of 04 |
| Review findings | `f-s17-platform-feature-reaches-tizen` (minor): a Tizen build with the default `platform` feature pulled the desktop platform in; `5c719ccbf` stops it with a `compile_error!` naming `--no-default-features --features js`, and `docs/dev/gpui.md` says so. `f-s17-tv-ui-lines-name-match` (minor): `ui_lines` matched TV families by exact name; `6325f81ea` matches them ignoring case and spaces (`tv::ui_candidates`, `is_ui_family`, `same_family`, the test `the_preferred_family_matches_ignoring_case_and_spaces`) and `2e0e29dc5` keeps the `UI_FAMILIES` list for the docs only. Passed in `review-s17-2.json` |
| Needs | the Tizen issue; no nsh-compat change. Nothing of dXn.GpuiTizen: it builds the crate for a Tizen target, and nothing runs it until 04 |

## Before opening

- **Upstream #1439** ("Tell apps when the platform cannot show a web view", `23ee20fc8`): platforms report
  `Environment::web_views` at start and the core sends `onError` with `ERR_FEATURE_UNAVAILABLE_ON_PLATFORM` to every
  web view where it is `false`. 03's `native/web.rs` placeholder reports `onError` for each source itself (on Tizen and
  in builds without `webview`). After the move, report `web_views: false` on Tizen and without `webview` instead, and
  drop what #1439 now does (the `native::web::placeholder::tests` group shrinks or goes). `PR.md` is written for the
  branch after that change, with an `[OWNER: …]` marker saying so; the branch as it is reports `onError` from its own
  placeholder.
- `2e0e29dc5` keeps `UI_FAMILIES` "for the docs only, allowed unused" on Tizen: a reviewer may prefer it moved to a
  doc comment.

## Checks

| Check | Command | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-gpui-tizen-build` | `cargo xtask cross --target armv7 --manifest-path ../nsh/nsh-wt/tizen/crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features --features js` (from dXn.GpuiTizen) | `t17-gpui-tizen-cfg` | passed, 2026-11-10 14:15 UTC (s17) |
| `chk-nsh-gpui-tizen-deps` | `cargo xtask tizen-deps --target armv7 --manifest-path ../nsh/nsh-wt/tizen/crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features --features js` (from dXn.GpuiTizen; "no desktop platform crates") | `t17-gpui-optional`, `t17-gpui-tizen-cfg` | passed, 14:14 UTC (s17) |
| `chk-nsh-gpui-tizen-settings` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib host_settings::tests` | `t17-gpui-tizen-cfg` | passed, 1 test (s17; 04 adds `lifecycle` to the group) |
| `chk-nsh-gpui-tizen-system` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib system::tv::tests` | `t17-gpui-tizen-cfg` | passed, 1 test (s17) |
| `chk-nsh-gpui-tizen-fonts` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib text::tv::tests` | `t17-gpui-tizen-cfg` | passed, 3 tests (s17) |
| `chk-nsh-gpui-webview-placeholder` | `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --no-default-features --features platform --lib native::web::placeholder::tests` | `t17-gpui-tizen-cfg` | passed, 1 test, 14:27 UTC (s17) |

The cross checks need the Tizen 8.0 rootstrap and the SDK's GCC toolchain (dXn.GpuiTizen's `docs/setup.md`).

**On this branch alone** (no result above is from it): the four `cargo test` commands run as written; the two cross
checks' `--manifest-path` names `nsh-wt/tizen`, which is `feat/tizen`: point it at a worktree that has this branch. Not
run: the pre-open checks.

## Updating and opening

After 02 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tizen/03-gpui-tizen-cfg pr/tizen/03-gpui-tizen-cfg
git rebase --onto upstream/main pr/tizen/02-gpui-platform-optional pr/tizen/03-gpui-tizen-cfg   # then #1439, above
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/tizen/03-gpui-tizen-cfg
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/03-gpui-tizen-cfg
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen/03-gpui-tizen-cfg \
  --title "gpui: Build for Tizen" --body "$(sed '1,2d' PR.md)"
```
