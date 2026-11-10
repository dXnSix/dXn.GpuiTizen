# The TV emulator on CI

How `.github/workflows/tizen-emulator.yml` runs the Samsung TV emulator on a GitHub-hosted runner, what it needs
from GitHub and the owner, and what each step works around. The NSH fork's emulator job (`fork-tizen.yml`,
`tv-specs` with `emulator: true`) has the same needs. Results of the runs go in
[`verification/tizen-devices.md`](verification/tizen-devices.md) (`CI-3`).

## What the job does

Each step and the problem it solves:

| Step | What it does | Why |
| --- | --- | --- |
| Free disk space | removes Android, GHC, CodeQL, Swift, PowerShell, Chromium and Julia from the runner image, and prunes Docker images | the SDK, the TV emulator platform and its VM image don't fit next to them: the first run stopped with `No space left on device` |
| KVM and Xvfb | gives every user access to `/dev/kvm` (udev rule) and installs Xvfb | the emulator is a QEMU VM and needs KVM; it opens a window, so it needs an X display |
| `setup`, `setup-dotnet` | Rust, Linux packages, .NET 8 | the diagnostic app's library and its C# host |
| `tizen-sdk` | installs the SDK CLI with `TIZEN-8.0-NativeAppDevelopment-CLI Emulator TV-SAMSUNG-Public-Emulator`, cached by the package list | see [SDK install](#sdk-install) |
| Signing profile | a throwaway Tizen author certificate (`tz cert`) and the SDK's public distributor certificate, in a profile `ci` | see [Signing](#signing) |
| Package | `cargo xtask package --app diag --target x86_64 --sign ci` | the emulator is x86_64 |
| Boot | `em-cli create` and `em-cli launch` on Xvfb `:99`, then waits up to 15 minutes until `sdb devices` lists the emulator and `sdb shell echo ready` answers | see [Boot](#boot) |
| Install and launch | `sdb install` (3 tries), `sdb shell app_launcher -s DxnGpuiTzD.Diag` | |
| Pull the report | `cargo xtask diag pull` (6 tries, 20 s apart) | the app needs time to start and answer |
| Emulator log, upload | the VM's logs and `sdb dlog`, uploaded as the `tizen-emulator` artifact with the report | |
| Remove the signing material | deletes the certificates and `profiles.xml` | |

## What GitHub and the owner provide

- **Actions on in `dXnSix/dXn.GpuiTizen`.** The job uses only GitHub-hosted `ubuntu-24.04` runners. Every run so far
  has had `/dev/kvm`.
- **`dXnSix/gpui-fast` and `dXnSix/wgpu` readable.** `Cargo.toml`'s git patch fetches them. Both are public.
- **No secrets.** See [Signing](#signing).
- **Starting a run.** The job runs weekly (Monday 04:41 UTC) and on demand. To run it on a branch:

  ```bash
  gh workflow run tizen-emulator.yml -R dXnSix/dXn.GpuiTizen --ref <branch>
  ```

  Starting it is the owner's acceptance of Samsung's SDK licence on the runner (`--accept-license`). The agent
  starts it on the owner's standing go.
- **Minutes.** `dXnSix/dXn.GpuiTizen` is private, so runs count against the account's Actions minutes. One run takes
  about 10 minutes until the boot, at most 20 minutes of boot and its logs, and a few minutes after. The job's
  limit is 90 minutes. If minutes run short, remove the weekly `schedule` and keep the manual trigger.
- **Cache.** `tizen-sdk` caches `~/tizen-studio` per package list. With the emulator packages that is several GB of
  the repository's 10 GB cache. GitHub evicts the oldest entries itself.

## SDK install

The package manager checks each package's OS prerequisites with `dpkg -l <name> | grep ^ii` and stops when one is
missing. The native packages list `python3.8` and `libpython3.8`. Ubuntu 24.04 has no Python 3.8, and nothing the
job runs needs it. So `.github/actions/tizen-sdk` installs empty stand-in packages with those names before
`install`. The installer's repository URL is `http://`, which the mirror answers with 403, so the action switches it
to `https://` ([`setup.md`](setup.md), step 4).

## Signing

The emulator accepts a plain Tizen certificate, so the job needs no Samsung account, no DUID and no secrets:

- The author certificate is made in the job with `tz cert` and a random password (masked in the log).
- The distributor certificate is the SDK's public one,
  `tools/certificate-generator/certificates/distributor/sdk-public/tizen-distributor-signer.p12`, with its
  published password.
- `tz security-profiles add` writes the profile `ci` to `~/tizen-studio-data/profile/profiles.xml`. It keeps the
  passwords in the OS keyring, never in `profiles.xml`. A hosted runner has no keyring, so the job starts a D-Bus
  session and an unlocked `gnome-keyring` and passes their addresses to the later steps through `GITHUB_ENV`.
- `cargo xtask package --sign ci` zips the staged tree and re-signs it with `tz pack -b`. `tz pack` doesn't sign a
  directory.

A Samsung certificate is needed only on a real TV, and it stays on the owner's machine
([`development_setup.md`](development_setup.md)).

## Boot

The emulator runs headless on Xvfb `:99`, which the boot step starts for the job's whole life. The step waits up to
15 minutes (`timeout-minutes: 20` on the step). If the emulator isn't up by then, the step prints the following
itself, because a job stopped by its time limit keeps no logs:
- `sdb devices`, `em-cli list-vm` and `em-cli launch --help`,
- the emulator processes,
- the last 200 lines of `launch.log`, `xvfb.log` and the VM's logs.

`sdb dlog` waits forever when no device is attached, so the log step bounds it with `timeout 60` and
`timeout-minutes: 5`.

Status: run 42891492722 got through signing and packaging, but the emulator didn't come up within 15 minutes. The
cause is open: run 42907330624, with the logs above, is the first run to report it. Until it's fixed, `CI-3`
accepts the recorded failure.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `Failed to install the following packages automatically` with `python3.8` | the prerequisite check above | the stand-in packages in `tizen-sdk` |
| `No space left on device` in `tizen-sdk` | the runner image's preinstalled tools | the Free disk space step; remove more there if it happens again |
| `wrong crypted size` from `tz pack` | a `profiles.xml` with a plain password | let `tz security-profiles add` write the profile |
| `is not a valid project` from `tz pack` | `tz pack` given a directory | `cargo xtask package --sign` does the zip and `-b` |
| `the emulator did not come up in 15 minutes` | see the printed logs | open; see [Boot](#boot) |
| `ls: cannot access '/dev/kvm'` in KVM and Xvfb | the runner has no KVM | a runner with KVM (a larger runner, or self-hosted) |

## The NSH fork's emulator job

`fork-tizen.yml` on `dXnSix/nsh` `fork/tizen-ci` uses this repository's `.github/actions/tizen-sdk`, so it gets
the SDK fix. Its signing step still writes a `profiles.xml` from `TIZEN_*` secrets with plain passwords, which
can't sign. It needs the same throwaway-certificate step as `tizen-emulator.yml`. That change belongs on
`fork/tizen-ci`, once this repository's emulator job boots.
