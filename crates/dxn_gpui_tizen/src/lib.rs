//! The Tizen platform for GPUI (gpui-fast): one full-screen window on Ecore_Wl2, a dispatcher on Ecore's
//! main loop, TV keys, the on-screen keyboard, the app lifecycle and one display.
//!
//! The feature modules hold every decision and are tested on the host with fakes; the Tizen shell
//! under `src/tizen/` (`cfg(target_vendor = "tizen")`) only calls the Tizen C APIs and GPUI.

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod a11y;
pub mod blob_cache;
pub mod dispatcher;
pub mod display;
#[cfg(target_os = "linux")]
pub mod egl;
pub mod egl_damage;
pub mod fonts;
pub mod frame;
pub mod ime;
pub mod keygrab;
pub mod keys;
pub mod lifecycle;
pub mod renderer_select;
pub mod settings;

pub use display::RunOptions;

#[cfg(target_vendor = "tizen")]
mod tizen;
#[cfg(target_vendor = "tizen")]
pub use tizen::{launch_extras, run};

/// The renderer and application types the Tizen shell builds on. Naming them here keeps gpui-fast
/// and gpui-fast-wgpu fully compiled for every Tizen target (the 32-bit ARM gate).
#[cfg(target_vendor = "tizen")]
pub mod gpui_types {
    pub use gpui::{App, Application, Platform};
    pub use gpui_wgpu::WgpuRenderer;
}
