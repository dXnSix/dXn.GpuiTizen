//! The diagnostic app: reports a TV's EGL and GL ES details, CPU, memory, scheduling and launch
//! timings, probes a wgpu surface on an Ecore_Wl2 window, and serves the report over TCP to
//! `cargo xtask diag pull`.
//!
//! It is built on `dxn_gpui_tizen_sys` and wgpu directly, not on GPUI, so it runs before the
//! platform exists and isolates EGL problems from GPUI. `report`, `probe` and `transport` are
//! platform-neutral and host-tested; the window, EGL loading and the real wgpu backend are under
//! `cfg(target_vendor = "tizen")`.

pub mod probe;
pub mod report;
pub mod transport;

#[cfg(target_vendor = "tizen")]
mod tizen;
#[cfg(target_vendor = "tizen")]
pub use tizen::dxn_gpui_tizen_main;
