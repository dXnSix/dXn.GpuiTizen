//! `TizenPlatform`: GPUI's `Platform` for one full-screen window on a TV. Menus, dialogs,
//! cursors, screen capture and a second window are no-ops or errors; the clipboard is
//! in-process.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::anyhow;
use dxn_gpui_tizen_sys as sys;
use futures::channel::oneshot;
use gpui::{
    Action, AnyWindowHandle, BackgroundExecutor, ClipboardItem, CursorStyle, DummyKeyboardMapper,
    ForegroundExecutor, KeyDownEvent, KeyUpEvent, Keymap, Menu, MenuItem, PathPromptOptions,
    Platform, PlatformDisplay, PlatformInput, PlatformKeyboardLayout, PlatformKeyboardMapper,
    PlatformTextSystem, PlatformWindow, Result, Task, ThermalState, WindowAppearance, WindowParams,
    WindowVisibility,
};

use super::display::TizenDisplay;
use super::main_loop::EcoreLoop;
use super::window::{FrameOutcome, TizenWindow, WindowInner};
use crate::activation::{Report, WindowStatus};
use crate::dispatcher::{MainLoop, TizenDispatcher};
use crate::display::{self, RefreshMeter, RunOptions};
use crate::frame::{FrameEffect, FrameScheduler};
use crate::keygrab::{self, GrabCall, GrabCalls, GrabSet};
use crate::keys::{self, HeldKeys, KeyEvent};
use crate::lifecycle::Lifecycle;

type OpenUrls = Box<dyn FnMut(Vec<String>)>;

/// What the platform, its window and the Tizen callbacks share on the main thread.
pub(crate) struct Shared {
    pub(crate) argc: c_int,
    pub(crate) argv: *mut *mut c_char,
    pub(crate) exit_code: Cell<c_int>,
    pub(crate) scale: f32,
    pub(crate) dispatcher: Arc<TizenDispatcher<EcoreLoop>>,
    pub(crate) main_loop: Arc<EcoreLoop>,
    pub(crate) lifecycle: RefCell<Lifecycle>,
    /// The window's activation and visibility as GPUI was last told (`crate::activation`).
    pub(crate) status: RefCell<WindowStatus>,
    pub(crate) frames: RefCell<FrameScheduler>,
    pub(crate) refresh: RefCell<RefreshMeter>,
    /// Whether the last frame was drawn straight from a frame callback (for the refresh meter).
    pub(crate) chained: Cell<bool>,
    pub(crate) held: RefCell<HeldKeys>,
    /// The renderer the app asked for, the app's cache directory, and the fallback logged once.
    pub(crate) renderer: crate::renderer_select::Renderer,
    pub(crate) cache_dir: Option<std::path::PathBuf>,
    pub(crate) renderer_fallback: RefCell<crate::renderer_select::FallbackLog>,
    pub(crate) gpu: gpui_wgpu::GpuContext,
    pub(crate) display: Cell<*mut sys::Ecore_Wl2_Display>,
    pub(crate) display_size: Cell<(u32, u32)>,
    pub(crate) window: RefCell<Option<Rc<WindowInner>>>,
    pub(crate) on_finish_launching: RefCell<Option<Box<dyn FnOnce()>>>,
    pub(crate) quit: RefCell<Vec<Box<dyn FnMut() -> bool>>>,
    pub(crate) open_urls: RefCell<Option<OpenUrls>>,
    pub(crate) clipboard: RefCell<Option<ClipboardItem>>,
    key_handlers: RefCell<Vec<*mut sys::Ecore_Event_Handler>>,
}

impl Shared {
    /// `create`: connect to the compositor, listen for keys, and let GPUI finish launching (it
    /// opens the window).
    pub(crate) fn start(self: &Rc<Self>) {
        // SAFETY: ui_app_main connected Ecore_Wl2; connecting ourselves is the fallback.
        let display = unsafe {
            let display = sys::ecore_wl2_connected_display_get(ptr::null());
            if display.is_null() {
                sys::ecore_wl2_display_connect(ptr::null())
            } else {
                display
            }
        };
        self.display.set(display);
        let (mut w, mut h) = (0, 0);
        if !display.is_null() {
            // SAFETY: a connected display and out pointers.
            unsafe { sys::ecore_wl2_display_screen_size_get(display, &mut w, &mut h) };
        }
        self.display_size.set(display::size(Some((w, h))));
        // SAFETY: Ecore's key events exist once ui_app_main initialized Ecore_Input; the
        // handlers live as long as the app.
        unsafe {
            for (kind, cb) in [
                (
                    sys::ECORE_EVENT_KEY_DOWN,
                    Some(
                        on_key_down
                            as unsafe extern "C" fn(
                                *mut c_void,
                                c_int,
                                *mut c_void,
                            ) -> sys::Eina_Bool,
                    ),
                ),
                (
                    sys::ECORE_EVENT_KEY_UP,
                    Some(
                        on_key_up
                            as unsafe extern "C" fn(
                                *mut c_void,
                                c_int,
                                *mut c_void,
                            ) -> sys::Eina_Bool,
                    ),
                ),
            ] {
                let handler = sys::ecore_event_handler_add(kind, cb, ptr::null());
                self.key_handlers.borrow_mut().push(handler);
            }
        }
        super::settings::start(&self.main_loop);
        super::video::start(&self.main_loop);
        if let Some(launch) = self.on_finish_launching.borrow_mut().take() {
            launch();
        }
    }

    pub(crate) fn stop_frames(&self) {
        self.frames.borrow_mut().pause();
        self.refresh.borrow_mut().pause();
        self.held.borrow_mut().clear();
    }

    /// The window is shown (opened or resumed): grab the TV keys.
    pub(crate) fn keys_shown(&self) {
        // Without a window nothing can be grabbed: wait for the window to open.
        if self.window.borrow().is_none() {
            return;
        }
        let calls = keygrab::with_grabs(GrabSet::shown);
        perform_grabs(&calls);
    }

    /// The window is hidden (paused): release them.
    pub(crate) fn keys_hidden(&self) {
        let calls = keygrab::with_grabs(GrabSet::hidden);
        perform_grabs(&calls);
    }

    /// The app terminates: release them for good.
    pub(crate) fn keys_terminated(&self) {
        let calls = keygrab::with_grabs(GrabSet::terminated);
        perform_grabs(&calls);
    }

    pub(crate) fn release_gpu(&self) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.release_gpu();
        }
    }

    pub(crate) fn rebuild_gpu(&self) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.rebuild_gpu(&self.gpu);
        }
    }

    pub(crate) fn force_redraw(self: &Rc<Self>) {
        let effect = self.frames.borrow_mut().resume();
        self.perform_frame(effect, false);
    }

    /// The window turned (in)active or hidden/visible (`crate::activation`): the IMF context and
    /// the AT-SPI window focus follow the activation, and GPUI hears of both. `status` already
    /// holds the new value, so the window's `is_active` and `visibility` agree with the report.
    pub(crate) fn report(&self, report: Report) {
        let Some(window) = self.window.borrow().clone() else {
            return;
        };
        match report {
            Report::Active(active) => {
                window.window_focus(active);
                window.active_status_changed(active);
            }
            Report::Visible(visible) => window.visibility_changed(if visible {
                WindowVisibility::Visible
            } else {
                WindowVisibility::Hidden
            }),
        }
    }

    pub(crate) fn quit_callbacks(&self) {
        let mut callbacks = std::mem::take(&mut *self.quit.borrow_mut());
        for callback in &mut callbacks {
            callback();
        }
    }

    pub(crate) fn app_controlled(&self) {
        let extras = self.lifecycle.borrow().extras().clone();
        log::info!("launch request with {} extra(s)", extras.len());
    }

    /// GPUI wants a frame (`schedule_frame`): draw from the main loop, never re-entrantly.
    pub(crate) fn request_frame(self: &Rc<Self>) {
        let effect = self.frames.borrow_mut().request();
        if effect.is_some() {
            self.post_frame(effect);
        }
    }

    /// The window is shown: its first (forced) frame.
    pub(crate) fn window_ready(self: &Rc<Self>) {
        let effect = self.frames.borrow_mut().ready();
        if effect.is_some() {
            self.post_frame(effect);
        }
    }

    /// Draws `effect` from a later main-loop iteration, unless the app paused meanwhile.
    fn post_frame(self: &Rc<Self>, effect: Option<FrameEffect>) {
        self.main_loop.post(Box::new(move || {
            if let Some(shared) = super::app::shared() {
                let effect = shared.frames.borrow_mut().posted(effect);
                shared.perform_frame(effect, false);
            }
        }));
    }

    /// The compositor's frame callback.
    pub(crate) fn frame_callback(self: &Rc<Self>, timestamp_ms: u32) {
        let chained = self.chained.get();
        self.refresh.borrow_mut().frame_callback(
            std::time::Duration::from_millis(u64::from(timestamp_ms)),
            chained,
        );
        let effect = self.frames.borrow_mut().frame_callback();
        if effect.is_none() {
            self.chained.set(false);
        }
        self.perform_frame(effect, true);
    }

    fn perform_frame(self: &Rc<Self>, effect: Option<FrameEffect>, from_callback: bool) {
        let Some(FrameEffect::Draw { force_render, .. }) = effect else {
            return;
        };
        let Some(window) = self.window.borrow().clone() else {
            let next = self.frames.borrow_mut().not_presented(false);
            self.post_frame(next);
            return;
        };
        self.chained.set(from_callback);
        let outcome = window.request_frame(force_render);
        let needs_redraw = window.needs_redraw();
        let mut frames = self.frames.borrow_mut();
        let next = match outcome {
            FrameOutcome::Presented => {
                frames.presented();
                None
            }
            FrameOutcome::Skipped => frames.not_presented(false),
            FrameOutcome::Failed => frames.not_presented(true),
        };
        // The renderer lost or cleared state (a failed frame, a cleared atlas): render in full.
        let next = next.or_else(|| if needs_redraw { frames.force() } else { None });
        let delay = frames.retry_delay();
        drop(frames);
        if !matches!(outcome, FrameOutcome::Presented) {
            self.chained.set(false);
        }
        if delay.is_zero() {
            self.post_frame(next);
        } else if next.is_some() {
            self.main_loop.add_timer(
                delay,
                Box::new(move || {
                    if let Some(shared) = super::app::shared() {
                        let effect = shared.frames.borrow_mut().posted(next);
                        shared.perform_frame(effect, false);
                    }
                }),
            );
        }
    }

    fn key(self: &Rc<Self>, event: &sys::Ecore_Event_Key, down: bool) {
        let text = |p: *const c_char| -> Option<String> {
            // SAFETY: Ecore's key strings are NUL-terminated or NULL, valid during the event.
            (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
        };
        let Some(keyname) = text(event.keyname) else {
            return;
        };
        let string = text(event.string);
        let key_event = KeyEvent {
            keyname: &keyname,
            string: string.as_deref(),
            modifiers: event.modifiers,
        };
        let Some(window) = self.window.borrow().clone() else {
            return;
        };
        // IME first: a key the input method consumes never reaches the key mapping.
        if window.filter_key(event, down) {
            return;
        }
        if down {
            let (keystroke, is_held) = self.held.borrow_mut().key_down(&key_event);
            let fallback = keys::fallback_text(&keystroke).map(str::to_owned);
            let input = PlatformInput::KeyDown(KeyDownEvent {
                keystroke,
                is_held,
                prefer_character_input: false,
            });
            let handled = window.dispatch_input(input);
            if !handled && let Some(text) = fallback {
                window.insert_text(&text);
            }
        } else {
            let keystroke = self.held.borrow_mut().key_up(&key_event);
            window.dispatch_input(PlatformInput::KeyUp(KeyUpEvent { keystroke }));
        }
    }
}

/// Makes the key grab calls `crate::keygrab` decided, on the window (main thread).
pub(crate) fn perform_grabs(calls: &GrabCalls) {
    if calls.is_empty() {
        return;
    }
    let Some(window) = super::app::shared().and_then(|shared| shared.window.borrow().clone())
    else {
        return;
    };
    window.keygrab(GrabCall::Release, &calls.release);
    window.keygrab(GrabCall::Grab, &calls.grab);
}

unsafe extern "C" fn on_key_down(
    _data: *mut c_void,
    _kind: c_int,
    event: *mut c_void,
) -> sys::Eina_Bool {
    if let (Some(shared), false) = (super::app::shared(), event.is_null()) {
        // SAFETY: ECORE_EVENT_KEY_DOWN delivers an `Ecore_Event_Key`, valid during the call.
        shared.key(unsafe { &*event.cast::<sys::Ecore_Event_Key>() }, true);
    }
    sys::ECORE_CALLBACK_PASS_ON
}

unsafe extern "C" fn on_key_up(
    _data: *mut c_void,
    _kind: c_int,
    event: *mut c_void,
) -> sys::Eina_Bool {
    if let (Some(shared), false) = (super::app::shared(), event.is_null()) {
        // SAFETY: ECORE_EVENT_KEY_UP delivers an `Ecore_Event_Key`, valid during the call.
        shared.key(unsafe { &*event.cast::<sys::Ecore_Event_Key>() }, false);
    }
    sys::ECORE_CALLBACK_PASS_ON
}

struct TizenKeyboardLayout;

impl PlatformKeyboardLayout for TizenKeyboardLayout {
    fn id(&self) -> &str {
        "tizen-tv"
    }

    fn name(&self) -> &str {
        "Tizen TV"
    }
}

pub struct TizenPlatform {
    shared: Rc<Shared>,
    background_executor: BackgroundExecutor,
    foreground_executor: ForegroundExecutor,
    text_system: Arc<dyn PlatformTextSystem>,
}

impl TizenPlatform {
    pub(crate) fn new(
        argc: c_int,
        argv: *mut *mut c_char,
        options: RunOptions,
        cache_dir: Option<PathBuf>,
    ) -> Self {
        let scale = display::scale(&options, std::env::var(display::SCALE_ENV).ok().as_deref());
        for rejected in &scale.rejected {
            log::warn!("{rejected}");
        }
        let text_system = super::fonts::text_system(options.font_families.as_deref());
        let grab_plan = keygrab::plan(options.grab_keys.as_deref());
        for (key, refusal) in &grab_plan.refused {
            log::warn!("not grabbing {key:?}: {refusal:?}");
        }
        // The TV keys grabbed while the window is shown live with `crate::keygrab`, where the
        // app may change them while it runs.
        keygrab::with_grabs(|grabs| *grabs = GrabSet::new(grab_plan.keys));
        let main_loop = Arc::new(EcoreLoop::new());
        let dispatcher = Arc::new(TizenDispatcher::new(Arc::clone(&main_loop)));
        let shared = Rc::new(Shared {
            argc,
            argv,
            exit_code: Cell::new(0),
            scale: scale.factor,
            dispatcher: Arc::clone(&dispatcher),
            main_loop,
            lifecycle: RefCell::new(Lifecycle::new()),
            status: RefCell::new(WindowStatus::new()),
            frames: RefCell::new(FrameScheduler::new()),
            refresh: RefCell::new(RefreshMeter::default()),
            chained: Cell::new(false),
            held: RefCell::new(HeldKeys::default()),
            renderer: options.renderer,
            cache_dir,
            renderer_fallback: RefCell::default(),
            gpu: gpui_wgpu::GpuContext::default(),
            display: Cell::new(ptr::null_mut()),
            display_size: Cell::new(display::DEFAULT_SIZE),
            window: RefCell::new(None),
            on_finish_launching: RefCell::new(None),
            quit: RefCell::new(Vec::new()),
            open_urls: RefCell::new(None),
            clipboard: RefCell::new(None),
            key_handlers: RefCell::new(Vec::new()),
        });
        Self {
            shared,
            background_executor: BackgroundExecutor::new(dispatcher.clone()),
            foreground_executor: ForegroundExecutor::new(dispatcher),
            text_system,
        }
    }

    pub(crate) fn shared(&self) -> Rc<Shared> {
        Rc::clone(&self.shared)
    }

    fn display(&self) -> Rc<dyn PlatformDisplay> {
        Rc::new(TizenDisplay::new(&self.shared))
    }
}

impl Platform for TizenPlatform {
    fn platform_info(&self) -> gpui::PlatformInfo {
        gpui::PlatformInfo {
            os: gpui::OperatingSystem::Linux,
            target: gpui::PlatformTarget::Native,
        }
    }

    fn background_executor(&self) -> BackgroundExecutor {
        self.background_executor.clone()
    }

    fn foreground_executor(&self) -> ForegroundExecutor {
        self.foreground_executor.clone()
    }

    fn text_system(&self) -> Arc<dyn PlatformTextSystem> {
        self.text_system.clone()
    }

    fn run(&self, on_finish_launching: Box<dyn 'static + FnOnce()>) {
        *self.shared.on_finish_launching.borrow_mut() = Some(on_finish_launching);
        super::app::enter_main_loop(&self.shared);
    }

    fn quit(&self) {
        // SAFETY: called on the main thread inside ui_app_main; ends the main loop.
        unsafe { sys::ui_app_exit() };
    }

    fn restart(&self, _binary_path: Option<PathBuf>, _arguments: Vec<std::ffi::OsString>) {
        log::warn!("Tizen apps cannot restart themselves");
    }

    fn activate(&self, _ignoring_other_apps: bool) {}

    fn hide(&self) {}

    fn hide_other_apps(&self) {}

    fn unhide_other_apps(&self) {}

    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> {
        vec![self.display()]
    }

    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(self.display())
    }

    fn on_displays_changed(&self, _callback: Box<dyn FnMut()>) {}

    fn active_window(&self) -> Option<AnyWindowHandle> {
        self.shared.window.borrow().as_ref().map(|w| w.handle)
    }

    fn open_window(
        &self,
        handle: AnyWindowHandle,
        options: WindowParams,
    ) -> anyhow::Result<Box<dyn PlatformWindow>> {
        if self.shared.window.borrow().is_some() {
            anyhow::bail!("a Tizen app has one window");
        }
        let window = TizenWindow::open(&self.shared, handle, options)?;
        self.shared.keys_shown();
        self.shared.window_ready();
        Ok(Box::new(window))
    }

    fn window_appearance(&self) -> WindowAppearance {
        WindowAppearance::Dark
    }

    fn open_url(&self, url: &str) {
        log::warn!("opening URLs is not supported on Tizen yet: {url}");
    }

    fn on_open_urls(&self, callback: Box<dyn FnMut(Vec<String>)>) {
        *self.shared.open_urls.borrow_mut() = Some(callback);
    }

    fn register_url_scheme(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn prompt_for_paths(
        &self,
        _options: PathPromptOptions,
    ) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>> {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(anyhow!("Tizen TVs have no file picker")));
        rx
    }

    fn prompt_for_new_path(
        &self,
        _directory: &Path,
        _suggested_name: Option<&str>,
    ) -> oneshot::Receiver<Result<Option<PathBuf>>> {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(anyhow!("Tizen TVs have no save dialog")));
        rx
    }

    fn can_select_mixed_files_and_dirs(&self) -> bool {
        false
    }

    fn reveal_path(&self, _path: &Path) {}

    fn open_with_system(&self, _path: &Path) {}

    fn on_quit(&self, callback: Box<dyn FnMut() -> bool>) {
        self.shared.quit.borrow_mut().push(callback);
    }

    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}

    fn on_system_sleep(&self, _callback: Box<dyn FnMut()>) {}

    fn on_system_wake(&self, _callback: Box<dyn FnMut()>) {}

    fn set_menus(&self, _menus: Vec<Menu>, _keymap: &Keymap) {}

    fn set_dock_menu(&self, _menu: Vec<MenuItem>, _keymap: &Keymap) {}

    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn Action)>) {}

    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}

    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn Action) -> bool>) {}

    fn thermal_state(&self) -> ThermalState {
        ThermalState::Nominal
    }

    fn on_thermal_state_change(&self, _callback: Box<dyn FnMut()>) {}

    fn prevent_idle_sleep(&self, _reason: &str) -> Task<Result<gpui::ActivityGuard>> {
        Task::ready(Err(anyhow!(
            "idle sleep prevention is not available on Tizen"
        )))
    }

    fn app_path(&self) -> Result<PathBuf> {
        let path = std::env::current_exe()?;
        Ok(path)
    }

    fn path_for_auxiliary_executable(&self, name: &str) -> Result<PathBuf> {
        Err(anyhow!("Tizen apps have no auxiliary executables ({name})"))
    }

    fn set_cursor_style(&self, _style: CursorStyle) {}

    fn hide_cursor_until_mouse_moves(&self) {}

    fn is_cursor_visible(&self) -> bool {
        false
    }

    fn should_auto_hide_scrollbars(&self) -> bool {
        true
    }

    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        self.shared.clipboard.borrow().clone()
    }

    fn write_to_clipboard(&self, item: ClipboardItem) {
        *self.shared.clipboard.borrow_mut() = Some(item);
    }

    fn read_from_primary(&self) -> Option<ClipboardItem> {
        None
    }

    fn write_to_primary(&self, _item: ClipboardItem) {}

    fn write_credentials(&self, _url: &str, _username: &str, _password: &[u8]) -> Task<Result<()>> {
        Task::ready(Err(anyhow!("no credential store on Tizen yet")))
    }

    fn read_credentials(&self, _url: &str) -> Task<Result<Option<(String, Vec<u8>)>>> {
        Task::ready(Ok(None))
    }

    fn delete_credentials(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> {
        Box::new(TizenKeyboardLayout)
    }

    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> {
        Rc::new(DummyKeyboardMapper)
    }

    fn on_keyboard_layout_change(&self, _callback: Box<dyn FnMut()>) {}
}
