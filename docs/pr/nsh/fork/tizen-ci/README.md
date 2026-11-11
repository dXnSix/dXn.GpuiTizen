# fork/tizen-ci

| | |
|---|---|
| Fork PR | not opened to `dXnSix/nsh` `main` (never to `longbridge/nsh`). Its CI runs on `feat/tizen` in the draft dXnSix/nsh#191, against `ci/tizen-base` (`8c733731f`), labelled `ci:tizen` and `ci:tizen-emulator`: CI only, never merged |
| Branch | `fork/tizen-ci`, head `413e9a8f4`, pushed to `dXnSix/nsh`, checked out in `nsh-wt/tizen-ci` |
| Base | `8c733731f`, which the fork's `main` holds (`origin/main` `8c8f96971`, 2026-11-10, not fetched since, is 707 commits on and merges the branch without conflict: `git merge-tree`). Recreated on `8c733731f` for the epic; not rebased since, and a fork branch merges, it is not rebased. Against `upstream/main` `c16a70096` it also merges cleanly |
| Size | 1 file, +258: `.github/workflows/fork-tizen.yml` (new) |
| Its commits | `7f6f6e49e` fork: Check the Tizen targets in a workflow of their own; `5018781f1` fork: Run the Tizen checks for the merge queue too; `6c94f6c73` ci: Give the fork's Tizen jobs the web toolchain and a steadier emulator; `138e51e1e` ci: Run the TV idiom's specs on the GPUI host and as a Tizen web app; `6dc91f475` ci: Take the forks from crates/gpui's git patch and boot the TV emulator the way that works; `413e9a8f4` ci: Keep one label's Tizen run from cancelling another's |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merges `e9936e8fb`, `edd1ab524`, `e1cda4852`) |
| Review findings | s21's (`review-s21.json` failed with 12, `review-s21-2.json` left 2 open, `review-s21-3.json` passed at snapshot `24e1a6df` with all 13 resolved; the sprint is checkpointed). This branch's: `f-s21-tizen-web-toolchain` (major: `tv-specs` built the wasm32 web runtime without the target or `llvm-ar`) and `f-s21-apt-masked` (minor: the `chromium-browser` install, its failure masked, hid any failed package) by `6c94f6c73`; `f-s21-emulator-boot` (minor) by `6c94f6c73`, an Xvfb of the job's own and a wait until `sdb shell echo ready` answers, with the emulator's log printed `if: always()`; `f-s21-fork-ci-docs` (minor), NSH's half by `6c94f6c73`: the header names the five `TIZEN_*` secrets, `author.p12` / `distributor.p12` under `@CERT_DIR@`, the repositories the token must read and the licence acceptance, `profiles.xml` is `0600`, runs are grouped per event so a push never cancels a manual emulator run, and the new commits use `ci:` (dXn.GpuiTizen's half: `d374b9a`); `f-s21-gpui-spec-not-tv-idiom` (major) by `pr/tv/03-dev-idiom`'s `nsh test --idiom` and `tests/apps/tv-idiom`, which `138e51e1e` runs; `f-s21-tv-specs-unchecked` (minor): every `tv-specs` command now has a local check (`chk-nsh-tv-idiom-gpui`, `chk-nsh-tv-idiom-tizen-web`, `chk-nsh-test-tizen-web`). dXn.GpuiTizen's own (`f-s21-yes-pipefail`, `-yaml-checker-no-duplicates`, `-emulator-launch`, `-sdk-cache-key`, `-tv-emulator-packages`, `-cross-cache-key`, `-plan-leftovers`) are in `5415b4a` and `d374b9a`; the SDK action this workflow uses (`.github/actions/tizen-sdk`) is among them |
| Needs | on the fork's `main`: `nsh-platform-tizen`, the Tizen CLI targets and `cargo xtask tizen` (the `pr/tizen/*` and `pr/tizen-web/*` branches), and `pr/tv/03-dev-idiom`; until then it runs on `feat/tizen` in dXnSix/nsh#191. For the emulator job's spec, the Samsung signing secrets, from a certificate whose distributor lists the TV emulator's DUID `XTCJYJZXZBZVK` (the owner's, at the end of the epic); without them the job boots the emulator and skips the spec |

## What it runs

- `cross` (every fork PR, push to `main`, the merge queue; matrix armv7, x86_64, aarch64): `nsh-platform-tizen`
  release and dev through dXn.GpuiTizen's `cargo xtask cross` with the Tizen SDK's 8.0 rootstraps installed in the job,
  its symbol check (one export, `nsh_tizen_main`), and the TV clippy.
- `tv-specs`: the TV idiom on the GPUI host under Xvfb (`nsh test tests/apps/tv-idiom --target gpui --idiom tv` and
  `nsh test tests/apps/pressable-focus --target gpui --idiom tv --filter arrows`) and as a Tizen web app in headless
  Chromium (`nsh test tests/apps/tv-idiom --target tizen-web`, `nsh test examples/counter --target tizen-web`), with
  the wasm32 target, `llvm` and `AR_wasm32_unknown_unknown=llvm-ar` for the web runtime and the image's Chrome.
  `pressable-focus` runs its arrow steps only: in the TV idiom its Tab steps do not pass (one of four fails,
  `s21-merge-pf-tv-gpui.log`).
- `emulator` (by hand with `emulator: true`, once the workflow is on the fork's `main`; or on a pull request labelled
  `ci:tizen-emulator`; a runner with KVM): the steps of dXn.GpuiTizen's `tizen-emulator.yml` (its
  `docs/ci-emulator.md`). It frees disk space, starts Xvfb with GLX before `em-cli create`, and waits until `sdb
  capability` answers. With the Samsung secrets (`TIZEN_AUTHOR_P12`, `TIZEN_DISTRIBUTOR_P12` and their passwords;
  `tz security-profiles add` in an unlocked gnome-keyring, removed afterwards), it builds the x86_64 runtime and runs
  `nsh test examples/counter --target tizen`. Without them, it boots the emulator and skips the spec with a warning.
  Certificates are never committed.

Drafts run `cross` and `tv-specs` only when labelled `ci:tizen`. Runs are grouped per event and per label, so one
label's run never cancels another's. `crates/gpui` commits the forks' git `[patch.crates-io]` lines, so the jobs
check out no forks.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-ci-yaml` | `ruby ../../../dXn.GpuiTizen/scripts/check_workflow_yaml.rb .github/workflows/fork-tizen.yml` ("workflow yaml ok"; the checker now also refuses duplicate keys) | `t21-nsh-ci` | passed, 2026-11-10 18:23 UTC (s21) |
| `chk-nsh-ci-paths` | `/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/artifacts-temp/venv/bin/python .github/ci/paths.py --print` (names `nsh-platform-tizen`; needs `pr/tizen/04-platform`'s `paths.py`: on `feat/tizen` only) | `t21-nsh-ci` | passed, 18:23 UTC (s21) |
| `chk-nsh-tv-idiom-gpui` | the two `--target gpui --idiom tv` commands of `tv-specs` (after `cargo build --manifest-path crates/gpui/Cargo.toml -p nsh-gpui-host`; on `feat/tizen` only) | `t21-nsh-ci` | passed, 3 tests, 18:28 UTC (s21) |
| `chk-nsh-tv-idiom-tizen-web` | `tests/apps/tv-idiom --target tizen-web` with `$WEB_RUNTIME` ([../../README.md](../../README.md#the-epics-checks-and-the-environment-they-need)); on `feat/tizen` only | `t21-nsh-ci` | passed, 2 tests, 18:30 UTC (s21) |
| `chk-nsh-test-tizen-web` | `examples/counter --target tizen-web` (s20's; on `feat/tizen` only) | `t20-run-test` | passed, 1 test, 17:46 UTC (s20) |

s21's final check (`check-s21.json`, revision 351): 7 passed, 3 run and 4 reused.

**On this branch alone** (no result above is from it): only `chk-nsh-ci-yaml` runs there. The branch adds only the
workflow: `chk-nsh-ci-paths` names `nsh-platform-tizen` only with `pr/tizen/04-platform`'s `paths.py` and crate, and the
three `nsh test` checks need `pr/tv/03-dev-idiom` (`--idiom`, `tests/apps/tv-idiom`), `pr/focus/02-gpui` and
`pr/focus/03-desktop-stops` (the arrows on GPUI, `tests/apps/pressable-focus`) and `pr/tizen-web/01-package` (`--target
tizen-web`): they run on `feat/tizen` only. The real check is the workflow's first run on the fork, after the
dependencies above are in place (the owner's s23–s25).

## Opening (fork PR)

After the `pr/tizen/*`, `pr/tizen-web/*` and `pr/tv/03-dev-idiom` branches are in the fork's `main` and the
repositories above are pushed (owner runs this). The title says `ci:`, the prefix of the fork's other fork-only CI
work (its first two commits say `fork:`; the review asked for `ci:` from then on, without a rebase):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen-ci
git fetch origin
git push -u origin fork/tizen-ci
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/fork/tizen-ci
gh pr create --repo dXnSix/nsh --base main --head fork/tizen-ci \
  --title "ci: Check the Tizen targets in a workflow of their own" --body "$(sed '1,2d' PR.md)"
```

Merge it with a merge commit, then add it to the fork-only table of nsh-root's `upstream-pr/README.md`.
