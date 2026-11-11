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
- **`emulator`** (by hand with `emulator: true`, or on a PR labelled `ci:tizen-emulator`): the counter's spec on
  the Samsung TV emulator on a KVM runner (`nsh test examples/counter --target tizen`). It starts Xvfb with GLX
  before `em-cli create`, because without a display the TV tuner device stops the VM. It waits until `sdb capability`
  answers, because the TV emulator's shell runs no plain commands. It signs with a Samsung certificate profile
  added by `tz security-profiles add` from the secrets `TIZEN_AUTHOR_P12`, `TIZEN_DISTRIBUTOR_P12`,
  `TIZEN_AUTHOR_PASSWORD` and `TIZEN_DISTRIBUTOR_PASSWORD`, with the passwords in a keyring and everything removed
  afterwards. The Samsung TV emulator trusts only Samsung's distributor roots. Without the secrets the job boots the
  emulator and skips the spec.

The workflow's header lists the secrets, the repositories the fork's token must read (`dXnSix/dXn.GpuiTizen`,
`dXnSix/gpui-fast`, `dXnSix/wgpu`) and that installing the Tizen SDK accepts Samsung's licence on the runner. Runs are
grouped per event and label, so a push to `main` never cancels a manual emulator run, and one label's run never
cancels another's. Drafts run `cross` and `tv-specs` only when labelled `ci:tizen`. The gpui-fast and wgpu forks come
from `crates/gpui`'s git patch.

## How to Test

- `ruby <dXn.GpuiTizen>/scripts/check_workflow_yaml.rb .github/workflows/fork-tizen.yml` (shape and duplicate keys).
- Locally, each `tv-specs` command: `nsh test tests/apps/tv-idiom --target gpui --idiom tv` (2 passed),
  `nsh test tests/apps/pressable-focus --target gpui --idiom tv --filter arrows` (1 passed), `nsh test
  tests/apps/tv-idiom --target tizen-web` (2 passed), `nsh test examples/counter --target tizen-web` (1 passed).
- This PR's own run: `cross` and `tv-specs` green. For the emulator job, label the PR `ci:tizen-emulator`: it boots
  the emulator, and it runs the spec when the Samsung secrets are set.
