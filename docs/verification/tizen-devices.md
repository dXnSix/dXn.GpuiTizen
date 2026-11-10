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
| `dXnSix/dXn.GpuiTizen` | `main` | 9cd8fa3 |
| `dXnSix/nsh` | `feat/tizen`, `fork/tizen-ci` and every `pr/{focus,tv,engine,tizen,tizen-web}/*` branch | `feat/tizen` 7c5cd1c13 |

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

Still open in this step (the owner's): push the commits above (dXn.GpuiTizen `main`, the NSH branches they changed),
open the fork PRs, add the signing secrets (`README.md`, "Continuous integration"), and watch the first CI runs
(`CI-3`), recorded below.

## CI on GitHub (`CI-3`)

Not run yet.

## Devices (`HOST-3`, `DIAG-5`, `DEV-2`, `PLAT-7`, `VID-4`, `DEV-3`)

Not run yet.
