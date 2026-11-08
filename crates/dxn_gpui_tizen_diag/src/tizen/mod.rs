//! The Tizen shell of the diagnostic app: the exported entry, `ui_app_main` with its lifecycle
//! callbacks, one full-screen opaque Ecore_Wl2 window, the collectors, and the probe driven one
//! stage per Ecore timer tick. Everything runs on the main thread except the report server.

mod egl;
mod wgpu_backend;

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr::{self, NonNull};
use std::time::Duration;

use dxn_gpui_tizen_sys as sys;

use crate::probe::{DEFAULT_FRAMES, Probe};
use crate::report::{self, CallResult, Report, StepTimer};
use crate::transport::{self, Shared};
use wgpu_backend::WgpuBackend;

const LOG_TAG: &CStr = c"DXN_DIAG";
const DEFAULT_SIZE: (c_int, c_int) = (1920, 1080);

type Timer = StepTimer<Box<dyn Fn() -> Duration>>;
type RealProbe = Probe<WgpuBackend, Box<dyn Fn() -> f64>>;

/// The app's state, owned by `ui_app_main`'s user data for the life of the main loop.
struct App {
    timer: Timer,
    shared: Shared,
    report: Report,
    display: *mut sys::Ecore_Wl2_Display,
    window: *mut sys::Ecore_Wl2_Window,
    size: (c_int, c_int),
    collected: bool,
    probe: Option<RealProbe>,
    server_started: bool,
    /// The Ecore timer driving the collectors and the probe; deleted while the app is paused,
    /// so a hidden surface never blocks the main loop in a Fifo present.
    timer_handle: *mut sys::Ecore_Timer,
    finished: bool,
}

impl App {
    fn log(&self, message: &str) {
        self.shared.log(message);
        if let Ok(text) = CString::new(message) {
            // SAFETY: a constant format consuming one NUL-terminated string argument.
            unsafe {
                sys::dlog_print(
                    sys::DLOG_INFO,
                    LOG_TAG.as_ptr(),
                    c"%s".as_ptr(),
                    text.as_ptr(),
                )
            };
        }
    }

    fn mark(&mut self, step: &str) {
        let ms = self.timer.mark(step);
        self.report.steps = self.timer.steps().to_vec();
        self.log(&format!("step {step} at {ms:.1} ms"));
    }

    fn publish(&mut self) {
        self.shared.set_report(&self.report);
    }

    /// Marks the report complete (nothing more will run) and publishes it.
    fn finish(&mut self) {
        self.report.complete = true;
        self.finished = true;
        self.timer_handle = ptr::null_mut();
        self.publish();
    }
}

fn read(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// How long ago the process started, from `/proc`.
fn process_age() -> Duration {
    // SAFETY: sysconf has no preconditions.
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    let uptime = read("/proc/uptime")
        .as_deref()
        .and_then(report::parse_uptime);
    let start = read("/proc/self/stat")
        .as_deref()
        .and_then(report::parse_stat_starttime);
    match (uptime, start, u64::try_from(ticks)) {
        (Some(uptime), Some(start), Ok(ticks)) => report::process_age(uptime, start, ticks),
        _ => Duration::ZERO,
    }
}

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

/// What `setpriority` and `sched_setattr` return for the main thread (restored afterwards).
fn scheduling() -> report::SchedulingInfo {
    // SAFETY: plain syscalls on this process/thread with valid arguments.
    unsafe {
        let before = libc::getpriority(libc::PRIO_PROCESS, 0);
        let ret = libc::setpriority(libc::PRIO_PROCESS, 0, -10);
        let setpriority = CallResult {
            ret: i64::from(ret),
            errno: if ret == 0 { 0 } else { errno() },
        };
        if ret == 0 {
            libc::setpriority(libc::PRIO_PROCESS, 0, before);
        }
        #[repr(C)]
        struct SchedAttr {
            size: u32,
            policy: u32,
            flags: u64,
            nice: i32,
            priority: u32,
            runtime: u64,
            deadline: u64,
            period: u64,
        }
        let mut attr = SchedAttr {
            size: std::mem::size_of::<SchedAttr>() as u32,
            policy: libc::SCHED_FIFO as u32,
            flags: 0,
            nice: 0,
            priority: 1,
            runtime: 0,
            deadline: 0,
            period: 0,
        };
        let ret = libc::syscall(libc::SYS_sched_setattr, 0, &attr as *const SchedAttr, 0);
        let sched_setattr = CallResult {
            ret: ret as i64,
            errno: if ret == 0 { 0 } else { errno() },
        };
        if ret == 0 {
            attr.policy = libc::SCHED_OTHER as u32;
            attr.priority = 0;
            libc::syscall(libc::SYS_sched_setattr, 0, &attr as *const SchedAttr, 0);
        }
        report::SchedulingInfo {
            setpriority: Some(setpriority),
            sched_setattr: Some(sched_setattr),
        }
    }
}

/// A `system_info` platform string.
fn platform_string(key: &CStr) -> Option<String> {
    let mut value: *mut c_char = ptr::null_mut();
    // SAFETY: a NUL-terminated key and an out pointer; the value is malloc'd and freed here.
    unsafe {
        if sys::system_info_get_platform_string(key.as_ptr(), &mut value) != 0 || value.is_null() {
            return None;
        }
        let text = CStr::from_ptr(value).to_string_lossy().into_owned();
        libc::free(value.cast());
        Some(text)
    }
}

fn collect(app: &mut App) {
    app.report.model = platform_string(c"http://tizen.org/system/model_name");
    app.report.platform_version = platform_string(c"http://tizen.org/feature/platform.version");
    let width = usize::BITS;
    app.report.cpu = read("/proc/cpuinfo")
        .map(|text| report::parse_cpuinfo(&text, width))
        .unwrap_or(report::CpuInfo {
            pointer_width: width,
            ..Default::default()
        });
    collect_memory(app);
    app.report.scheduling = scheduling();
    // SAFETY: the display is the connected Ecore_Wl2 display.
    let wl_display = unsafe { sys::ecore_wl2_display_native_get(app.display) };
    let (egl, gles, errors) = egl::query(wl_display);
    app.report.egl = egl;
    app.report.gles = gles;
    app.report.errors.extend(errors);
    app.mark("egl");
}

fn collect_memory(app: &mut App) {
    if let Some(text) = read("/proc/self/smaps_rollup") {
        report::parse_smaps_rollup(&text, &mut app.report.memory);
    }
    if let Some(text) = read("/proc/meminfo") {
        report::parse_meminfo(&text, &mut app.report.memory);
    }
}

fn start_probe(app: &mut App) {
    // SAFETY: the display and window are alive (created in `create`, freed in `terminate`).
    let (wl_display, wl_surface) = unsafe {
        (
            sys::ecore_wl2_display_native_get(app.display),
            sys::ecore_wl2_window_native_surface_get(app.window),
        )
    };
    match (NonNull::new(wl_display), NonNull::new(wl_surface)) {
        (Some(display), Some(surface)) => {
            let (w, h) = app.size;
            let backend = WgpuBackend::new(display, surface, w as u32, h as u32);
            let start = std::time::Instant::now();
            let clock: Box<dyn Fn() -> f64> =
                Box::new(move || start.elapsed().as_secs_f64() * 1000.0);
            app.probe = Some(Probe::new(backend, DEFAULT_FRAMES, clock));
        }
        _ => {
            app.report
                .errors
                .push("Ecore_Wl2 returned no wl_display or wl_surface".into());
            app.report.probe.ok = false;
        }
    }
}

/// Step names for the probe stages, recorded when each one completes.
fn probe_step_name(stage: &str) -> Option<&'static str> {
    match stage {
        "instance" => Some("wgpu-instance"),
        "surface" => Some("wgpu-surface"),
        "adapter" => Some("wgpu-adapter"),
        "device" => Some("wgpu-device"),
        "configure" => Some("wgpu-configure"),
        _ => None,
    }
}

unsafe extern "C" fn tick(data: *mut c_void) -> sys::Eina_Bool {
    // SAFETY: `data` is the `App` that `ui_app_main` owns for the life of the main loop.
    let app = unsafe { &mut *data.cast::<App>() };
    if !app.collected {
        app.collected = true;
        collect(app);
        start_probe(app);
        app.publish();
        return sys::ECORE_CALLBACK_RENEW;
    }
    let Some(probe) = app.probe.as_mut() else {
        app.finish();
        return sys::ECORE_CALLBACK_CANCEL;
    };
    let before = probe.report();
    let more = probe.step();
    let after = probe.report();
    for outcome in after.stages.iter().skip(before.stages.len()) {
        if let Some(error) = &outcome.error {
            app.log(error);
        } else if let Some(step) = probe_step_name(&outcome.stage) {
            app.mark(step);
        }
    }
    if before.frames_presented == 0 && after.frames_presented == 1 {
        app.mark("first-present");
    }
    if more {
        return sys::ECORE_CALLBACK_RENEW;
    }
    let interval = app.probe.as_ref().and_then(Probe::frame_interval_ms);
    app.report.probe = after;
    app.report.frame_interval_ms = interval;
    collect_memory(app);
    app.log(&format!("probe finished: ok={}", app.report.probe.ok));
    app.finish();
    sys::ECORE_CALLBACK_CANCEL
}

unsafe extern "C" fn create(data: *mut c_void) -> bool {
    // SAFETY: as in `tick`.
    let app = unsafe { &mut *data.cast::<App>() };
    app.mark("create");
    // SAFETY: ui_app_main has initialized Ecore_Wl2 and connected to the compositor; connecting
    // ourselves is the fallback.
    unsafe {
        app.display = sys::ecore_wl2_connected_display_get(ptr::null());
        if app.display.is_null() {
            app.display = sys::ecore_wl2_display_connect(ptr::null());
        }
    }
    if app.display.is_null() {
        app.report.errors.push("no Ecore_Wl2 display".into());
        app.finish();
        return true;
    }
    let (mut w, mut h) = (0, 0);
    // SAFETY: a connected display and out pointers.
    unsafe { sys::ecore_wl2_display_screen_size_get(app.display, &mut w, &mut h) };
    app.size = if w > 0 && h > 0 { (w, h) } else { DEFAULT_SIZE };
    // SAFETY: a connected display; one top-level window at the UI plane's size, opaque, shown.
    unsafe {
        app.window =
            sys::ecore_wl2_window_new(app.display, ptr::null_mut(), 0, 0, app.size.0, app.size.1);
        if !app.window.is_null() {
            sys::ecore_wl2_window_alpha_set(app.window, sys::EINA_FALSE);
            sys::ecore_wl2_window_show(app.window);
            sys::ecore_wl2_window_activate(app.window);
        }
    }
    if app.window.is_null() {
        app.report.errors.push("ecore_wl2_window_new failed".into());
        app.finish();
        return true;
    }
    app.mark("window");
    app.log(&format!("window {}x{}", app.size.0, app.size.1));
    start_ticks(app);
    true
}

/// The collectors and the probe run from the main loop, one stage per tick.
fn start_ticks(app: &mut App) {
    if app.finished || !app.timer_handle.is_null() {
        return;
    }
    // SAFETY: `app` outlives the loop; the timer callback has the Ecore_Task_Cb signature.
    app.timer_handle = unsafe { sys::ecore_timer_add(0.0, Some(tick), (app as *mut App).cast()) };
}

fn stop_ticks(app: &mut App) {
    if !app.timer_handle.is_null() {
        // SAFETY: the handle is a live timer of this loop (cleared whenever it cancels itself).
        unsafe { sys::ecore_timer_del(app.timer_handle) };
        app.timer_handle = ptr::null_mut();
    }
}

unsafe extern "C" fn app_control(control: sys::app_control_h, data: *mut c_void) {
    // SAFETY: as in `tick`.
    let app = unsafe { &mut *data.cast::<App>() };
    if app.server_started {
        return;
    }
    let mut value: *mut c_char = ptr::null_mut();
    // SAFETY: a valid app_control handle, a NUL-terminated key and an out pointer; the value is
    // malloc'd and freed here.
    let extra = unsafe {
        if sys::app_control_get_extra_data(control, c"port".as_ptr(), &mut value)
            == sys::APP_ERROR_NONE
            && !value.is_null()
        {
            let text = CStr::from_ptr(value).to_string_lossy().into_owned();
            libc::free(value.cast());
            Some(text)
        } else {
            None
        }
    };
    let port = transport::port_from_extra(extra.as_deref());
    match transport::start(port, app.shared.clone()) {
        Ok(_) => {
            app.server_started = true;
            app.log(&format!("serving the report on port {port}"));
        }
        Err(err) => app.log(&format!("cannot listen on port {port}: {err}")),
    }
}

unsafe extern "C" fn pause(data: *mut c_void) {
    // SAFETY: as in `tick`.
    let app = unsafe { &mut *data.cast::<App>() };
    app.log("pause: probe suspended");
    stop_ticks(app);
}

unsafe extern "C" fn resume(data: *mut c_void) {
    // SAFETY: as in `tick`.
    let app = unsafe { &mut *data.cast::<App>() };
    app.log("resume");
    if !app.window.is_null() {
        start_ticks(app);
    }
}

unsafe extern "C" fn terminate(data: *mut c_void) {
    // SAFETY: as in `tick`.
    let app = unsafe { &mut *data.cast::<App>() };
    app.log("terminate");
    stop_ticks(app);
    app.probe = None;
    if !app.window.is_null() {
        // SAFETY: the window was created in `create` and nothing uses it after the probe is gone.
        unsafe { sys::ecore_wl2_window_free(app.window) };
        app.window = ptr::null_mut();
    }
}

/// Panics abort (`panic = "abort"`); the hook leaves the reason in dlog and in the log lines the
/// report server holds (it keeps serving until the process is gone).
fn install_panic_hook(shared: Shared) {
    std::panic::set_hook(Box::new(move |info| {
        let message = format!("panic: {info}");
        shared.log(&message);
        if let Ok(text) = CString::new(message) {
            // SAFETY: a constant format consuming one NUL-terminated string argument.
            unsafe {
                sys::dlog_print(
                    sys::DLOG_ERROR,
                    LOG_TAG.as_ptr(),
                    c"%s".as_ptr(),
                    text.as_ptr(),
                )
            };
        }
    }));
}

/// The library's only export, called by the C# host's `Main`.
///
/// # Safety
///
/// `argv` must point to `argc` NUL-terminated strings followed by a null pointer, as the host
/// passes them, and stay valid until the function returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dxn_gpui_tizen_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let clock: Box<dyn Fn() -> Duration> = {
        let start = std::time::Instant::now();
        Box::new(move || start.elapsed())
    };
    let mut app = Box::new(App {
        timer: StepTimer::new(process_age(), clock),
        shared: Shared::new(),
        report: Report::new(),
        display: ptr::null_mut(),
        window: ptr::null_mut(),
        size: DEFAULT_SIZE,
        collected: false,
        probe: None,
        server_started: false,
        timer_handle: ptr::null_mut(),
        finished: false,
    });
    install_panic_hook(app.shared.clone());
    app.mark("entry");
    let mut callbacks = sys::ui_app_lifecycle_callback_s {
        create: Some(create),
        terminate: Some(terminate),
        pause: Some(pause),
        resume: Some(resume),
        app_control: Some(app_control),
    };
    let data: *mut App = &mut *app;
    // SAFETY: argv comes from the host as documented; the callbacks and `app` outlive the loop,
    // which ui_app_main runs until the app exits.
    unsafe { sys::ui_app_main(argc, argv, &mut callbacks, data.cast()) }
}
