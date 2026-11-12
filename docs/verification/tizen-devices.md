# Verification log: publication, CI and devices

What the owner-run sprints (s23–s25) proved, step by step, with dates. Raw logs stay under the ignored
`artifacts-temp/`; this file records the outcome of each step and where it ran.

## Publication of the forks and the git patch (`DEV-1`, 2026-11-10)

The owner turned GitHub Actions off on `dXnSix/gpui-fast` and `dXnSix/wgpu` and pushed:

| Repository | Branches | Commit |
| --- | --- | --- |
| `dXnSix/gpui-fast` | `tizen`, `tizen-release` | `tizen` 7047b05, `tizen-release` 53659f0 (generated from `tizen` 7047b05) |
| `dXnSix/gpui-fast` | `gles-instance-path`, `wgpu-backend-env`, `pipeline-cache`, `scene-damage`, `external-target`, `video-hole` | the PR branches of `docs/pr/gpui-fast/` |
| `dXnSix/wgpu` | `v30-tizen` | 4e3d80d5d |
| `dXnSix/wgpu` | `egl14-wayland`, `gles-program-binaries` | the PR branches of `docs/pr/wgpu/` |
| `dXnSix/wgpu` | `v30-egl14`, `v30-gles-program-binaries` | 66e825217, de226d2fa: the v30 ports `v30-tizen` merges, pushed 2026-11-12 (the s23 review's `s23-dev1-wgpu-v30-branches`) |
| `dXnSix/dXn.GpuiTizen` | `main` | 9cd8fa3, then f1fa54d (the git patch below) |
| `dXnSix/nsh` | `feat/tizen`, `fork/tizen-ci` and every `pr/{focus,tv,engine,tizen,tizen-web}/*` branch | `feat/tizen` 7c5cd1c13, then 289fcd8ff with the six branches the git patch below changed |

Then, on the owner's go, the git `[patch.crates-io]` lines were committed:

- dXn.GpuiTizen `47eae6b` (`build: take gpui-fast and wgpu from the forks' pushed branches`): all nine `gpui-fast-*`
  crates from `dXnSix/gpui-fast` `tizen-release` and the wgpu stack (11 crates) from `dXnSix/wgpu` `v30-tizen`;
  `8fb7af5` lets `cargo xtask cross` build from the committed patch when no local patch config is there.
- NSH `pr/tizen/04-platform` `ac5701419` (`tizen: Take gpui-fast and wgpu from their forks' pushed branches`), the
  same set in `crates/gpui/Cargo.toml`, merged forward into 05, 06, 07, `pr/tizen-web/03-module` and `feat/tizen`
  (289fcd8ff).

Confirmed, with no local patch config (`artifacts-temp/gpui-fast-patch.toml` moved aside):

- `cargo tree -i gpui-fast` (armv7 target): `gpui-fast v0.1.13 (https://github.com/dXnSix/gpui-fast?branch=tizen-release#53659f04)`;
  `cargo tree -i wgpu-hal`: `wgpu-hal v30.0.1 (https://github.com/dXnSix/wgpu?branch=v30-tizen#4e3d80d5)`. The
  lockfile holds the 20 patched crates from git.
- dXn.GpuiTizen: `cargo xtask policy`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo xtask cross --target armv7 --clippy --all-tv-libs`, `cargo xtask tizen-deps --target armv7` (one wgpu 30.0.1),
  `cargo xtask cross --target armv7 --release -p dxn_gpui_tizen_diag --check-symbols` and
  `-p example_counter --check-symbols` (each exports only `dxn_gpui_tizen_main`), `cargo test -p xtask --lib` (106).
- NSH (`pr/tizen/04-platform`): `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-tizen --lib` (23
  passed), `cargo check --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui` (the desktop build, now on the
  fork's gpui-fast), and from dXn.GpuiTizen `cargo xtask cross --target armv7 --release --manifest-path
  <nsh>/crates/gpui/Cargo.toml -p nsh-platform-tizen --with-dev-variant --check-symbols --entry nsh_tizen_main`
  (release and dev `libnsh.so` export only `nsh_tizen_main`). On `feat/tizen`, `cargo metadata --locked` passes and
  gpui-fast resolves to the fork.

Pushed on 2026-11-10: dXn.GpuiTizen `main` at f1fa54d, and the NSH branches it changed: `feat/tizen` 289fcd8ff, `pr/tizen/04-platform`
ac5701419, `05-video` 42a6cd56c, `06-module` 84758b656, `07-cli` 5881eb40f, `pr/tizen-web/03-module` 445c2e43e.
Still open in this step (the owner's): add the signing secrets (`README.md`, "Continuous integration").

On 2026-11-11, on the owner's go, the NSH stacks merged the fork's `main` (`7f5084268`, 881 commits past
`8c733731f`) by merges, `feat/tizen` took the new heads (`68e5f86af`), and the fork PRs into the fork's `main` were
opened for the ten stack tops, dXnSix/nsh#194 to #203; dXnSix/nsh#191 now targets `main`. The epic's 78 NSH checks
ran on `feat/tizen` again: all pass but the two `fmt` checks, on three files the fork's `main` leaves unformatted
itself. What each merge decided, and the PRs: `docs/pr/nsh/README.md`, "The fork's `main` merged in". The fork PRs
merge on the owner's go; their CI is watched with auto-fix.

By 2026-11-12 every fork PR had merged into the fork's `main` with a merge commit: #194, #195, #197, #198, #203 and
#205 on 2026-11-11; #202 `7ef5eb28b`, #199 `c9e2c3242`, #201 `fe3d9d8cd`, #200 `d7e13e2bd`, #196 `cfd36de9f` and
`feat/tizen` (#191) `8b680f8a5`, last, on 2026-11-12, with the fork-only #208, #209, #212, #214 and #215 between
them (`docs/pr/nsh/README.md`, "Merged into the fork's `main`"). Since #202 the fork's `main` takes the nine
gpui-fast crates from `tizen-release` and the wgpu stack from `v30-tizen` in `crates/gpui/Cargo.toml`. `DEV-1` is met.

## CI on GitHub (`CI-3`)

`ci.yml` on `dXnSix/dXn.GpuiTizen`, on pushes to `main`:

| Run | Commit | Result |
| --- | --- | --- |
| 42883920411 | 9cd8fa3 (no committed patch) | every job past `plan` and `host` failed in `setup`: with the forks checked out under `artifacts-temp/forks/`, `cargo tree` with the generated patch config could not load `naga` (`error inheriting authors from workspace root manifest's workspace.package.authors`). The fallback stays for a tree without the committed patch, but no CI run takes it any more |
| 42887002585 | f1fa54d (the git patch) | `plan`, `host`, `check` and `test` (ubuntu-24.04, macos-15), `egl` and `gles` passed; `cross (armv7)` and `cross (x86_64)` failed in `tizen-sdk`: the package manager stopped on the OS prerequisites `python3.8` and `libpython3.8`, which Ubuntu 24.04 lacks |

The fix went through dXnSix/dXn.GpuiTizen#1: `.github/actions/tizen-sdk` installs empty stand-in packages for the
prerequisite check (tried in the `gpuitizen` VM on Ubuntu 24.04). On that PR, every `ci.yml` job passed, the
`cross` jobs included.

`tizen-emulator.yml`, dispatched on the PR's branch:

| Run | Result |
| --- | --- |
| 42890534996 | passed the prerequisite check, then the runner's disk filled up while the SDK installed the TV emulator packages (`No space left on device`); fixed by a step that frees disk space |
| 42891492722 | SDK installed; the signing step (a throwaway `tz cert` author certificate and the SDK's public distributor, no secrets) and the signed x86_64 diagnostic TPK passed. The emulator did not boot in 15 minutes, then `sdb dlog` waited for a device until the job's 90-minute limit, and GitHub kept no logs. The boot step now prints the emulator's logs itself, and the steps have their own limits |
| 42907330624 | the boot step's own diagnostics: `em-cli create` ran without a display and configured VIGS's software backend, so the TV tuner device failed (`winsys interface 'vigs_wsi' not found`) and the VM exited at once |
| 42911347984 | Xvfb with GLX before `create`: the VM got VIGS's GL backend and booted (`sdb devices`: `emulator-26101 device`), but the readiness check through `sdb shell echo` never answered (the TV emulator's shell runs no plain commands) |
| 42915085901 | ready by `sdb capability` (`cpu_arch:x86_64`, `platform_version:10.0`, `profile_name:tv`); `sdb install` handed the `.tpk` to the web-app installer, which rejected it while `sdb` exited 0 |
| 42917352601 | installing with `tz install` / `sdb shell 0 vd_appinstall`: the installer rejected the signature (`unrecognized root CA certificate` for `signature1.xml`) with the SDK's old public distributor (expired 2012) |
| 42919211444 | the same with the SDK's current public distributor (valid until 2032): the Samsung TV emulator trusts only Samsung's distributor roots. The emulator's DUID is `XTCJYJZXZBZVK` on every run |
| 42921473021 | without the Samsung secrets: signed with a throwaway certificate, the emulator booted, install and report skipped with a warning; the job passed |

`CI-3`, emulator part, recorded as a failure: the emulator boots on a hosted runner, but installing needs the
owner's Samsung certificate with the emulator's DUID (`docs/development_setup.md`, sections 2 and 5). It's
deferred with the TV work to `s25-owner-devices`. Without the secrets, the job now boots the emulator and skips the
install with a warning.

The NSH fork's `fork-tizen.yml` (`fork/tizen-ci`) runs on `feat/tizen` in the draft dXnSix/nsh#191, labelled
`ci:tizen` and `ci:tizen-emulator`:

| Run | `feat/tizen` | Result |
| --- | --- | --- |
| 42943752989, 42943763914 | 696b30f19 | `cross` and `emulator` could not check out `dXnSix/dXn.GpuiTizen`, then private, with the job's token; `tv-specs` lacked the GPUI host's system libraries (`glib-2.0` not found). The owner made dXn.GpuiTizen public, and `fork/tizen-ci` `d5ddb90c2` installs `gpui.yml`'s libraries |
| 42944522991 | 5e51c6411 | all five jobs passed (2026-11-11): `cross` for armv7, x86_64 and aarch64 (`nsh-platform-tizen` release and dev, one export each, the TV clippy), `tv-specs` (the TV idiom on the GPUI host and as a Tizen web app, the counter), and `emulator` boot-only (`cpu_arch:x86_64`, `platform_version:10.0`; the counter's spec skipped without the Samsung secrets) |

After the merges (2026-11-12):

| Run | Where | Result |
| --- | --- | --- |
| 43152735862 | dXn.GpuiTizen `ci.yml`, `main` 0164102 | every job passed: `plan`, `host`, `check` and `test` (ubuntu-24.04, macos-15), `egl`, `gles`, `cross (armv7)`, `cross (x86_64)` |
| 43149590583 | NSH `fork-tizen.yml` on dXnSix/nsh#191, `feat/tizen` 2d011802b (its tree the fork's `main` after #196) | all six jobs passed, `emulator` boot-only, with `.github/actions/tizen-sdk` as of 6171d2e (the cached SDK's emulator libraries) |
| 43152711045 | NSH `fork-tizen.yml` on the push of the fork's `main` 8b680f8a5 | `plan`, `tv-specs` and the three `cross` jobs passed; `emulator` runs on labelled PRs only |

`CI-3`, NSH part: passed, except the counter's spec on the emulator, which waits for the same Samsung certificate.
`CI-3` is met with that recorded failure: the emulator's install, report and counter spec move to `t9-install`
(`s25-owner-devices`) with the owner's Samsung certificate.

Found while preparing the certificates (`docs/development_setup.md`): `cargo xtask package --sign` could never sign,
because `tz pack` rejects a plain directory ("is not a valid project"). It now zips the staged tree and re-signs it
with `tz pack -b` (tried locally with a throwaway profile: `author-signature.xml` and `signature1.xml` in the
package). And `tz` keeps certificate passwords in the OS keyring, never in `profiles.xml`, so the planned
`TIZEN_PROFILES_XML` secret could not have worked. The four certificate and password secrets stay.

## Devices (`HOST-3`, `DIAG-5`, `DEV-2`, `PLAT-7`, `VID-4`, `DEV-3`)

Not run yet.
