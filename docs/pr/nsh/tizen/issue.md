# Samsung TVs: Os::Tizen, a native `tizen` target on the GPUI backend, and a `tizen-web` target

> Draft for longbridge/nsh, not posted. Post only with the owner's go, once a gpui-fast release has the TV changes and
> dXn.GpuiTizen is on crates.io (fill in the versions below), and before any `pr/tizen/*` or `pr/tizen-web/*` PR. The
> branches are built ahead of the answer and change in place if it differs.

We would like NSH apps to run on Samsung TVs (Tizen 8.0 and later, 2024 models on, and the Tizen TV emulator), from the
same app code, in two ways:

| | `tizen` (native) | `tizen-web` |
|---|---|---|
| What ships | a TPK: a C# host that calls `libnsh.so` | a `.wgt`: the web runtime with a TV shim |
| Runtime | **the GPUI backend** (`AppView`) on a Tizen platform for GPUI | `nsh-platform-web` in the TV's browser engine (Chromium 108 on 2024 TVs) |
| `platform.os` | `"tizen"` (`Os::Tizen`) | `"web"` |
| Idiom | TV | TV, from the page |
| `nsh run` / `nsh test` | over `sdb`, with hot reload | a development `.wgt` opening the dev server's page; headless Chromium for tests |

The TV-agnostic parts are separate issues: directional focus (#(focus issue)) and the remote's keys and the TV
category (#(TV issue)).

## 1. GPUI would be the native backend of packaged apps

Today NSH's GPUI backend is a desktop and development host. On Tizen it would be what packaged apps run on: there is no
other native backend for Tizen, and drawing with Tizen's own toolkits (EFL, or NUI on DALi's C++) would be a backend
of its own, written from scratch; GPUI already draws NSH's views, over GL ES on a TV. `nsh-platform-tizen` runs `AppView` full screen on a GPUI platform for Tizen and handles what a TV app framework
needs (its lifecycle, the TV's settings, Back and Samsung's exit rule, the remote's keys, packaged bundles).

That makes GPUI's correctness a shipping concern on that platform: accessibility (Voice Guide reads AT-SPI), text input
(the TV's on-screen keyboard), video (the TV's player, on a hardware plane under the window), memory (a 300 / 380 MB
budget per app, GPU included) and start-up time (a watchdog). **Are you open to the GPUI backend being the runtime of
packaged apps on a platform**, with what that asks of `nsh-platform-gpui` (the small API below, a Tizen cfg)?

## 2. The Tizen platform for GPUI lives outside NSH

[dXn.GpuiTizen](https://github.com/dXnSix/dXn.GpuiTizen) ([OWNER: link once public]) is a separate repository,
MIT OR Apache-2.0: Tizen target specs and cross builds, bindings to documented Tizen 8.0 C APIs only (checked against
an allow-list, one export per library), the GPUI platform (`dxn_gpui_tizen`: `ui_app_main` and Ecore's loop,
Ecore_Wl2, EGL / GL ES, keys, IME, lifecycle, settings, key grabs, system info, the video player), a C# host template
and packaging. Nothing of NSH is in it. NSH would depend on it:

- `nsh-platform-tizen` (not published: it ships built, as the CLI's `tizen` component) depends on it;
- `nsh-platform-gpui`, which NSH publishes, depends on it on Tizen only (`cfg(target_vendor = "tizen")`, for videos).
  `cargo publish` checks target-specific dependencies too, so **dXn.GpuiTizen must be on crates.io before the first NSH
  release that has it**. Today the branches pin it by git `rev` with `version = "0.1.0"`; they would take the
  crates.io version only ([OWNER: dxn_gpui_tizen version]).

## 3. gpui-fast and wgpu changes

GPUI on a TV needs six renderer changes in gpui-fast (GL ES 3.0 devices without storage buffers in vertex shaders,
`WGPU_BACKEND`, a pipeline cache on disk, scene damage, drawing into a target the platform supplies with partial
presents, and holes for the video plane), proposed to longbridge/gpui-fast separately ([OWNER: gpui-fast issue/PR
links]), and two in wgpu's GL ES backend (EGL 1.4 drivers on Wayland, gfx-rs/wgpu#10708; GL program binaries for the
pipeline cache) ([OWNER: wgpu PR links]). Until they are released the TV build patches gpui-fast and wgpu from forks
through a local Cargo config that is never committed, and `nsh-platform-tizen` stays out of the GPUI workspace's
`default-members`, so NSH's CI builds without it. One more, in NSH: the engine's shaders on wgpu 30, so the GPUI
workspace holds one wgpu (#(pr/engine/01-wgpu-30)).

## 4. What changes in NSH

- **`Os::Tizen`**: `"tizen"`, not desktop, a TV by default (`Idiom::default_on`); a Tizen build is
  `target_os = "linux"` with `target_vendor = "tizen"`. `tizen-web` stays `Os::Web`.
- **`nsh-platform-gpui`**: `gpui_platform` behind a new default feature `platform` (the TV host brings its own
  platform); a Tizen cfg that leaves the Linux desktop's code and dependencies out; `AppView` setters for the TV's
  settings (`set_reduce_motion`, `set_screen_reader`, `set_time_zone`), `set_lifecycle`, `on_unhandled_back`,
  `connect_dev_server_with` (a dev client over connections the host makes, here `sdb`).
- **`nsh-platform-tizen`**: the backend and its development runtime; videos through the TV's player.
- **`nsh/tizen`**: product information, the remote's keys (`registerKey` / `unregisterKey`), `exitApp`,
  `setPreviewData` (Smart Hub); unavailable elsewhere.
- **CLI**: `--target tizen` and `--target tizen-web` for `run`, `package`, `dev`, `test`; an `nsh.json` `tizen` section
  (`package`, `appId`, `version`, `apiVersion`, `privileges`, `signing`); signing with the Tizen SDK's `tz` and a
  certificate profile that is never committed; `cargo xtask tizen` and a `tizen` component.
- **Web**: `scrollend` emulated where the engine lacks it (Chromium < 114); `nsh/tizen` from the TV's web APIs.

## Questions

1. **GPUI as a shipping backend on Tizen** (section 1): yes, no, or under conditions (a support tier, a CI job)?
2. **The dependency** on an external crate for the platform (section 2): acceptable once it is on crates.io? Would you
   rather it lived under longbridge?
3. **`Os::Tizen`** as its own OS rather than `Os::Linux` with a TV idiom: Tizen apps use a different app model,
   packaging and key set, and must not get desktop behaviour.
4. **`nsh/tizen`** as a module of its own, as `nsh/android` is "what only Android has", or inside an existing one?
5. **Two targets**: native first, `tizen-web` as well? The web target depends on how the TV's engine loads module
   scripts, import maps and Wasm from package files, which a loader probe (`nsh package --target tizen-web --probe`)
   checks on a TV before that PR ([OWNER: probe results]).
6. **CI**: upstream has no Tizen job; the fork runs one (cross builds for armv7, x86_64 and aarch64, the TV idiom's
   specs, the emulator by hand). Would you want one upstream, and on what runner?
7. **Panics**: TV libraries keep unwinding (NSH catches panics in views, ops and SVG decoding); fine?

## Numbers from the TV

[OWNER: DEV-3 / DEV-5: library size, launch to first frame, memory after start and after scrolling, frame times, on
<device>]

## The PRs

| # | Branch | What | Needs |
|---|---|---|---|
| 1 | `pr/tizen/01-os` | `Os::Tizen` | #(pr/tv/02-category); nsh-compat `platform tizen` |
| 2 | `pr/tizen/02-gpui-platform-optional` | the `platform` feature | – |
| 3 | `pr/tizen/03-gpui-tizen-cfg` | `nsh-platform-gpui` builds for Tizen | 2 |
| 4 | `pr/tizen/04-platform` | `nsh-platform-tizen` and its dev runtime | 1, 3, #(pr/tv/01-keys); dXn.GpuiTizen on crates.io |
| 5 | `pr/tizen/05-video` | videos through the TV's player | 4 |
| 6 | `pr/tizen/06-module` | `nsh/tizen` | 4 |
| 7 | `pr/tizen-web/01-package` | `--target tizen-web`, the TV shim, the loader probe | #(pr/tv/01-keys), #(pr/tv/02-category) |
| 8 | `pr/tizen/07-cli` | `--target tizen` | 5, 6, 7 |
| 9 | `pr/tizen-web/02-scrollend` | `scrollend` emulated | – |
| 10 | `pr/tizen-web/03-module` | `nsh/tizen` on the web | 6, 7 |
