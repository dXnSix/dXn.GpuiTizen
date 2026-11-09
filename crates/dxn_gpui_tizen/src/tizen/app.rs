//! `run`: GPUI on Tizen. `ui_app_main` runs inside `Platform::run`, its lifecycle callbacks feed
//! the lifecycle machine, and the effects it returns are performed here.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr;
use std::rc::Rc;

use dxn_gpui_tizen_sys as sys;

use super::platform::{Shared, TizenPlatform};
use crate::display::RunOptions;
use crate::lifecycle::Effect;

thread_local! {
    /// The platform state, reachable from Ecore and app-framework callbacks on the main thread.
    static SHARED: RefCell<Option<Rc<Shared>>> = const { RefCell::new(None) };
}

pub(crate) fn shared() -> Option<Rc<Shared>> {
    SHARED.with(|s| s.borrow().clone())
}

pub(crate) fn install(shared: Rc<Shared>) {
    SHARED.with(|s| *s.borrow_mut() = Some(shared));
}

/// The extras of the latest launch request (`app_control`), as a string map.
pub fn launch_extras() -> BTreeMap<String, String> {
    shared()
        .map(|s| s.lifecycle.borrow().extras().clone())
        .unwrap_or_default()
}

/// Logs to dlog (`DXN_GPUI_TIZEN`): retail TVs have no shell, but development TVs and the
/// emulator show it.
struct DlogLogger;

impl log::Log for DlogLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let priority = match record.level() {
            log::Level::Error => sys::DLOG_ERROR,
            log::Level::Warn => sys::DLOG_WARN,
            log::Level::Info => sys::DLOG_INFO,
            _ => sys::DLOG_DEBUG,
        };
        if let Ok(text) = CString::new(format!("{}: {}", record.target(), record.args())) {
            // SAFETY: a constant format consuming one NUL-terminated string.
            unsafe {
                sys::dlog_print(
                    priority,
                    c"DXN_GPUI_TIZEN".as_ptr(),
                    c"%s".as_ptr(),
                    text.as_ptr(),
                )
            };
        }
    }

    fn flush(&self) {}
}

static LOGGER: DlogLogger = DlogLogger;

fn malloced_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: the app framework returns a malloc'd NUL-terminated string, freed here.
    unsafe {
        let text = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        libc::free(ptr.cast());
        Some(text)
    }
}

/// Starts GPUI on Tizen and returns `ui_app_main`'s result when the app exits.
///
/// `on_launch` runs once the app framework created the app (its `create` callback), on the main
/// thread; it opens the app's one window.
///
/// # Safety
///
/// `argv` must point to `argc` NUL-terminated strings (as the C# host passes them) that stay
/// valid until this returns.
pub unsafe fn run(
    argc: c_int,
    argv: *mut *mut c_char,
    options: RunOptions,
    on_launch: impl FnOnce(&mut gpui::App) + 'static,
) -> c_int {
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info));
    // gpui-fast's pipeline cache goes to the app's cache directory. Set before any thread
    // exists: the dispatcher's workers start with the platform below.
    // SAFETY: no function call returns a string we keep; the value is freed by `malloced_string`.
    let cache = malloced_string(unsafe { sys::app_get_cache_path() });
    if let Some(cache) = cache {
        // SAFETY: no GPUI or wgpu thread exists yet, and the .NET host is blocked in the P/Invoke
        // that called us; the runtime's own threads do not read this variable, so nothing reads
        // the environment concurrently with this write in practice.
        unsafe { std::env::set_var("GPUI_PIPELINE_CACHE_DIR", cache) };
    }
    let platform = Rc::new(TizenPlatform::new(argc, argv, options));
    let shared = platform.shared();
    install(Rc::clone(&shared));
    gpui::Application::with_platform(platform).run(on_launch);
    shared.exit_code.get()
}

/// Enters `ui_app_main`; called by `Platform::run`.
pub(crate) fn enter_main_loop(shared: &Rc<Shared>) {
    let mut callbacks = sys::ui_app_lifecycle_callback_s {
        create: Some(on_create),
        terminate: Some(on_terminate),
        pause: Some(on_pause),
        resume: Some(on_resume),
        app_control: Some(on_app_control),
    };
    let (argc, argv) = (shared.argc, shared.argv);
    // SAFETY: argv is the host's (see `run`); the callbacks reach the state through `shared()`.
    let code = unsafe { sys::ui_app_main(argc, argv, &mut callbacks, ptr::null_mut()) };
    shared.exit_code.set(code);
}

fn perform(shared: &Rc<Shared>, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::Start => shared.start(),
            Effect::StopFrames => shared.stop_frames(),
            Effect::ReleaseGpu => shared.release_gpu(),
            Effect::RebuildGpu => shared.rebuild_gpu(),
            Effect::ForceRedraw => shared.force_redraw(),
            Effect::Quit => shared.quit_callbacks(),
            Effect::AppControl => shared.app_controlled(),
        }
    }
}

unsafe extern "C" fn on_create(_data: *mut c_void) -> bool {
    if let Some(shared) = shared() {
        let effects = shared.lifecycle.borrow_mut().create();
        perform(&shared, effects);
    }
    true
}

unsafe extern "C" fn on_resume(_data: *mut c_void) {
    if let Some(shared) = shared() {
        let effects = shared.lifecycle.borrow_mut().resume();
        perform(&shared, effects);
        shared.keys_shown();
    }
}

unsafe extern "C" fn on_pause(_data: *mut c_void) {
    if let Some(shared) = shared() {
        shared.keys_hidden();
        let effects = shared.lifecycle.borrow_mut().pause();
        perform(&shared, effects);
    }
}

unsafe extern "C" fn on_terminate(_data: *mut c_void) {
    if let Some(shared) = shared() {
        shared.keys_terminated();
        super::settings::stop();
        let effects = shared.lifecycle.borrow_mut().terminate();
        perform(&shared, effects);
        shared.dispatcher.shut_down();
    }
}

unsafe extern "C" fn collect_extra(
    control: sys::app_control_h,
    key: *const c_char,
    data: *mut c_void,
) -> bool {
    // SAFETY: `data` is the map `on_app_control` passes for this synchronous iteration.
    let extras = unsafe { &mut *data.cast::<Vec<(String, String)>>() };
    if key.is_null() {
        return true;
    }
    let mut value: *mut c_char = ptr::null_mut();
    // SAFETY: a valid handle and key during the callback; the value is malloc'd.
    let found =
        unsafe { sys::app_control_get_extra_data(control, key, &mut value) } == sys::APP_ERROR_NONE;
    // SAFETY: Ecore passes a NUL-terminated key.
    let key = unsafe { CStr::from_ptr(key) }
        .to_string_lossy()
        .into_owned();
    if let (true, Some(value)) = (found, malloced_string(value)) {
        extras.push((key, value));
    }
    true
}

unsafe extern "C" fn on_app_control(control: sys::app_control_h, _data: *mut c_void) {
    let mut extras: Vec<(String, String)> = Vec::new();
    // SAFETY: a valid handle during the callback; `collect_extra` runs synchronously.
    unsafe {
        sys::app_control_foreach_extra_data(
            control,
            Some(collect_extra),
            (&mut extras as *mut Vec<(String, String)>).cast(),
        )
    };
    if let Some(shared) = shared() {
        let effects = shared.lifecycle.borrow_mut().app_control(extras);
        perform(&shared, effects);
    }
}
