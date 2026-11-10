# ci: Check the Tizen targets in a workflow of their own

Fork-only: it adds only its own file and never goes upstream (upstream has no Tizen target CI).

## Description

`fork-tizen.yml` checks the Samsung TV targets on the fork:

- **`cross`** (every PR, pushes to `main`, the merge queue): `nsh-platform-tizen` (release and dev) cross-built for
  armv7, x86_64 and aarch64 through dXn.GpuiTizen's cross driver, with the Tizen 8.0 rootstraps installed in the job;
  each library's one export checked (`nsh_tizen_main`, only documented Tizen 8.0 symbols imported); the TV clippy.
- **`tv-specs`**: the TV idiom's specs on the desktop GPUI host under Xvfb (`nsh test tests/apps/tv-idiom --target
  gpui --idiom tv`, and `tests/apps/pressable-focus`'s arrow steps with `--idiom tv --filter arrows`) and as a Tizen web
  app in headless Chromium with the TV shim (`tests/apps/tv-idiom` and `examples/counter`, `--target tizen-web`; the
  job installs the wasm32 target and `llvm-ar` for the web runtime).
- **`emulator`** (run by hand, `emulator: true`): the counter's spec on the Samsung TV emulator on a KVM runner
  (`nsh test examples/counter --target tizen`), under an Xvfb of the job's own and once `sdb shell` answers, signed
  with the certificate profile in the repository's secrets (`TIZEN_AUTHOR_P12`, `TIZEN_DISTRIBUTOR_P12`,
  `TIZEN_AUTHOR_PASSWORD`, `TIZEN_DISTRIBUTOR_PASSWORD`, `TIZEN_PROFILES_XML`; written `0600` as `author.p12` and
  `distributor.p12` under the profile's `@CERT_DIR@`, removed afterwards), with the emulator's log printed.

The workflow's header lists the secrets, the repositories the fork's token must read (`dXnSix/dXn.GpuiTizen`,
`dXnSix/gpui-fast`, `dXnSix/wgpu`) and that installing the Tizen SDK accepts Samsung's licence on the runner. Runs are
grouped per event, so a push to `main` never cancels a manual emulator run.

Until `crates/gpui` commits the gpui-fast and wgpu forks' git patch, the jobs check out `dXnSix/gpui-fast`
(`tizen-release`) and `dXnSix/wgpu` (`v30-tizen`) and pass dXn.GpuiTizen's patch config.

## How to Test

- `ruby <dXn.GpuiTizen>/scripts/check_workflow_yaml.rb .github/workflows/fork-tizen.yml` (shape and duplicate keys).
- Locally, each `tv-specs` command: `nsh test tests/apps/tv-idiom --target gpui --idiom tv` (2 passed),
  `nsh test tests/apps/pressable-focus --target gpui --idiom tv --filter arrows` (1 passed), `nsh test
  tests/apps/tv-idiom --target tizen-web` (2 passed), `nsh test examples/counter --target tizen-web` (1 passed).
- This PR's own run: `cross` and `tv-specs` green; `emulator` once by hand (`gh workflow run fork-tizen.yml -f
  emulator=true`).
