//! A GPUI counter that runs on Tizen TVs and on the desktop: two focusable buttons and a count.
//! Left and Right move the focus, Enter presses the focused button, Back quits (Escape on the
//! desktop). The keys are the platform's names of design `d14-keys`.

pub mod view;

pub use view::{Counter, bindings, launch, quit_after_frames};

/// The TV entry the C# host calls.
///
/// # Safety
///
/// Called by the host with `argc` NUL-terminated strings in `argv`.
#[cfg(target_vendor = "tizen")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dxn_gpui_tizen_main(
    argc: core::ffi::c_int,
    argv: *mut *mut core::ffi::c_char,
) -> core::ffi::c_int {
    // SAFETY: argv comes from the host as documented above.
    unsafe { dxn_gpui_tizen::run(argc, argv, dxn_gpui_tizen::RunOptions::default(), launch) }
}
