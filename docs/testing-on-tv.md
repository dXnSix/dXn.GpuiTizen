# Testing on a Samsung TV and on Remote Test Lab

How the owner runs this repository's apps on a real Samsung TV, or on a TV in Samsung's Remote Test Lab, and what to
send back. This is the `s25-owner-devices` sprint of the epic. It builds on two other guides:

- [`setup.md`](setup.md): the Tizen SDK, rootstraps and toolchain on the Mac (done once, already done on this Mac).
- [`development_setup.md`](development_setup.md): developer mode, the Samsung certificate and the CI secrets. This
  guide refers to its sections instead of repeating them.

Nothing here goes to the store, to crates.io or upstream (the epic's `d35-epic-ends-before-release`).

## What you need

| What | Notes |
| --- | --- |
| A Samsung TV from 2024 or later (Tizen 8.0 or newer) | Older TVs may work but are not what the platform targets. Update its firmware first (**Settings > Support > Software Update**). |
| The TV and the Mac on the same network | Wired Ethernet on the TV gives steadier `sdb` connections. No client isolation between the two (guest Wi-Fi often has it). |
| A Samsung account signed up on [Samsung Developers](https://developer.samsung.com) | Needed for the certificate and for Remote Test Lab. Free. |
| VS Code with the **Tizen TV Extension** | `samsungtvsdk.tv-vscode-tizen` on the VS Code Marketplace. It installs the **Tizen Extension** it depends on. Only used to create the certificate. |
| The Tizen SDK CLI from [`setup.md`](setup.md) | `~/tizen-studio/tools/sdb` and `~/tizen-studio/tools/tizen-core/tz`. Not Tizen Studio, which is deprecated. |

To keep the commands short, put the SDK tools on your `PATH` for the session:

```bash
export PATH="$HOME/tizen-studio/tools:$HOME/tizen-studio/tools/tizen-core:$PATH"
```

## 1. Put the TV in developer mode and connect

Follow [`development_setup.md`, section 1](development_setup.md#1-put-the-tv-in-developer-mode). In short: **Apps**,
enter `12345`, **Developer mode** on, your Mac's IP as **Host PC IP**, then restart the TV fully. Then:

```bash
sdb connect <tv-ip>
```

```bash
sdb devices
```

Expect `<tv-ip>:26101  device  <model>`. Then note the TV's details. The first line decides `--target`
(`armv7` or `aarch64`) for every package below:

```bash
sdb capability | grep -E 'cpu_arch|platform_version|profile_name'
```

```bash
sdb shell 0 getduid
```

`getduid` prints the TV's DUID, which the certificate in step 2 must list.

Things that catch people out:

- **Samsung Instant On** (or **Quick Start**) keeps the TV in a warm standby, so turning it off with the remote is not a
  restart. After changing developer mode, hold the power button until the TV reboots, or unplug it for 30 seconds.
- **Apps installed in developer mode are removed when the TV is switched off.** Samsung's FAQ calls this intended. Keep
  the TV on during a session, and reinstall after each power cycle.
- `sdb` loses the TV when the TV sleeps or changes IP. Run `sdb connect <tv-ip>` again. A DHCP reservation for the TV in
  your router keeps its IP stable.
- Port 26101 on the TV can't be opened or changed. If `sdb connect` is refused, check **Host PC IP** in the developer
  mode dialog (some firmware stores it with its parts reversed) and the Mac's firewall.

## 2. Get a Samsung certificate

A TV installs only packages signed with a Samsung certificate whose distributor part lists that TV's DUID. Samsung
issues it through your Samsung Developers account, from Samsung's Certificate Manager, which now runs inside VS Code.
Follow [`development_setup.md`, section 2](development_setup.md#2-a-samsung-certificate-for-your-tv) with these
points from Samsung's current documentation:

1. **Use a current Samsung Certificate Extension.** Samsung: certificate creation does not work with Samsung
   Certificate Extension versions before 2.0.73 after September 2025. In VS Code's Tizen **Packages** view, update
   **TV Extension** and **Samsung Certificate Extension** to their latest versions before you start.
2. **Profile name `tv`.** The packaging commands below use it.
3. **Author certificate:** create a new one the first time. Keep its password and the `author.p12` file safe: every
   later distributor certificate, for another TV or for Remote Test Lab, is made with the same author certificate, and
   an app can only be updated with the author certificate it was first signed with.
4. **Distributor certificate:** privilege level **Public** (the apps use only the public `internet` privilege). Add
   these DUIDs:
   - your TV's, from `sdb shell 0 getduid` (step 1). With the TV connected, the wizard offers it as a checkbox. Samsung's
     certificate documentation says a DUID can also be typed in by hand;
   - the TV emulator's, `XTCJYJZXZBZVK`, so CI can install too (`development_setup.md`, section 5);
   - a Remote Test Lab TV's, if you will use one (section 6 below says how to read it).
5. **Sign in** with your Samsung account when the browser opens. The new profile appears in the Certificate Profile
   List.

Check that the CLI sees it:

```bash
tz security-profiles list
```

It should list `tv`. Back up `~/SamsungCertificate/tv/` (`author.p12`, `distributor.p12`) and the passwords outside
the repository. They are never committed.

Adding a TV later means a new distributor certificate made with the same author certificate, listing all the DUIDs you
want; then repackage.

## 3. Install and start the diagnostic app first

The diagnostic app needs no GPUI and reports what the TV's GPU stack offers, so it goes first. Use the `--target` from
step 1.

```bash
cargo xtask package --app diag --target armv7 --sign tv
```

```bash
tz install -p target/tpk/DxnGpuiTzD-0.1.0-armv7.tpk -e <tv-ip>:26101
```

```bash
sdb shell 0 execute DxnGpuiTzD.Diag
```

```bash
cargo xtask diag pull
```

`diag pull` forwards the app's port 47123 over `sdb` and saves the report and its log under `artifacts-temp/diag/`.
It uses the only connected device, so `sdb disconnect` any other (an RTL TV, the emulator) first.
Send that folder's newest files to the agent. Use `tz install`, not `sdb install`: a TV has no `pkgcmd`, and `sdb
install` reports success while the TV's web-app installer rejects the `.tpk`.

## 4. The examples

| App | `--app` | Package | Start with |
| --- | --- | --- | --- |
| Counter | `counter` | `DxnGpuiTzC-0.1.0-<target>.tpk` | `sdb shell 0 execute DxnGpuiTzC.Counter` |
| Long list | `long_list` | `DxnGpuiTzL-0.1.0-<target>.tpk` | `sdb shell 0 execute DxnGpuiTzL.LongList` |
| Video | `video` | `DxnGpuiTzV-0.1.0-<target>.tpk` | `sdb shell 0 execute DxnGpuiTzV.Video` |

Package, install and start each one as in step 3, for example:

```bash
cargo xtask package --app long_list --target armv7 --sign tv
```

```bash
tz install -p target/tpk/DxnGpuiTzL-0.1.0-armv7.tpk -e <tv-ip>:26101
```

```bash
sdb shell 0 execute DxnGpuiTzL.LongList
```

They also appear in **Apps** on the TV, so you can start them with the remote. Check that one is installed with:

```bash
sdb shell 0 vd_applist | grep DxnGpuiTz
```

The platform logs to `dlog` under the tag `DXN_GPUI_TIZEN`. Retail TVs usually refuse `sdb dlog`; if yours allows it,
save the log while you test:

```bash
sdb dlog -v time DXN_GPUI_TIZEN:V > artifacts-temp/tv-dlog.txt
```

### What to check, and what to write down

For each app, note what worked and what didn't. Photos or phone videos of the screen help when something looks wrong.

**Counter and long list (`DEV-2`):**

- [ ] The app fills the screen at 1920x1080, sharp text, no black borders.
- [ ] Arrow keys move the focus, **Enter** activates, **Back** leaves (or goes back).
- [ ] The color keys (red, green, yellow, blue) and the media keys (play, pause, stop, rewind, fast forward) reach the
  app. The counter shows the name of the last key it received under its buttons (`DEV-8`, the epic's `t25-key-echo`,
  added before this run).
- [ ] Long list: holding an arrow key scrolls smoothly, and keeps scrolling past the first screens of rows.
- [ ] Long list: selecting the filter field opens the TV's on-screen keyboard, and typing filters the list.
- [ ] **Home** leaves the app; starting it again (or choosing it in the recent apps) shows it redrawn correctly, not
  black or stale.
- [ ] Change the system font size (**Settings > General > Accessibility**, or **Settings > General > System Manager**
  depending on the model) and the menu language, then start the app: the change shows.

**Voice Guide (`PLAT-7`):** turn on **Settings > General > Accessibility > Voice Guide Settings > Voice Guide**, start
the counter and the long list, and move the focus. Write down whether the focused button or row is read aloud, and what
is read. Turn Voice Guide off afterwards.

**Video (`VID-4`):**

- [ ] The clip plays full screen; Left and Right move between the controls drawn over the video; **Enter** uses one.
- [ ] The controls stay visible over the playing video.
- [ ] **Home** pauses it; returning resumes or shows it paused, without a crash.

The video plays on the TV's video plane by default (the overlay). The texture output, and the second renderer path the
measurements need, need launch settings inside the package (`DEV-7`, the epic's `t25-launch-settings`). Once the agent
has added them, a package picks them like this:

```bash
cargo xtask package --app video --target armv7 --sign tv --env DXN_GPUI_TIZEN_VIDEO_OUTPUT=texture
```

```bash
cargo xtask package --app long_list --target armv7 --sign tv --env DXN_GPUI_TIZEN_RENDERER=wgpu
```

**Measurements (`DEV-3`):** the diagnostic report (step 3) carries launch timings and memory. The agent reads the rest
(library size, first-frame time, memory after scrolling, frame times under key repeat) from the reports and logs you
send, for the default renderer and for `DXN_GPUI_TIZEN_RENDERER=wgpu`, with the shader cache cold (first start after
install) and warm (second start).

### What to send back

- The files `cargo xtask diag pull` saved under `artifacts-temp/diag/`.
- The TV's model (**Settings > Support > About This TV**), `sdb capability`'s `cpu_arch` and `platform_version`.
- The checklists above with notes, and any photos or `dlog` output.

The agent records them in [`verification/tizen-devices.md`](verification/tizen-devices.md) and turns failures into
corrective tasks.

## 5. The NSH apps (`DEV-4`, `DEV-5`, `DEV-6`)

After the examples work: the NSH counter with `nsh run --target tizen` and hot reload, the NSH gallery packaged with
`nsh package --target tizen`, and the `tizen-web` loader probe, then the gallery as a `.wgt`. They use the same TV
connection and the same `tv` profile. The agent gives the exact commands from the NSH fork's `docs/dev/` when you get
there, since they depend on `nsh component add tizen` from the fork's build.

## 6. Remote Test Lab

Remote Test Lab (RTL) lends you a real Samsung TV in one of Samsung's labs, controlled from the browser. It needs only
your Samsung account. On 2026-11-13 its TV list had 2024 models (`QN55Q80D`, `QE55Q80D`, `QA55Q80D`, Tizen 8.0), a 2025
`QN55QN80F` and a 2026 `QN55QN80H`, next to older ones. The 2024 ones are the platform's target.

### Sign up and reserve

1. Sign in at [developer.samsung.com](https://developer.samsung.com) with your Samsung account (the same one as for the
   certificate). If the account isn't signed up on Samsung Developers yet, the site asks you to accept its terms.
2. Open [Remote Test Lab > TV](https://developer.samsung.com/remotetestlab/devices/117/tv) and pick a model, preferably
   a 2024 one. Choose the lab location nearest to you: latency grows with distance.
3. Reserve it. The reservation dialog shows how long you get; Samsung doesn't publish the limits for TVs, and its
   [Partner Program](https://developer.samsung.com/RemoteTestLab/business-partner/m8y15asikit62fek) offers longer
   reservations. The RTL client opens in the browser with the TV's screen and a remote.
4. Samsung takes the TV lab down for maintenance now and then (a notice on the TV page announced one from October 30
   to November 2, 2026).

### Connect `sdb`

The RTL client forwards the TV's `sdb` to a port on your Mac:

1. In the RTL client, right-click the TV's screen (or open its menu) and choose **Remote Debug Bridge**. It shows a
   command such as `sdb connect localhost:<port>`. Copy it.
2. Run it from the Mac, with the SDK tools on your `PATH`. If it fails, run it again a few times; the tunnel can take a
   moment.

   ```bash
   sdb connect localhost:<port>
   ```

3. Check it, and read the TV's DUID:

   ```bash
   sdb devices
   ```

   ```bash
   sdb shell 0 getduid
   ```

   ```bash
   sdb capability | grep -E 'cpu_arch|platform_version'
   ```

If the connection keeps failing, check that the Mac's date and time are set automatically: an answer on Samsung's
developer forum blames clock differences between the RTL device and the PC.

### Certificate for an RTL TV

The RTL TV is a different device with its own DUID, so your distributor certificate must list it. Create a new
distributor certificate in VS Code's Certificate Manager with the same author certificate (section 2) and type in the
RTL TV's DUID together with your TV's and the emulator's. Reserving the same device (the same device name in the list)
next time keeps the DUID the same; a different device needs its DUID added again. Then repackage with `--sign tv`.

### Install and test

The same commands as sections 3 and 4, with `localhost:<port>` instead of `<tv-ip>:26101`:

```bash
tz install -p target/tpk/DxnGpuiTzD-0.1.0-armv7.tpk -e localhost:<port>
```

```bash
sdb -s localhost:<port> shell 0 execute DxnGpuiTzD.Diag
```

```bash
cargo xtask diag pull
```

Use the RTL client's remote for the checklists (it has arrows, Enter, Back, Home and usually the color and media keys),
and its screenshot and video recording to capture results. Voice Guide is hard to judge through the browser's audio, so
leave `PLAT-7` to your own TV. Release the reservation when you are done.

What's not confirmed yet, because Samsung's TV documentation for RTL is thin: the exact menu name of **Remote Debug
Bridge** in the TV client, whether `cargo xtask diag pull`'s port forward works through the RTL tunnel, and whether
the RTL TVs allow `getduid`. Tell the agent what you see and it updates this section.

## 7. The emulator

The TV emulator runs on CI (`tizen-emulator.yml`, [`ci-emulator.md`](ci-emulator.md)), because it needs an x86_64
machine with KVM. Once the four `TIZEN_*` secrets are set from your certificate (with the emulator's DUID,
[`development_setup.md`, section 5](development_setup.md#5-repository-secrets)), ask the agent to run the job: it
installs the diagnostic app and pulls the report.

## Troubleshooting

| What you see | Cause | Fix |
| --- | --- | --- |
| `sdb connect` refused or times out | developer mode off, wrong **Host PC IP**, the TV only in standby, or a firewall | step 1: check the IP, restart the TV fully, allow incoming connections for `sdb` |
| `tz install` reports a signature or certificate error | the distributor certificate doesn't list this TV's DUID, or the package was signed with another profile | step 2 with the DUID from `sdb shell 0 getduid`, then repackage with `--sign tv` |
| An install error mentioning the architecture | wrong `--target` | use the `cpu_arch` from `sdb capability` |
| The app was there yesterday and is gone | the TV was switched off; developer-mode installs don't survive that | reinstall |
| The app starts and closes at once | a crash at start | send `sdb dlog` output if the TV allows it, and the diag report |
| Certificate Manager can't create a certificate | an extension older than 2.0.73, or the Samsung sign-in didn't finish | update the extensions in VS Code's Tizen **Packages** view and try again |

## Sources

- Samsung, [Connecting to a TV device](https://developer.samsung.com/smarttv/develop/getting-started/using-sdk/tv-device.html)
  and, for VS Code, [Connecting to TV Device](https://developer.samsung.com/smarttv/develop/tools/additional-tools/vscode/connecting-to-tv-device.html).
- Samsung, [Creating Certificates](https://developer.samsung.com/smarttv/develop/getting-started/setting-up-sdk/creating-certificates.html)
  and, for VS Code, [Creating Certificates](https://developer.samsung.com/smarttv/develop/tools/additional-tools/vscode/creating-certificates.html).
- Samsung, [Installing TV SDK](https://developer.samsung.com/smarttv/develop/getting-started/using-sdk/) (the 2.0.73
  notice).
- Samsung, [Application Testing Q&A](https://developer.samsung.com/smarttv/develop/faq/application-testing.html)
  (developer-mode apps removed at power-off, Instant On, error codes).
- Samsung, [Remote Test Lab](https://developer.samsung.com/remote-test-lab) and its
  [TV device list](https://developer.samsung.com/remotetestlab/devices/117/tv).
- Samsung's VS Code extension wiki, [Setting for "RTL TV"](https://github.com/Samsung/vscode-extension-tizentv/wiki/Setting-for-%22RTL-TV%22-in-%22Tizentv-extension%22),
  and the blog post [Troubleshooting Common Issues While Using the Remote Test Lab Service](https://developer.samsung.com/sdp/blog/en-us/2022/08/23/troubleshooting-common-issues-while-using-the-remote-test-lab-service).
