# dxn_gpui_tizen

The Tizen platform for [GPUI](https://github.com/longbridge/gpui-fast) (gpui-fast), so GPUI apps run on Samsung TVs:
one full-screen window on Ecore_Wl2, a dispatcher on Ecore's main loop, TV remote keys, the on-screen keyboard,
the app lifecycle, the display, accessibility over AT-SPI, a platform-owned EGL context with the driver's shader
cache and partial presents, and video through Tizen's media player.

```rust,ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dxn_gpui_tizen_main(argc: core::ffi::c_int, argv: *mut *mut core::ffi::c_char) -> core::ffi::c_int {
    unsafe { dxn_gpui_tizen::run(argc, argv, dxn_gpui_tizen::RunOptions::default(), |cx| { /* open windows */ }) }
}
```

The app is a `cdylib` built for a Tizen target and started by a small C# host; cross builds, the host, packaging and
the store check live in the [repository](https://github.com/dXnSix/dXn.GpuiTizen). The platform-neutral modules
(keys, frames, lifecycle, IME, renderer selection, video state, …) build and test on any host.

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
