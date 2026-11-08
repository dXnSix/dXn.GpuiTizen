//! The one window: a full-screen, opaque Ecore_Wl2 window at the UI plane's size, drawn by GPUI's
//! wgpu renderer on its Wayland handles. Pause frees the surface and the atlas textures (GPUI
//! caches the atlas, so the renderer stays and keeps it); resume configures a new surface on the
//! same handles.

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::rc::Rc;
use std::sync::Arc;

use dxn_gpui_tizen_sys as sys;
use gpui::{
    AnyWindowHandle, Bounds, Capslock, DevicePixels, DispatchEventResult, GpuSpecs, Modifiers,
    Pixels, PlatformAtlas, PlatformDisplay, PlatformInput, PlatformInputHandler, PlatformWindow,
    Point, PromptButton, PromptLevel, RequestFrameOptions, Scene, Size, TextInputConfiguration,
    TextInputStateChange, WindowAppearance, WindowBackgroundAppearance, WindowBounds,
    WindowControlArea, WindowInsets, WindowParams, WindowVisibility, px, size,
};
use gpui_wgpu::{WgpuRenderer, WgpuSurfaceConfig};
use raw_window_handle as rwh;

use super::display::TizenDisplay;
use super::platform::Shared;
use crate::display;

/// The window's Wayland handles, for wgpu.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RawWindow {
    surface: NonNull<c_void>,
    display: NonNull<c_void>,
}

// SAFETY: the pointers are the window's wl_surface and the app's wl_display, which outlive the
// renderer; wgpu requires Send + Sync for surface creation and uses them on the main thread.
unsafe impl Send for RawWindow {}
// SAFETY: as above.
unsafe impl Sync for RawWindow {}

impl rwh::HasWindowHandle for RawWindow {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        let handle = rwh::WaylandWindowHandle::new(self.surface);
        // SAFETY: the wl_surface lives as long as the window, which outlives every borrow.
        Ok(unsafe { rwh::WindowHandle::borrow_raw(handle.into()) })
    }
}

impl rwh::HasDisplayHandle for RawWindow {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        let handle = rwh::WaylandDisplayHandle::new(self.display);
        // SAFETY: the wl_display lives as long as the app.
        Ok(unsafe { rwh::DisplayHandle::borrow_raw(handle.into()) })
    }
}

/// What came of asking GPUI for a frame.
pub(crate) enum FrameOutcome {
    Presented,
    /// GPUI found nothing to draw.
    Skipped,
    /// GPUI drew, but the renderer could not present.
    Failed,
}

#[derive(Default)]
struct Callbacks {
    request_frame: Option<Box<dyn FnMut(RequestFrameOptions)>>,
    input: Option<Box<dyn FnMut(PlatformInput) -> DispatchEventResult>>,
    active: Option<Box<dyn FnMut(bool)>>,
    close: Option<Box<dyn FnOnce()>>,
}

pub(crate) struct WindowInner {
    pub(crate) handle: AnyWindowHandle,
    ecore: *mut sys::Ecore_Wl2_Window,
    raw: RawWindow,
    pixels: (u32, u32),
    scale: f32,
    renderer: RefCell<WgpuRenderer>,
    callbacks: RefCell<Callbacks>,
    input_handler: RefCell<Option<PlatformInputHandler>>,
    /// Set by `draw`: whether GPUI drew this frame, and whether it was presented.
    drawn: Cell<bool>,
    presented: Cell<bool>,
    frame_handle: Cell<*mut sys::Ecore_Wl2_Frame_Cb_Handle>,
    shared: std::rc::Weak<Shared>,
}

impl WindowInner {
    fn config(&self) -> WgpuSurfaceConfig {
        WgpuSurfaceConfig {
            size: Size {
                width: DevicePixels(self.pixels.0 as i32),
                height: DevicePixels(self.pixels.1 as i32),
            },
            transparent: false,
            preferred_present_mode: None,
        }
    }

    /// Asks GPUI for a frame.
    pub(crate) fn request_frame(&self, force_render: bool) -> FrameOutcome {
        self.drawn.set(false);
        self.presented.set(false);
        let callback = self.callbacks.borrow_mut().request_frame.take();
        if let Some(mut callback) = callback {
            callback(RequestFrameOptions {
                require_presentation: force_render,
                force_render,
                signal_at: None,
                signal_source: gpui::FrameRequestSource::NativeCallback,
            });
            let mut callbacks = self.callbacks.borrow_mut();
            if callbacks.request_frame.is_none() {
                callbacks.request_frame = Some(callback);
            }
        }
        match (self.drawn.get(), self.presented.get()) {
            (_, true) => FrameOutcome::Presented,
            (true, false) => FrameOutcome::Failed,
            (false, false) => FrameOutcome::Skipped,
        }
    }

    /// Whether the renderer asks for the next frame in full (read on every outcome).
    pub(crate) fn needs_redraw(&self) -> bool {
        self.renderer.borrow_mut().needs_redraw()
    }

    /// Dispatches input to GPUI; returns whether it was handled.
    pub(crate) fn dispatch_input(&self, input: PlatformInput) -> bool {
        let callback = self.callbacks.borrow_mut().input.take();
        let Some(mut callback) = callback else {
            return false;
        };
        let result = callback(input);
        let mut callbacks = self.callbacks.borrow_mut();
        if callbacks.input.is_none() {
            callbacks.input = Some(callback);
        }
        !result.propagate || result.default_prevented
    }

    pub(crate) fn insert_text(&self, text: &str) {
        let handler = self.input_handler.borrow_mut().take();
        if let Some(mut handler) = handler {
            handler.replace_text_in_range(None, text);
            let mut slot = self.input_handler.borrow_mut();
            if slot.is_none() {
                *slot = Some(handler);
            }
        }
    }

    pub(crate) fn release_gpu(&self) {
        let mut renderer = self.renderer.borrow_mut();
        renderer.unconfigure_surface();
        renderer.sprite_atlas().clear();
    }

    pub(crate) fn rebuild_gpu(&self, gpu: &gpui_wgpu::GpuContext) {
        let instance = gpu
            .borrow()
            .as_ref()
            .map(|context| context.instance.clone());
        let Some(instance) = instance else {
            log::error!("no wgpu context to rebuild the surface with");
            return;
        };
        if let Err(err) =
            self.renderer
                .borrow_mut()
                .replace_surface(&self.raw, self.config(), &instance)
        {
            log::error!("rebuilding the surface failed: {err:#}");
        }
    }

    fn logical_size(&self) -> Size<Pixels> {
        let (w, h) = display::logical_size(self.pixels, self.scale);
        size(px(w), px(h))
    }
}

unsafe extern "C" fn on_frame(
    _window: *mut sys::Ecore_Wl2_Window,
    timestamp: u32,
    _data: *mut c_void,
) {
    if let Some(shared) = super::app::shared() {
        shared.frame_callback(timestamp);
    }
}

pub(crate) struct TizenWindow(Rc<WindowInner>);

impl TizenWindow {
    pub(crate) fn open(
        shared: &Rc<Shared>,
        handle: AnyWindowHandle,
        _options: WindowParams,
    ) -> anyhow::Result<Self> {
        let display = shared.display.get();
        anyhow::ensure!(!display.is_null(), "no Ecore_Wl2 display");
        let (w, h) = shared.display_size.get();
        // SAFETY: a connected display; one top-level, opaque window at the UI plane's size.
        let ecore = unsafe {
            let window =
                sys::ecore_wl2_window_new(display, ptr::null_mut(), 0, 0, w as i32, h as i32);
            if !window.is_null() {
                sys::ecore_wl2_window_alpha_set(window, sys::EINA_FALSE);
                sys::ecore_wl2_window_show(window);
                sys::ecore_wl2_window_activate(window);
            }
            window
        };
        anyhow::ensure!(!ecore.is_null(), "ecore_wl2_window_new failed");
        // SAFETY: the window and display were just checked.
        let (surface, wl_display) = unsafe {
            (
                sys::ecore_wl2_window_native_surface_get(ecore),
                sys::ecore_wl2_display_native_get(display),
            )
        };
        let raw = RawWindow {
            surface: NonNull::new(surface)
                .ok_or_else(|| anyhow::anyhow!("the window has no wl_surface"))?,
            display: NonNull::new(wl_display).ok_or_else(|| anyhow::anyhow!("no wl_display"))?,
        };
        let config = WgpuSurfaceConfig {
            size: Size {
                width: DevicePixels(w as i32),
                height: DevicePixels(h as i32),
            },
            transparent: false,
            preferred_present_mode: None,
        };
        let renderer = WgpuRenderer::new(shared.gpu.clone(), &raw, config, None)?;
        let inner = Rc::new(WindowInner {
            handle,
            ecore,
            raw,
            pixels: (w, h),
            scale: shared.scale,
            renderer: RefCell::new(renderer),
            callbacks: RefCell::new(Callbacks::default()),
            input_handler: RefCell::new(None),
            drawn: Cell::new(false),
            presented: Cell::new(false),
            frame_handle: Cell::new(ptr::null_mut()),
            shared: Rc::downgrade(shared),
        });
        // SAFETY: the window is alive; the callback reaches the state through `shared()`; the
        // handle is deleted before the window is freed.
        let handle = unsafe {
            sys::ecore_wl2_window_frame_callback_add(ecore, Some(on_frame), ptr::null_mut())
        };
        inner.frame_handle.set(handle);
        *shared.window.borrow_mut() = Some(Rc::clone(&inner));
        log::info!("window {w}x{h} at scale {}", shared.scale);
        Ok(Self(inner))
    }
}

impl rwh::HasWindowHandle for TizenWindow {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        rwh::HasWindowHandle::window_handle(&self.0.raw)
    }
}

impl rwh::HasDisplayHandle for TizenWindow {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        rwh::HasDisplayHandle::display_handle(&self.0.raw)
    }
}

impl Drop for TizenWindow {
    fn drop(&mut self) {
        if let Some(shared) = self.0.shared.upgrade() {
            shared.window.borrow_mut().take();
        }
        if let Some(close) = self.0.callbacks.borrow_mut().close.take() {
            close();
        }
        // The renderer's EGL surface goes before the wl_surface it was made on.
        self.0.renderer.borrow_mut().destroy();
        let handle = self.0.frame_handle.replace(ptr::null_mut());
        // SAFETY: the handle (if any) and the window come from `open`; nothing draws any more.
        unsafe {
            if !handle.is_null() {
                sys::ecore_wl2_window_frame_callback_del(handle);
            }
            sys::ecore_wl2_window_free(self.0.ecore);
        }
    }
}

impl PlatformWindow for TizenWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        Bounds {
            origin: Point::default(),
            size: self.0.logical_size(),
        }
    }

    fn is_maximized(&self) -> bool {
        true
    }

    fn window_bounds(&self) -> WindowBounds {
        WindowBounds::Fullscreen(self.bounds())
    }

    fn content_size(&self) -> Size<Pixels> {
        self.0.logical_size()
    }

    fn resize(&mut self, _size: Size<Pixels>) {}

    fn scale_factor(&self) -> f32 {
        self.0.scale
    }

    fn appearance(&self) -> WindowAppearance {
        WindowAppearance::Dark
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        self.0
            .shared
            .upgrade()
            .map(|shared| Rc::new(TizenDisplay::new(&shared)) as Rc<dyn PlatformDisplay>)
    }

    fn on_display_changed(&self, _callback: Box<dyn FnMut()>) {}

    fn mouse_position(&self) -> Point<Pixels> {
        Point::default()
    }

    fn modifiers(&self) -> Modifiers {
        Modifiers::default()
    }

    fn capslock(&self) -> Capslock {
        Capslock::default()
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        *self.0.input_handler.borrow_mut() = Some(input_handler);
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.0.input_handler.borrow_mut().take()
    }

    fn prompt(
        &self,
        _level: PromptLevel,
        _msg: &str,
        _detail: Option<&str>,
        _answers: &[PromptButton],
    ) -> Option<futures::channel::oneshot::Receiver<usize>> {
        None
    }

    fn activate(&self) {}

    fn is_active(&self) -> bool {
        true
    }

    fn is_hovered(&self) -> bool {
        false
    }

    fn set_title(&mut self, _title: &str) {}

    fn background_appearance(&self) -> WindowBackgroundAppearance {
        WindowBackgroundAppearance::Opaque
    }

    fn set_background_appearance(&self, _background_appearance: WindowBackgroundAppearance) {}

    fn minimize(&self) {}

    fn zoom(&self) {}

    fn toggle_fullscreen(&self) {}

    fn is_fullscreen(&self) -> bool {
        true
    }

    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        self.0.callbacks.borrow_mut().request_frame = Some(callback);
    }

    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> DispatchEventResult>) {
        self.0.callbacks.borrow_mut().input = Some(callback);
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.callbacks.borrow_mut().active = Some(callback);
    }

    fn visibility(&self) -> WindowVisibility {
        WindowVisibility::Visible
    }

    fn on_visibility_change(&self, _callback: Box<dyn FnMut(WindowVisibility)>) {}

    fn on_hover_status_change(&self, _callback: Box<dyn FnMut(bool)>) {}

    fn on_resize(&self, _callback: Box<dyn FnMut(Size<Pixels>, f32)>) {}

    fn on_moved(&self, _callback: Box<dyn FnMut()>) {}

    fn on_should_close(&self, _callback: Box<dyn FnMut() -> bool>) {}

    fn on_hit_test_window_control(&self, _callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
    }

    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        self.0.callbacks.borrow_mut().close = Some(callback);
    }

    fn on_appearance_changed(&self, _callback: Box<dyn FnMut()>) {}

    fn draw(&self, scene: &Scene) {
        // The frame callback is requested on the commit that carries this frame's buffer: commit
        // without flushing, then the swap in `draw` flushes it (`Ecore_Wayland2.h`).
        // SAFETY: the window is alive.
        unsafe { sys::ecore_wl2_window_commit(self.0.ecore, sys::EINA_FALSE) };
        let presented = self.0.renderer.borrow_mut().draw(scene);
        self.0.drawn.set(true);
        self.0.presented.set(presented);
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.0.renderer.borrow().sprite_atlas().clone()
    }

    fn is_subpixel_rendering_supported(&self) -> bool {
        false
    }

    fn gpu_specs(&self) -> Option<GpuSpecs> {
        self.0.renderer.borrow().gpu_specs()
    }

    fn update_ime_position(&self, _bounds: Bounds<Pixels>) {}

    fn insets(&self) -> WindowInsets {
        WindowInsets::default()
    }

    fn on_insets_changed(&self, _callback: Box<dyn FnMut(WindowInsets)>) {}

    fn show_soft_keyboard(&self) {}

    fn hide_soft_keyboard(&self) {}

    fn text_input_state_changed(&self, _change: TextInputStateChange) {}

    fn set_text_input_configuration(&mut self, _configuration: TextInputConfiguration) {}

    fn schedule_frame(&self) {
        if let Some(shared) = self.0.shared.upgrade() {
            shared.request_frame();
        }
    }
}
