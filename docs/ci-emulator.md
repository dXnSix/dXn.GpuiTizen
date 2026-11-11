# The TV emulator on CI

How `.github/workflows/tizen-emulator.yml` runs the Samsung TV emulator on a GitHub-hosted runner, what it needs
from GitHub and the owner, and what each step works around. The NSH fork's `emulator` job
(`fork-tizen.yml`) has the same needs. Results of the runs go in
[`verification/tizen-devices.md`](verification/tizen-devices.md) (`CI-3`).

## What the job does

Each step and the problem it solves:

| Step | What it does | Why |
| --- | --- | --- |
| Free disk space | removes Android, GHC, CodeQL, Swift, PowerShell, Chromium and Julia from the runner image, and prunes Docker images | the SDK, the TV emulator platform and its VM image don't fit next to them: the first run stopped with `No space left on device` |
| KVM and Xvfb | gives every user access to `/dev/kvm` (udev rule) and installs Xvfb | the emulator is a QEMU VM and needs KVM; it opens a window, so it needs an X display |
| `setup`, `setup-dotnet` | Rust, Linux packages, .NET 8 | the diagnostic app's library and its C# host |
| `tizen-sdk` | installs the SDK CLI with `TIZEN-8.0-NativeAppDevelopment-CLI Emulator TV-SAMSUNG-Public-Emulator`, cached by the package list | see [SDK install](#sdk-install) |
| Signing profile | the owner's Samsung certificate from secrets, or without them a throwaway Tizen certificate, in a profile `ci` | see [Signing](#signing) |
| Package | `cargo xtask package --app diag --target x86_64 --sign ci` | the emulator is x86_64 |
| Boot | starts Xvfb `:99` with GLX, then `em-cli create` and `em-cli launch`, then waits up to 15 minutes until `sdb devices` lists the emulator and `sdb capability` reports its CPU architecture | see [Boot](#boot) |
| Install and launch (Samsung-signed only) | `tz install`, or `sdb push` and `sdb shell 0 vd_appinstall` if that fails, until `sdb shell 0 vd_applist` lists the package (3 tries); then `sdb shell 0 execute DxnGpuiTzD.Diag` | see [Installing](#installing) |
| Pull the report (Samsung-signed only) | `cargo xtask diag pull` (6 tries, 20 s apart) | the app needs time to start and answer |
| Emulator log, upload | the VM's logs and `sdb dlog`, uploaded as the `tizen-emulator` artifact with the report | |
| Remove the signing material | deletes the certificates and `profiles.xml` | |

## What GitHub and the owner provide

- **Actions on in `dXnSix/dXn.GpuiTizen`.** The job uses only GitHub-hosted `ubuntu-24.04` runners. Every run so far
  has had `/dev/kvm`.
- **`dXnSix/gpui-fast` and `dXnSix/wgpu` readable.** `Cargo.toml`'s git patch fetches them. Both are public.
- **The Samsung signing secrets, for the install.** `TIZEN_AUTHOR_P12`, `TIZEN_AUTHOR_PASSWORD`,
  `TIZEN_DISTRIBUTOR_P12` and `TIZEN_DISTRIBUTOR_PASSWORD`, from a Samsung certificate whose distributor lists the
  emulator's DUID ([`development_setup.md`](development_setup.md), sections 2 and 5). Without them the job only
  boots the emulator. See [Signing](#signing).
- **Starting a run.** The job runs weekly (Monday 04:41 UTC) and on demand. To run it on a branch:

  ```bash
  gh workflow run tizen-emulator.yml -R dXnSix/dXn.GpuiTizen --ref <branch>
  ```

  Starting it is the owner's acceptance of Samsung's SDK licence on the runner (`--accept-license`). The agent
  starts it on the owner's standing go.
- **Minutes.** `dXnSix/dXn.GpuiTizen` is public, so standard GitHub-hosted runners cost no Actions minutes. One run
  takes about 10 minutes until the boot, at most 20 minutes of boot and its logs, and a few minutes after. The
  job's limit is 90 minutes.
- **Cache.** `tizen-sdk` caches `~/tizen-studio` per package list. With the emulator packages that is several GB of
  the repository's 10 GB cache. GitHub evicts the oldest entries itself.

## SDK install

The package manager checks each package's OS prerequisites with `dpkg -l <name> | grep ^ii` and stops when one is
missing. The native packages list `python3.8` and `libpython3.8`. Ubuntu 24.04 has no Python 3.8, and nothing the
job runs needs it. So `.github/actions/tizen-sdk` installs empty stand-in packages with those names before
`install`. The installer's repository URL is `http://`, which the mirror answers with 403, so the action switches it
to `https://` ([`setup.md`](setup.md), step 4).

## Signing

The Samsung TV emulator trusts only Samsung's distributor roots, like a TV. Runs 42917352601 and 42919211444
signed with a `tz cert` author certificate and the SDK's Tizen public distributor. The author signature passed, but
the emulator's certificate service rejected the distributor's: `signature1.xml has got unrecognized root CA
certificate`, reported by `vd_appinstall` as `install failed[118, -12], reason: Check certificate error : :Invalid
certificate chain`. That happened with both the SDK's old public distributor, `tizen-distributor-signer.p12` (expired
in 2012), and its current one, `tizen-distributor-signer-new.p12` (valid until 2032).

So the install needs a Samsung certificate, made with the owner's Samsung account, whose distributor lists the
emulator's DUID. `sdb shell 0 getduid` reported `XTCJYJZXZBZVK` on every fresh VM and runner so far
(42915085901, 42917352601, 42919211444). One certificate serves the TV and the emulator.

The signing step:

- **With the secrets** (`TIZEN_AUTHOR_P12`, `TIZEN_DISTRIBUTOR_P12` and their passwords): it decodes the
  certificates, adds them as the profile `ci`, and sets `SAMSUNG_SIGNED=true`. Install, launch and the report run.
- **Without them:** it creates a throwaway author certificate with `tz cert` and a random password (masked in the
  log), pairs it with `tizen-distributor-signer-new.p12`, sets `SAMSUNG_SIGNED=false` and warns. The package is
  signed and the emulator boots, but install, launch and the report are skipped.

In both cases:

- `tz security-profiles add` keeps the passwords in the OS keyring, never in `profiles.xml`. A hosted runner has no
  keyring, so the step starts a D-Bus session and an unlocked `gnome-keyring` and passes their addresses to the
  later steps through `GITHUB_ENV`.
- `cargo xtask package --sign ci` zips the staged tree and re-signs it with `tz pack -b`. `tz pack` doesn't sign a
  directory.
- The certificates and `profiles.xml` are removed at the end, whatever happened.

## Boot

The emulator runs headless on Xvfb `:99`, which the boot step starts for the job's whole life, with GLX, **before
`em-cli create`**. `create` checks the host for OpenGL. Without a display it configures VIGS, the emulator's display
device, with its software backend (`-device vigs,backend=sw`). The TV platform's tuner device needs VIGS's window
system interface, which only the GL backend provides. So under `sw` it fails with `winsys interface 'vigs_wsi' not
found` (`Initialization of device maru-tuner-decoder failed`), and the VM exits within a second of starting. On
Xvfb with GLX, Mesa's llvmpipe gives `create` the GL it looks for. The step prints `glxinfo -B` and the VM's
`vm_launch.conf`, so a run shows which backend it got.

Ready means `sdb capability` answers with `cpu_arch`. `sdb devices` alone lists the emulator before it takes
packages. A plain `sdb shell <command>` never answers on the TV emulator: run 42911347984 booted, but its
`echo ready` check waited out the 15 minutes. TVs take only `sdb shell 0 <command>`, and the step prints what both
forms answer. The step waits up to 15 minutes (`timeout-minutes: 20` on the step). If the emulator isn't up by then, the step
prints the following itself, because a job stopped by its time limit keeps no logs:
- `sdb devices`, `em-cli list-vm` and `em-cli launch --help`,
- the emulator processes,
- the last 200 lines of `launch.log`, `xvfb.log` and the VM's logs.

`sdb dlog` waits forever when no device is attached, so the log step bounds it with `timeout 60` and
`timeout-minutes: 5`.

The rest of the emulator log is harmless on a runner: ALSA finds no sound card (`audio: Failed to create voice`),
and Xvfb warns about unknown keysyms.

## Installing

The TV emulator, like a TV, has no `pkgcmd` and no plain shell (`sdb capability`: `appcmd_support:disabled`,
`intershell_support:disabled`). So `sdb install` doesn't work for a `.tpk`: it hands the file to the web-app
installer, which logs `This path is NOT package file`, and `sdb` still exits 0 (run 42915085901). TVs install
with `sdb shell 0 vd_appinstall <package id> <path on the device>` and list apps with `sdb shell 0 vd_applist`.
`tz install` does the same. The job installs with `tz install`, falls back to pushing to
`/home/owner/share/tmp/sdk_tools/` and calling `vd_appinstall` itself, and fails unless `vd_applist` lists the
package. It launches with `sdb shell 0 execute <app id>`.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `Failed to install the following packages automatically` with `python3.8` | the prerequisite check above | the stand-in packages in `tizen-sdk` |
| `No space left on device` in `tizen-sdk` | the runner image's preinstalled tools | the Free disk space step; remove more there if it happens again |
| `wrong crypted size` from `tz pack` | a `profiles.xml` with a plain password | let `tz security-profiles add` write the profile |
| `is not a valid project` from `tz pack` | `tz pack` given a directory | `cargo xtask package --sign` does the zip and `-b` |
| `winsys interface 'vigs_wsi' not found`, `Failed to start this VM` | the VM was created without a display, so with VIGS's software backend | create it with Xvfb (GLX) running and `DISPLAY` set; see [Boot](#boot) |
| `the emulator did not come up in 15 minutes` with `emulator-26101 device` listed | a readiness check through `sdb shell <command>` | check with `sdb capability` |
| `the emulator did not come up in 15 minutes`, nothing listed | the guest is slow or stuck | read `emulator.klog` (the guest kernel's console) in the printed logs |
| `Check certificate error : :Invalid certificate chain` from `vd_appinstall`; `unrecognized root CA certificate` for `signature1.xml` in the VM's log | a distributor certificate that isn't Samsung's, or one that doesn't list the device's DUID | the Samsung certificate secrets, with the emulator's DUID `XTCJYJZXZBZVK` |
| `install app failed` from `sdb install`, then `is not available for the user 5001` at launch | `sdb install` on a TV (emulator) | `tz install`, or `sdb shell 0 vd_appinstall`; see [Installing](#installing) |
| `ls: cannot access '/dev/kvm'` in KVM and Xvfb | the runner has no KVM | a runner with KVM (a larger runner, or self-hosted) |

## The NSH fork's emulator job

`fork-tizen.yml` on `dXnSix/nsh` `fork/tizen-ci` uses this repository's `.github/actions/tizen-sdk` and the same
steps: free disk space, Xvfb with GLX before `em-cli create`, readiness by `sdb capability`, and `tz
security-profiles add` in the keyring. It checks this repository out with the job's token, which works because
`dXnSix/dXn.GpuiTizen` is public. It runs by hand (`emulator: true`, once the workflow is on the fork's `main`) or on
a pull request labelled `ci:tizen-emulator`. With the Samsung secrets on `dXnSix/nsh` it builds the x86_64 runtime
and runs `nsh test examples/counter --target tizen`. Without them it boots the emulator and skips the spec, as in
run 42944522991 on dXnSix/nsh#191.
