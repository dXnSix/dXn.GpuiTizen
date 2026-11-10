# cli: Run, package and test apps natively on Samsung TVs

Closes #(Tizen issue) for the native target. **Depends on:** #(pr/tizen/05-video), #(pr/tizen/06-module) and
#(pr/tizen-web/01-package) (the `nsh.json` `tizen` section), merged. No nsh-compat change.

## Description

`--target tizen` for `nsh run`, `nsh package`, `nsh dev` and `nsh test` runs apps in NSH's Tizen runtime
(`nsh-platform-tizen`'s `libnsh.so`, started by a TPK's C# host).

**The runtime and its component.** `cargo xtask tizen [--variant release|dev|both] [--arch armv7l,...] [--config
<file>]` builds it into `target/tizen`, as `cargo xtask android` builds Android's: for each architecture (armv7l,
aarch64, x86_64) through dXn.GpuiTizen's cross driver (`cargo xtask cross ... -p nsh-platform-tizen --check-symbols
--entry nsh_tizen_main`, in its checkout: `DXN_GPUI_TIZEN`, else `../dXn.GpuiTizen`), the C# host (`dotnet build -c
Release`), the libraries stripped with the nightly's `llvm-strip` (the export stays), and dXn.GpuiTizen's target specs
with the nightly they build with, for apps' Rust crates. `cargo xtask dist` packs it as `runtime/tizen/`
(`NSH_DIST_NO_TIZEN` leaves it out), with dXn.GpuiTizen's symbol allowlist beside the specs
(`runtime/tizen/targets/symbols.allow`), which `nsh component pack` makes the `tizen` component, the same for every
host; the CLI adds it the first time a command needs it.

**`nsh package --target tizen [--arch armv7l,aarch64,x86_64] [--sign <profile>] [--run]`** stages the TPK (the host,
the release library, the bundle, capabilities, `app.json` and assets in `res/nsh/`, the icon, a `tizen-manifest.xml`
made from `nsh.json`) and signs it with `tz pack -t tpk -s <profile>` into `<out>/<name>.tpk`; without a profile it
refuses, saying how to make one. With several architectures each package gets the fourth version number (armv7l 1,
aarch64 2, x86_64 3) and its CPU feature, and the TVs' packages (armv7l, aarch64; not the emulator's x86_64) go into
the multi-architecture `<package>-<version>.zip` for Samsung's Seller Office. `--run` installs and launches the TV's
architecture's package.

**`nsh run --target tizen [--device <serial>]`** builds and signs the development TPK when the runtime changed,
installs it with `sdb install` when the device lacks this build, maps the TV's port 8790 to the dev server with `sdb
reverse` (else `sdb forward` to its 8791 and a relay) and launches it (`0 execute` on a TV, `app_launcher` on the
emulator); the run ends stopping it and removing the mappings. **`nsh test --target tizen [--device <serial>]`**
installs the same package and runs each app a test opens there over the dev protocol, one at a time (`open`'s width
and height are refused: the app fills the screen). Both start the runtime through the same typed steps with their
failure policy (`tizen::open_sequence`: a stop that fails is fine, a refused `sdb reverse` falls back to `sdb forward`
and the relay, a failing launch ends the run, and what was mapped is undone also when a later step fails). With
several devices `--device` picks one by its `sdb` serial; the other targets refuse it. **`nsh dev`** names `nsh run
--target tizen` for a Tizen host, which its hello now names (`HostPlatform::Tizen`; older servers read it as `Other`).

**`nsh.json` `tizen`** is `tizen-web`'s (`package`, `appId`, `version`, `apiVersion`, `privileges`, `signing`); the
native target checks its values too.

**Apps with Rust crates** get their own `libnsh.so`, as on Android: `.nsh/rust/tizen`, a workspace whose `cdylib`
links `nsh-platform-tizen` (without its default feature `entry`, so its own `nsh_tizen_main` is left out) and the
app's crates, and defines the library's one export, `nsh_tizen_main`, giving the runtime the app's UI sets and running
it (or a Rust app's root). It is built with the component's target specs (the nightly they name, `-Zbuild-std`, clang
with the Tizen 8.0 rootstrap, the SDK's GCC 9.2 linker) and stripped; the CLI checks each library itself, as
dXn.GpuiTizen's symbol check does but without its checkout, against the component's `symbols.allow` (`tizen::symbols`,
with the nightly's `llvm-nm` and `llvm-readobj`: exactly one defined export, nothing exported weak, imports only of the
allowlist, glibc, `libgcc_s` and the loader's weak symbols, needed libraries only those). It says what to install when
the rootstrap, toolchain or nightly is missing. `nsh-platform-tizen` is not on crates.io, so such apps build against a
checkout of nsh (`nsh init --template rust --nsh-source <dir>`), and an app that takes nsh from crates.io is refused
for `--target tizen` up front, saying so.

**Panics.** dXn.GpuiTizen builds its TV libraries with `panic = "abort"`; NSH's keep unwinding, a documented
exception (`docs/dev/tizen.md`, "Panics"): NSH catches panics in what a TV library runs (a view's Rust code and its
`catch_error` boundaries, a module's op, SVG decoding, the engine's host channel), and abort would end the app
instead.

Docs: `docs/dev/tizen.md` (both targets, what apps see, the CLI, panics, the public API, limits); the website's new
Tizen page and its CLI, platforms, installation and `nsh.json` tables (English and zh-CN).

## Public API

- CLI: `--target tizen` for `nsh run`, `nsh package`, `nsh dev`, `nsh test`; `nsh package --target tizen --arch
  <list> --sign <profile> --run`; `nsh run` / `nsh test --target tizen --device <serial>`.
- `nsh.json`: the `tizen` section (from #(pr/tizen-web/01-package)) is read by `--target tizen` too.
- The `tizen` component (`nsh component`), with `runtime/tizen/targets/symbols.allow`; `cargo xtask tizen`; `cargo xtask
  dist`'s `runtime/tizen/`, `NSH_DIST_NO_TIZEN`, `NSH_TIZEN_CARGO_CONFIG`, `DXN_GPUI_TIZEN` (NSH's own runtime build
  only: apps with Rust crates need no dXn.GpuiTizen checkout).
- `nsh_dev::protocol::HostPlatform::Tizen`.
- `nsh_platform_gpui::AppView::connect_dev_server_with(url, transport, platform, cx)` (feature `dev`): takes the
  `HostPlatform` its host reports; supersedes #(pr/tizen/04-platform)'s three-argument `(url, transport, cx)`, which no
  release carries (if a release comes between the two PRs, this is a breaking change of it).
- `nsh-platform-tizen` (not published): feature `entry` (default) for `nsh_tizen_main`; `main(argc, argv)`,
  `run_rust(argc, argv, root)`, `set_ui_sets` for the libraries of apps with Rust crates.

## How to Test

- `cargo test -p nsh-cli --test cli tizen::` (through the binary with a fake runtime, `tz` and `sdb`: the targets
  parse, values refused, no profile, a TPK's layout, the multi-architecture upload, `nsh run`'s `sdb` sequence).
- `cargo test -p nsh-cli --bin nsh tizen::tests` (manifest, staging, naming, `tz` and `sdb` command lines, devices,
  the install record, the test driver, the open and close sequences, the upload's architectures),
  `tizen::symbols::tests` (the symbol check's parsing and rules), `rust_app::tests::a_tizen` (the generated workspace
  and its one export), `component::tests::tizen`, `test_host::tests::tizen` and
  `test_host::tests::a_device_is_for_the_tizen_target_only`.
- `cargo test -p xtask --bin xtask tizen::tests`, `cargo test -p nsh-dev --lib protocol::tests::tizen`.
- On a Samsung TV or the Tizen emulator: [OWNER: `nsh run --target tizen`, `nsh package --target tizen --run`,
  `nsh test examples/counter --target tizen`].

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): CLI, `nsh.json` and tooling only.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (on a TV and the emulator instead).
