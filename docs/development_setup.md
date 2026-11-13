# Development setup: signing, the TV and CI secrets

What the owner sets up by hand before a Tizen app built here runs on a Samsung TV: the certificates, the TV's
developer mode, and the repository's secrets. [`setup.md`](setup.md) covers the SDK, rootstraps and toolchain,
which come first. [`testing-on-tv.md`](testing-on-tv.md) then runs the apps on a TV or a Remote Test Lab TV. None of
them uses Tizen Studio.

| Where the app runs | Certificate | Who sets it up |
| --- | --- | --- |
| a real Samsung TV | a Samsung certificate whose distributor lists the TV's DUID | you, once per TV ([Samsung certificate](#2-a-samsung-certificate-for-your-tv)) |
| the TV emulator, locally or in CI (`tizen-emulator.yml`) | the same Samsung certificate, its distributor also listing the emulator's DUID `XTCJYJZXZBZVK` | you, in the same step; CI gets it as secrets ([section 5](#5-repository-secrets), [`ci-emulator.md`](ci-emulator.md)) |

The Samsung TV emulator trusts only Samsung's distributor roots, like a TV. A plain Tizen certificate, which `tz
cert` makes without an account, is rejected (`signature1.xml has got unrecognized root CA certificate`). All of
this is the owner's work at the end of the epic, together with the TV. Until then, the emulator job boots the
emulator and skips the install.

Certificates, `profiles.xml`, `.p12` and `.pwd` files are never committed. `.gitignore` excludes them, and
everything below keeps them under your home directory.

## 1. Put the TV in developer mode

The TV and your Mac must be on the same network.

1. Find your Mac's IP address:

   ```bash
   ipconfig getifaddr en0
   ```

   Use `en1` instead if the Mac uses a different interface. On Linux, use `hostname -I`.
2. On the TV, open **Smart Hub** and go to **Apps**.
3. In **Apps**, enter `12345` with the remote's number keys. On a remote without number keys, use the on-screen
   keypad, which the **123** button opens. On some models you enter the code in the **App Settings** panel at the
   bottom of **Apps** instead. The **Developer mode** dialog opens.
4. Set **Developer mode** to **On**. Enter your Mac's IP from step 1 as **Host PC IP**, then select **OK**.
   - Some firmware saves the IP with its parts in reverse order. If `sdb connect` is refused later, check the IP
     here.
5. Restart the TV completely. Standby is not enough: hold the power button until the TV restarts, or unplug it for
   30 seconds.
6. After the restart, **Apps** shows **Develop Mode** at the top. The TV's own IP is under **Settings > General >
   Network > Network Status > IP Settings**.
7. Connect from the Mac. `sdb` is in the SDK's `tools/`:

   ```bash
   ~/tizen-studio/tools/sdb connect <tv-ip>
   ```

   ```bash
   ~/tizen-studio/tools/sdb devices
   ```

   `devices` should list `<tv-ip>:26101` with the state `device`. If the connection fails, restart the TV fully
   again (unplug it) and check the **Host PC IP**. A firewall on the Mac can also block port 26101.
8. Note the TV's CPU architecture. It picks the package's `--target` later: `armv7` or `aarch64`.

   ```bash
   ~/tizen-studio/tools/sdb capability | grep -E 'cpu_arch|platform_version'
   ```

Developer mode stays on until you turn it off in the same dialog. A factory reset also turns it off.

## 2. A Samsung certificate for your TV

A real TV installs only packages signed with a **Samsung** certificate. Samsung issues one through its developer
account, and its distributor part lists the DUIDs (device ids) of the TVs it may install on. It cannot be created
from the command line. Samsung's Certificate Manager is the only way, and it now ships as a VS Code extension
instead of Tizen Studio.

You need:
- A Samsung account that is signed up on [Samsung Developers](https://developer.samsung.com).
- The TV in developer mode and connected with `sdb` (step 1). The certificate wizard reads the DUID from the
  connected TV.

Steps:

1. Install [VS Code](https://code.visualstudio.com), then the **Tizen TV** extension (publisher **samsungtvsdk**,
   id `samsungtvsdk.tv-vscode-tizen`). It also installs the **Tizen** extension it depends on.

   ```bash
   code --install-extension samsungtvsdk.tv-vscode-tizen
   ```

2. In VS Code, open the **Tizen** activity bar and click **Packages**. Under **Advanced**, install the latest **TV
   Extension** (it includes the Samsung certificate support). If the extension asks where the SDK lives, point it
   at `~/tizen-studio`, the SDK from [`setup.md`](setup.md), so that `tz` and VS Code share one `profiles.xml`.
3. Connect the TV in VS Code. Under **Actions**, choose **Connect Device**, then **+ Create Custom Connection**.
   Enter a name, the TV's IP and port `26101`, click **Add**, then select the TV and click **Connect**.
   Samsung: from September 2025, certificates can only be created with Samsung Certificate Extension 2.0.73 or later,
   so update the extensions in **Packages** first.
4. Under **Actions**, choose **Create Certificate**.
   1. **Profile Name**: `tv`. The packaging commands below use this name.
   2. Choose **Create Samsung Certificate**.
   3. **Author Details**: your name and a password. Keep the password in your password manager. You need it again
      when you create a new distributor certificate for another TV, and the author certificate is what identifies
      you to the store.
   4. **Distributor Details**: privilege level **Public**. The apps here use only
      `http://tizen.org/privilege/internet`, which is public, so **Partner** isn't needed. Keep the default
      version.
   5. Tick the **DUID** of your TV. Add the TV emulator's DUID as well: `XTCJYJZXZBZVK`. Every fresh TV emulator
      VM has reported this DUID, on every CI runner so far (`sdb shell 0 getduid`). If the wizard only offers the
      DUIDs of connected devices, also connect a running TV emulator, which needs an x86_64 machine with KVM. Or
      ask the agent to find another way.
   6. Click **Create** and sign in with your Samsung account when the browser asks.
5. Check that `tz` sees the profile:

   ```bash
   ~/tizen-studio/tools/tizen-core/tz security-profiles list
   ```

   It should list `tv`. The profile lives in `~/tizen-studio-data/profile/profiles.xml`, and the certificates
   usually live in `~/SamsungCertificate/tv/` (`author.p12`, `distributor.p12`). `tz` keeps their passwords in the
   macOS keychain (service `certificate-manager`), not in `profiles.xml`. Back up the `.p12` files and the author
   password outside the repository.

Adding another TV later: create a new distributor certificate with the same author certificate and both DUIDs.
In **Create Certificate**, reuse the existing author certificate.

## 3. The emulator

Use the profile `tv` from step 2 for the emulator too. Its distributor must list `XTCJYJZXZBZVK`. A Tizen
certificate (`tz cert` with the SDK's public distributor) packages and signs, but the TV emulator won't install
it. The SDK's older public distributor, `tizen-distributor-signer.p12`, also expired in 2012.

## 4. Install the diagnostic app on the TV

With the TV connected (step 1) and the profile `tv` (step 2). Use `--target aarch64` if `sdb capability`
reported an `aarch64` CPU.

```bash
cargo xtask package --app diag --target armv7 --sign tv
```

This builds the release library and the C# host, stages the tree and signs it into
`target/tpk/DxnGpuiTzD-<version>-armv7.tpk`, where the version comes from `Cargo.toml` (`0.1.0` today). Then install it, start it, and pull its report:

```bash
~/tizen-studio/tools/tizen-core/tz install -p target/tpk/DxnGpuiTzD-0.1.0-armv7.tpk -e <tv-ip>:26101
```

Check that the TV lists it:

```bash
~/tizen-studio/tools/sdb shell 0 vd_applist | grep DxnGpuiTzD
```

Don't use `sdb install` on a TV. A TV has no `pkgcmd`, so `sdb install` hands a `.tpk` to the web-app installer,
which rejects it, and `sdb` still reports success. `tz install` uses the TVs' installer (`sdb shell 0
vd_appinstall`), as the emulator job does ([`ci-emulator.md`](ci-emulator.md#installing)).

```bash
~/tizen-studio/tools/sdb shell 0 execute DxnGpuiTzD.Diag
```

```bash
cargo xtask diag pull
```

`diag pull` forwards the app's port over `sdb` and saves the report and its log under `artifacts-temp/diag/`.
Send those to the agent, which records the results in [`verification/tizen-devices.md`](verification/tizen-devices.md).

If `tz install` fails, or `vd_applist` doesn't list the package:

| What it reports | Cause | Fix |
| --- | --- | --- |
| a signature or certificate error | the package isn't signed with a Samsung certificate that lists this TV's DUID | step 2, with this TV's DUID ticked |
| an architecture or "not supported" error | wrong `--target` | step 1.8 |
| no device found | `sdb` lost the TV | `sdb connect <tv-ip>` again; restart the TV fully |

## 5. Repository secrets

The emulator job (`tizen-emulator.yml`) installs the diagnostic app only when it has the Samsung certificate from
step 2, with the emulator's DUID. It reads four secrets:

| Secret | Value |
| --- | --- |
| `TIZEN_AUTHOR_P12` | `author.p12`, base64-encoded |
| `TIZEN_AUTHOR_PASSWORD` | the author certificate's password |
| `TIZEN_DISTRIBUTOR_P12` | `distributor.p12`, base64-encoded |
| `TIZEN_DISTRIBUTOR_PASSWORD` | the distributor certificate's password |

The job adds them to a profile with `tz security-profiles add`, which keeps the passwords in a keyring it unlocks.
`profiles.xml` can't carry passwords, so there is no `TIZEN_PROFILES_XML` secret. If you created one, delete it.
Without the secrets, the job signs with a throwaway Tizen certificate, boots the emulator, and skips the install with
a warning.

Set them from the certificate folder of step 2. `gh secret set` reads a value from standard input, so neither the
files nor the passwords land in your shell history:

```bash
base64 -i ~/SamsungCertificate/tv/author.p12 | gh secret set TIZEN_AUTHOR_P12 -R dXnSix/dXn.GpuiTizen
```

```bash
base64 -i ~/SamsungCertificate/tv/distributor.p12 | gh secret set TIZEN_DISTRIBUTOR_P12 -R dXnSix/dXn.GpuiTizen
```

```bash
gh secret set TIZEN_AUTHOR_PASSWORD -R dXnSix/dXn.GpuiTizen
```

```bash
gh secret set TIZEN_DISTRIBUTOR_PASSWORD -R dXnSix/dXn.GpuiTizen
```

The last two prompt for the value. Samsung's wizard may use one password for both certificates. Then ask the agent
to run the emulator job.

Publishing to crates.io (`release-plz.yml`) needs two more. Nothing is published until the platform is stable and
tested on a TV, the emulator and Remote Test Lab, so add them only then:

| Secret | Value | Where to get it |
| --- | --- | --- |
| `CARGO_REGISTRY_TOKEN` | a crates.io API token with the `publish-new` and `publish-update` scopes | [crates.io/settings/tokens](https://crates.io/settings/tokens) |
| `RELEASE_PLZ_TOKEN` | a GitHub fine-grained token for `dXnSix/dXn.GpuiTizen` with **Contents** and **Pull requests** read and write | **GitHub > Settings > Developer settings > Fine-grained tokens** |

In the browser, secrets are under **Repository > Settings > Secrets and variables > Actions > New repository
secret**.
