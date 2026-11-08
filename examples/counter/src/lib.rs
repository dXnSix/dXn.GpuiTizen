//! A GPUI counter that runs on Tizen TVs and on the desktop.

/// The TV entry the C# host calls. Until the Tizen platform exists (sprint s6), the example has no
/// UI to run, so it reports failure and exits immediately.
///
/// # Safety
///
/// Called by the host with `argc` NUL-terminated strings in `argv`.
#[cfg(target_vendor = "tizen")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dxn_gpui_tizen_main(
    _argc: core::ffi::c_int,
    _argv: *mut *mut core::ffi::c_char,
) -> core::ffi::c_int {
    1
}
