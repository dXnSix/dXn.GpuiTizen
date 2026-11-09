//! A video player for Tizen TVs and the desktop: the bundled clip (`res/clip.mp4`) plays full
//! screen through `dxn_gpui_tizen::video` on the TV (on the video plane under a hole where the
//! window can turn translucent, else as a texture), with a focusable control row over it. Left
//! and Right move between the controls, Enter uses one, Back quits (Escape on the desktop). The
//! desktop shows a placeholder where the video plays.

pub mod view;

pub use view::{Command, Control, Controls, VideoApp, bindings, launch, quit_after_frames};

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
