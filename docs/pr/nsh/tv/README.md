# tv: the TV remote's keys and the TV category

What a TV app needs that is not Tizen's: the remote's media, color, channel and Info keys in `nsh/keymap`; TVs as a
category on any OS (`platform.tv`, `tv={{…}}` props, a `tv` key in `platform.select`, never desktop); the web page
telling the player it is on a TV; the TV idiom on the Mac with hot reload; and a remote-key step in `nsh/test`.

One issue comes first: [issue.md](issue.md) (the keys' names, the category's rules, the questions). Two of the PRs
need an nsh-compat PR merged before them ([../compat/README.md](../compat/README.md)).

## The PRs, in order

All pushed to `dXnSix/nsh` (2026-11-10), worked on in `nsh-wt/tizen`; 01 and 02 are cut from `upstream/main` `8c733731f` on their own.

| # | Folder | Branch | Built on | Title | Own size | Needs | Status |
|---|---|---|---|---|---|---|---|
| 01 | [01-keys/](01-keys/) | `pr/tv/01-keys` | `upstream/main` `8c733731f` | keymap: Name the media and TV remote keys | 22 files, +740 −27 | the issue | ready, not opened: `up/tv/01-keys` |
| 02 | [02-category/](02-category/) | `pr/tv/02-category` | `upstream/main` `8c733731f` | js: Add platform.tv, the tv prop group and the tv select key | 42 files, +957 −106 | the issue; nsh-compat [`tv-idiom`](../compat/tv-idiom/) merged | ready, not opened: `up/tv/02-category` |
| 03 | [03-dev-idiom/](03-dev-idiom/) | `pr/tv/03-dev-idiom` | 02 | cli: Run apps in the TV idiom on the desktop | 32 files, +850 −54 | 02 merged | ready, not opened: `up/tv/03-dev-idiom` |
| 04 | [04-remote-keys/](04-remote-keys/) | `pr/tv/04-remote-keys` | 01 | test: Press TV remote keys in app tests | 21 files, +768 −11 | 01 merged | ready, not opened: `up/tv/04-remote-keys` |

```
01-keys ───── 04-remote-keys
02-category ─ 03-dev-idiom
```

01 and 02 can be open at the same time; they conflict with each other in nothing (`git merge-tree`, both on
`8c733731f`: clean). 03 waits for 02, 04 for 01. The `tizen` and `tizen-web` areas build on 01 and 02
(`pr/tizen/01-os` on 02, `pr/tizen/04-platform` and `pr/tizen-web/01-package` merge 01), and `fork/tizen-ci`'s
`tv-specs` job runs 03's `nsh test --idiom tv` and `tests/apps/tv-idiom`.

## Stack state (2026-11-10, 19:30 CET)

- 03 and 04 hold 02 and 01 at their heads. 03 gained two commits at 19:13 and 19:15 (`35ce21821` `nsh test --idiom`,
  `2cb11014a` `tests/apps/tv-idiom`), the repair of s21's `f-s21-gpui-spec-not-tv-idiom`, appended without a rebase
  and merged into `feat/tizen` (`e7325eb31`).
- Every branch that builds on 01 or 02 now holds its head: `pr/tizen/01-os` merged the latest 02 (`5c1bfe2fe`),
  `pr/tizen/04-platform` the latest 01 (`39e6b0c8e`) and the latest 02 through `pr/tizen/01-os` (`a892c2d4c`),
  `pr/tizen-web/01-package` both.
- `nsh-wt/tizen-engine` now has 03 checked out (it had 01); 01 is checked out nowhere.

## The reviews

| Finding | Branch | What changed |
|---|---|---|
| `f-s16-macos-agent-key-match` (major) | 01 | macOS' test agent fails the 15 media and TV keys with an error naming the key and macOS (`b8bc8dee1`); a clippy build of the macOS crate checks it (`chk-nsh-keys-macos-build`) |
| `f-s16-tv-keycode-fallback-narrow` (minor) | 01 | in the TV idiom `nsh.js` reads Samsung's key codes for any key it does not name (`c621bc72b`) |
| `f-s16-gpui-info-key-name` (minor) | 01 | Info is `info`, `xf86info` an alias (`f908ff626`; dXn.GpuiTizen `e8e5693` names it) |
| `f-s16-tv-web-test-reads-source` (minor) | 02 | `tv.test.mjs` checks the TV idiom on the browser-tests module itself, not the Rust source (`532345852`) |
| `f-s16-tv-host-props-undeclared` (minor) | 02 | the unused host-prop platform `tv` is gone (`a8f7ad63c`) |
| `f-s16-remote-keys-every-platform` (minor) | 04 | the docs say where each remote key works; Windows' and iOS' agents fail clearly; GPUI's and the web's remote paths have tests (`aa20e0e58`) |
| `f-s16-nsh-compat-one-thing-per-pr` (minor) | nsh-compat | one companion split into `tv-idiom`, `focus-props`, `tizen-platform` ([../compat/README.md](../compat/README.md)) |
| `f-s16-tv5-os-tizen-unmet` (major) | – | by amendment: `Os::Tizen`'s case moved to NTZ-2 (`pr/tizen/01-os`, `chk-nsh-compat-tizen`) |
| `f-s21-gpui-spec-not-tv-idiom` (major) | 03 | `nsh test --target gpui` and `--target web` take `--idiom` (`35ce21821`), and `tests/apps/tv-idiom` checks `platform.tv` and `tv=` props in the TV idiom (`2cb11014a`); `chk-nsh-tv-idiom-gpui` ran it, 2 passed |

s16's review (`review-s16.json` failed, `review-s16-2.json` passed at snapshot `3517f005`, all 17 checks passing) covers
01, 02, 03 (its first commit) and 04. s21's (`review-s21.json`, failed) raised the 03 finding; `review-s21-2.json`
resolved it, and `review-s21-3.json` passed at snapshot `24e1a6df` with all 13 of s21's findings resolved.

## Against the latest `upstream/main` (`c16a70096`)

- 01 conflicts in `crates/nsh-platform-web/web/nsh.test.mjs`; 04 also in `website/docs/testing-and-trace.md` and its
  zh-CN page.
- 02 and 03 conflict in `crates/nsh-core/src/prelude.rs` (upstream's `data_dir`, #1550, and other exports beside
  `is_tv`) and `crates/nsh-platform-web/src/browser/engine.rs`.

## Rules for this area

nsh-root's ([../README.md](../README.md#rules)). The new keys and the category widen types: each has its
`tests/compat` case (`modules/mod-keymap-tv`, `tv/mod-device-tv`, `tv/props-platform-tv`), passing the v0.2.0 record.
