# pr/tizen/06-module

| | |
|---|---|
| Upstream PR | not opened |
| Fork PR | none of its own: in dXnSix/nsh#201, dXnSix/nsh#202 (its stack tops'), into the fork's `main`, on the owner's go |
| Branch | `pr/tizen/06-module`, head `77d8ec194`, pushed to `dXnSix/nsh` |
| Base | `pr/tizen/04-platform` at its head (merged in five times: `aac49042f`, `4638db353`, `5a7c09c19`, `78692dd6a`, `8f55629e3`), on `upstream/main` `8c733731f`. Against `c16a70096` it has 04's conflicts. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `84758b656`: `575b6556f`, `77d8ec194` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | 04 merged; nsh-compat [`tv-idiom`](../../compat/tv-idiom/) (its case is in the `tv` domain) |
| Size | own: 17 files, +790 −7: `nsh-core` (the new `runtime/tizen.rs` +324, `runtime/mod.rs`, `element/ambient.rs`), `nsh-js/src/declarations.rs` (+40), `nsh-platform-tizen` (the new `src/tv.rs` +160, `keys.rs`, `lib.rs`, `tizen.rs`), compat case `tv/mod-tizen.tsx`, `docs/dev/{js-modules,tizen}.md`, the website's new `nsh/tizen` module page and the modules index (English and zh-CN). With 04 and below: 118 files, +6154 −315 |
| Its commits | `97fe382a6` core: Add nsh/tizen, what Samsung TVs have; `73fe674b5` tizen: Give nsh/tizen the TV's product information and keys; `635203bbd` tizen: Grab the remote's keys for nsh/tizen while the app runs (merges of 04 between them) |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes; merged at its head into 07 (`976dd2ac4`) and `pr/tizen-web/03-module` (`661d69f2d`) |
| Review findings | none: s18's review found nothing in `nsh/tizen`'s native half (argument and availability checks, key grabs on the main thread). Its web half's Smart Hub preview is `pr/tizen-web/03-module`'s (s20) |
| Needs | the Tizen issue answered on a module of its own (`nsh/tizen`) |

## Before opening

- `73fe674b5` says the platform grabs the keys at start and "changes no grab while the app runs"; `635203bbd` then uses
  dXn.GpuiTizen's run-time grabs (Info on request, giving a key back). The PR text follows the last state; squash the
  two if the maintainers prefer.
- `setPreviewData` rejects natively (Smart Hub's preview has no public native API); only a TV's web app sets it
  (`pr/tizen-web/03-module`). Say so in the docs, as the branch does.

## Checks

| Check | Command | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-tizen-module` | `cargo test -p nsh-core --lib runtime::tizen` | `t18-module` | passed, 5 tests, 2026-11-10 16:09 UTC (s18) |
| `chk-nsh-tizen-module-decl` | `cargo test -p nsh-js --lib declarations::tests::tizen` | `t18-module` | passed, 1 test (s18) |
| `chk-nsh-platform-tizen` | 04's (with `$TIZEN_ENV`, `$PATCH`, `$LOCK`): `tv::tests`, `keys::tests::nsh_keys_have_their_tizen_names` among the 27 | `t18-module` | passed, 27 tests, 15:49 UTC (s18) |
| `chk-nsh-compat` | the compat run (`tv/mod-tizen`) | `t16-compat` | the s16 run (13:00 UTC) came before this case; s17's `chk-nsh-compat-tizen` (14:42 UTC) too. **Not recorded for `tv/mod-tizen`** |

**On this branch alone** (no result above is from it): the three test commands run as written (`chk-nsh-platform-tizen`
selects the same 27 tests there). For `cargo xtask compat` with `tv/mod-tizen`, clone nsh-compat's `tizen-platform`
(which holds `tv-idiom`; the branch also holds 01's `tizen` cases). Not run: the pre-open checks, `cargo xtask compat`
with `tv/mod-tizen`.

## Updating and opening

After 04 has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tizen/06-module pr/tizen/06-module
git rebase --onto upstream/main pr/tizen/04-platform pr/tizen/06-module
# the checks above, cargo xtask compat, and the pre-open checks, then (owner runs this)
git push -u origin pr/tizen/06-module
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen/06-module
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen/06-module \
  --title "core: Add nsh/tizen, what Samsung TVs have" --body "$(sed '1,2d' PR.md)"
```
