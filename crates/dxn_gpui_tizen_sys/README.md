# dxn_gpui_tizen_sys

Hand-written `extern "C"` bindings to the documented Tizen 8.0 native C APIs that
[`dxn_gpui_tizen`](https://crates.io/crates/dxn_gpui_tizen) uses: the application framework, Ecore, Ecore_Wl2,
Ecore_IMF, system settings and information, dlog, the media player and media tool. Nothing else is declared.

- `symbols.allow` lists every symbol with the public API module that documents it; `DECLARED_SYMBOLS` and the
  `symbols` module keep the two in step, and `cargo xtask check-symbols` in the
  [repository](https://github.com/dXnSix/dXn.GpuiTizen) checks built TV libraries against it.
- Link directives apply only for `target_vendor = "tizen"`, so the crate builds and its tests run on any host.
- EGL and GL ES are never declared here: they are loaded at run time.

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
