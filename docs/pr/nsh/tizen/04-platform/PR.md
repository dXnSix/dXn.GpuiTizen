# tizen: Add nsh-platform-tizen, the Samsung TV backend

Part of #(Tizen issue). **Depends on:** #(pr/tizen/01-os), #(pr/tizen/03-gpui-tizen-cfg) and #(pr/tv/01-keys), merged;
dXn.GpuiTizen [0.1.x on crates.io](https://crates.io/crates/dxn_gpui_tizen) ([OWNER: the published version]). No
nsh-compat change.

## Description

NSH apps run natively on Samsung TVs (Tizen 8.0 and later) through `crates/gpui/nsh-platform-tizen`: NSH's GPUI
backend (`nsh-platform-gpui`'s `AppView`) full screen on dXn.GpuiTizen's Tizen platform for GPUI (`dxn_gpui_tizen`:
`ui_app_main` and Ecore's main loop, one full-screen window on Ecore_Wl2, EGL and GL ES, the remote's keys, the
on-screen keyboard, the app's lifecycle, the TV's settings). So on a TV the GPUI backend is the runtime of packaged
apps, as #(Tizen issue) describes.

**The crate.** `libnsh.so` (library name `nsh`, a `cdylib` and an `rlib`, as Android's) exports one symbol,
`nsh_tizen_main(argc, argv)`, which a TPK's C# host calls with its one P/Invoke (`host/`: `Program.cs` and
`Nsh.Tizen.Host.csproj` on `Tizen.NET.Sdk`, no TizenFX). It is `publish = false`: it ships built. The GPUI
workspace takes `dxn_gpui_tizen` and `dxn_gpui_tizen_sys` with `version = "0.1.0"` (crates.io, [OWNER: the published
version]), so `release-crate.yml` can publish the workspace's other crates.

- The app comes from the TPK's resources (`res/nsh/bundle.nshc`, its assets and `app.json`, as Linux's packages hold
  them), or a newer signed package of it; data and caches go to the app framework's directories.
- The TV's settings (locale, time zone, font scale, screen reader, reduce motion) go to `AppView`'s setters before the
  app loads (so its top-level code reads the TV's values), and again as they change; the appearance is dark (Tizen 8.0 reports no light or dark setting).
- The platform's window activation, off while the app framework has the app paused, is the app's state:
  `background`, then `active` again (`AppView::set_lifecycle`).
- Back is NSH's back navigation (a modal, a stack, `onBackPress`); when nothing takes it, the app asks whether to leave,
  staying chosen (Samsung's rule), in the TV's language (English, Chinese, Korean, Japanese, Spanish, French, German,
  Portuguese, Russian), at its text size, as an alert dialog Voice Guide reads.
- The remote's key names are read as NSH's keys (#(pr/tv/01-keys)).

**The development runtime** (`--features dev`): the dev server's app with hot updates, the status line and error
overlay, logs in the developer's terminal (else the TV's dlog, tag `NSH`). A TV reaches the developer's machine through
`sdb`: first `ws://127.0.0.1:8790/ws`, which `nsh run` maps with `sdb reverse`; after three refusals it also listens on
the TV's loopback port 8791 for `sdb forward` (loopback only: `sdbd` delivers forwarded connections there, and no host
on the TV's network can pose as the dev server).

**What it needs elsewhere:**

- `AppView::set_lifecycle(Option<Lifecycle>, cx)`: the app's state over the one the windows give, for a host whose
  platform pauses the app while its window stays shown. `AppView::on_unhandled_back(f)`: a back press nothing took
  goes to `f` instead of nowhere. `nsh_platform_gpui::nsh_key(name)`: the NSH key a GPUI key name is read as.
  `Lifecycle` re-exported beside `Appearance`, `Locale` and `TimeZone`.
- `nsh-dev`: `DevTransport` makes the dev client's connections, `DevClient::start_with(url, transport, host,
  on_event)` speaks the protocol over them (retrying with the client's backoff); `AppView::connect_dev_server_with(url,
  transport, cx)`; `dev::forward(record)` sends a log record to the dev server while connected.
- The GPUI workspace lists the crate but leaves it out of `default-members`, so `gpui.yml`'s bare `cargo test` /
  `cargo clippy` build the desktop crates alone; the TV backend is built by name. `.github/ci/paths.py` reads the GPUI
  workspace too and checks `gpui.yml`'s paths, which gain `.cargo/**`.

`docs/dev/tizen.md` (new) has the crate, the host, the development runtime and the builds; `docs/dev/{embedding,gpui,
tooling}.md` and the website's GPUI page (English and zh-CN) the new `AppView` API.

## Public API

- `nsh_platform_gpui`: `AppView::set_lifecycle`, `AppView::on_unhandled_back`, `nsh_key(name: &str) -> Option<Key>`,
  re-export `Lifecycle`; with `dev`: `AppView::connect_dev_server_with(url, transport, cx)`, `dev::forward(record) ->
  bool`.
- `nsh_dev`: `DevTransport` (trait), `DevClient::start_with`.
- `nsh-platform-tizen` (new, not published): the exported `nsh_tizen_main(argc, argv)`.

## How to Test

On any desktop (with the Cargo config for the gpui-fast and wgpu forks until they are released, `docs/dev/tizen.md`):

- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-tizen --lib` (`app`: the TV's settings set before
  the app loads; `keys`, `settings`, `back`, `prompt`: the exit question's languages, text size and role; `lifecycle`,
  `package`, `host`: the C# project parsed as XML as MSBuild reads it, a double hyphen in a comment refused) and with
  `--features dev` (`dev::tests`: the connection policy and the fallback over loopback sockets, listening on the
  loopback only).
- `cargo test --manifest-path crates/gpui/Cargo.toml -p nsh-platform-gpui --lib -- host_settings::tests::lifecycle navigation::tests::unhandled_back keys::tests`.
- `cargo test -p nsh-dev --features client,server --test roundtrip` (`a_client_speaks_over_the_connections_its_transport_makes`).
- The TV libraries for armv7, x86_64 and aarch64, release and dev, with dXn.GpuiTizen's cross driver and symbol check
  (one export, `nsh_tizen_main`; only documented Tizen 8.0 symbols and glibc imported).

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): a new backend; `Os::Tizen`'s case is #(pr/tizen/01-os)'s.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (the gallery on a Samsung TV or the Tizen emulator instead: [OWNER: DEV runs]).
