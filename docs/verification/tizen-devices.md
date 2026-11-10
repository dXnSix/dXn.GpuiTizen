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

## CI on GitHub (`CI-3`)

`ci.yml` on `dXnSix/dXn.GpuiTizen`, on pushes to `main`:

| Run | Commit | Result |
| --- | --- | --- |
| 42883920411 | 9cd8fa3 (no committed patch) | every job past `plan` and `host` failed in `setup`: with the forks checked out under `artifacts-temp/forks/`, `cargo tree` with the generated patch config could not load `naga` (`error inheriting authors from workspace root manifest's workspace.package.authors`). The fallback stays for a tree without the committed patch, but no CI run takes it any more |
| 42887002585 | f1fa54d (the git patch) | `plan`, `host`, `check` and `test` (ubuntu-24.04, macos-15), `egl` and `gles` passed; `cross (armv7)` and `cross (x86_64)` failed in `tizen-sdk`: the package manager stopped on the OS prerequisites `python3.8` and `libpython3.8`, which Ubuntu 24.04 lacks |

The fix (`.github/actions/tizen-sdk`: empty stand-in packages for the prerequisite check, run in the `gpuitizen` VM
on Ubuntu 24.04) went through a pull request; its run is recorded below.

## Devices (`HOST-3`, `DIAG-5`, `DEV-2`, `PLAT-7`, `VID-4`, `DEV-3`)

Not run yet.
