//! A long GPUI list for Tizen TVs and the desktop: 1,000 rows in a `uniform_list` under a filter
//! field. Up and Down move the selection and keep it visible, Up from the first row enters the
//! filter field (text input through the TV's on-screen keyboard), Back quits (Escape on the
//! desktop).

pub mod filter;
pub mod view;

pub use view::{LongList, bindings, launch, quit_after_frames};

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
