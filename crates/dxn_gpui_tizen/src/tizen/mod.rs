//! The Tizen shell: `ui_app_main`, Ecore's main loop, one Ecore_Wl2 window with GPUI's wgpu
//! renderer, and key input. It performs what the feature modules decide and holds no decisions of
//! its own (design `d10-logic-shell-split`). Compiled only for `target_vendor = "tizen"`.

mod app;
mod display;
mod main_loop;
mod platform;
mod window;

pub use app::{launch_extras, run};
