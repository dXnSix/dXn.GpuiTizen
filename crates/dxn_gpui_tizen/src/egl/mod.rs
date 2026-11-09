//! The platform-owned EGL context (`EGL-1`, design `d24-platform-egl`): the platform creates the
//! EGL display, context and window surface on a Wayland surface, hands the GL ES context to wgpu
//! (`wgpu_hal::gles::Adapter::new_external`) and draws GPUI scenes with gpui_wgpu's
//! `WgpuExternalRenderer` into the default framebuffer, Y-flipped.
//!
//! Owning EGL gives what wgpu's own surface cannot:
//! - the driver's shader cache (`EGL_ANDROID_blob_cache`), registered on the display before any
//!   context exists and backed by [`crate::blob_cache`];
//! - partial presents: the back buffer's age (`EGL_EXT_buffer_age`), `eglSetDamageRegionKHR`
//!   and `eglSwapBuffersWithDamageKHR`, decided by [`crate::egl_damage`].
//!
//! EGL, GL ES and wayland-egl are loaded at run time (`libEGL.so.1`, `libGLESv2.so.2`,
//! `libwayland-egl.so.1`), never linked. The module takes raw `wl_display` and `wl_surface`
//! pointers and no Ecore types, so it is compiled for every Linux and the Lima VM tests it under
//! Weston. Everything happens on the thread that creates the window, with its context current.
//!
//! - A failed `new` undoes what it made (no context current, no surface on the `wl_surface`), so
//!   the wgpu surface can take the window over.
//! - Pause frees the window surface and the target-sized textures; while there is no window
//!   surface the context is current surfaceless (`EGL_KHR_surfaceless_context`) or on a 1×1
//!   pbuffer, so wgpu's GL objects are always deleted with the context current.
//! - A frame that fails with its damage region in force gets a new window surface
//!   ([`crate::egl_damage::FrameState`]).

use std::ffi::{c_int, c_void};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{Context as _, Result, anyhow, bail};
use gpui::{Bounds, DevicePixels, Scene, Size};
use gpui_wgpu::{WgpuExternalRenderer, wgpu};
use khronos_egl as egl;

use crate::blob_cache::{BlobCache, Counts};
use crate::egl_damage::{self, DamageSupport, FrameState, Recovery, Swap};

type Egl = egl::DynamicInstance<egl::EGL1_4>;

const EGL_PLATFORM_WAYLAND_KHR: egl::Enum = 0x31D8;
const EGL_BUFFER_AGE_EXT: egl::Int = 0x313D;
const EGL_OPENGL_ES3_BIT: egl::Int = 0x0040;
const EGL_CONTEXT_MAJOR_VERSION: egl::Int = 0x3098;
const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
/// The format GPUI draws the default framebuffer in.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

type SetDamageRegionFn = unsafe extern "system" fn(
    egl::EGLDisplay,
    egl::EGLSurface,
    *const egl::Int,
    egl::Int,
) -> egl::Boolean;
type SwapWithDamageFn = unsafe extern "system" fn(
    egl::EGLDisplay,
    egl::EGLSurface,
    *const egl::Int,
    egl::Int,
) -> egl::Boolean;
/// `EGLSetBlobFuncANDROID`.
pub type BlobSetFn = unsafe extern "C" fn(*const c_void, isize, *const c_void, isize);
/// `EGLGetBlobFuncANDROID`.
pub type BlobGetFn = unsafe extern "C" fn(*const c_void, isize, *mut c_void, isize) -> isize;
type SetBlobCacheFuncsFn = unsafe extern "system" fn(egl::EGLDisplay, BlobSetFn, BlobGetFn);
type WlEglWindowCreate = unsafe extern "C" fn(*mut c_void, c_int, c_int) -> *mut c_void;
type WlEglWindowDestroy = unsafe extern "C" fn(*mut c_void);
type GlReadPixels = unsafe extern "system" fn(i32, i32, i32, i32, u32, u32, *mut c_void);

/// The process's blob cache: EGL's callbacks carry no user data.
static BLOB_CACHE: OnceLock<BlobCache> = OnceLock::new();
/// Whether `eglSetBlobCacheFuncsANDROID` was called on a display.
static BLOB_CACHE_REGISTERED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

unsafe extern "C" fn blob_set(
    key: *const c_void,
    key_size: isize,
    value: *const c_void,
    size: isize,
) {
    let (Some(cache), Ok(key_size), Ok(size)) = (
        BLOB_CACHE.get(),
        usize::try_from(key_size),
        usize::try_from(size),
    ) else {
        return;
    };
    if key.is_null() || value.is_null() {
        return;
    }
    // SAFETY: EGL passes valid key and value buffers of these sizes for the call.
    let (key, value) = unsafe {
        (
            std::slice::from_raw_parts(key.cast::<u8>(), key_size),
            std::slice::from_raw_parts(value.cast::<u8>(), size),
        )
    };
    cache.set(key, value);
}

unsafe extern "C" fn blob_get(
    key: *const c_void,
    key_size: isize,
    value: *mut c_void,
    size: isize,
) -> isize {
    let (Some(cache), Ok(key_size), Ok(size)) = (
        BLOB_CACHE.get(),
        usize::try_from(key_size),
        usize::try_from(size),
    ) else {
        return 0;
    };
    if key.is_null() {
        return 0;
    }
    // SAFETY: EGL passes a valid key of `key_size` bytes and, when `size` > 0, a writable buffer
    // of `size` bytes, for the call.
    let (key, out) = unsafe {
        (
            std::slice::from_raw_parts(key.cast::<u8>(), key_size),
            if value.is_null() || size == 0 {
                &mut [][..]
            } else {
                std::slice::from_raw_parts_mut(value.cast::<u8>(), size)
            },
        )
    };
    cache.get(key, out) as isize
}

/// Whether the blob cache was registered on a display (the driver offers
/// `EGL_ANDROID_blob_cache`).
pub fn blob_cache_registered() -> bool {
    BLOB_CACHE_REGISTERED.load(std::sync::atomic::Ordering::Relaxed)
}

/// The callbacks handed to `eglSetBlobCacheFuncsANDROID`, for tests of the glue.
#[doc(hidden)]
pub fn blob_cache_callbacks() -> (BlobSetFn, BlobGetFn) {
    (blob_set, blob_get)
}

/// What the blob cache did in this process (for the diagnostic app and tests).
pub fn blob_cache_counts() -> Option<Counts> {
    BLOB_CACHE.get().map(BlobCache::counts)
}

/// How the window is made.
#[derive(Clone, Debug, Default)]
pub struct EglOptions {
    /// An alpha channel (only while a video hole is active).
    pub alpha: bool,
    /// The blob cache's directory; `None` registers no cache.
    pub blob_cache_dir: Option<PathBuf>,
    /// Fails `new` once everything is made, for tests of the undo.
    #[doc(hidden)]
    pub fail_at_end: bool,
}

/// What a frame did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameReport {
    /// Whether the frame was swapped (nothing drawn: the last frame stays).
    pub presented: bool,
    /// The back buffer's age the frame was drawn for (0: unknown, a full frame).
    pub age: u32,
    /// The frame's damage, in GL's bottom-left coordinates.
    pub damage: Vec<Bounds<DevicePixels>>,
    /// What was drawn into the back buffer.
    pub redraw: Vec<Bounds<DevicePixels>>,
    /// Whether the swap carried damage rectangles.
    pub damaged_swap: bool,
}

struct Libraries {
    _gles: libloading::Library,
    wayland_egl: libloading::Library,
}

/// What draws with the context; dropped while it is still current.
struct Gpu {
    renderer: WgpuExternalRenderer,
    view: wgpu::TextureView,
    _target: wgpu::Texture,
    device: wgpu::Device,
}

/// What `new` has made so far, undone on drop unless handed to the window.
struct Setup<'a> {
    egl: &'a Egl,
    display: egl::Display,
    context: Option<egl::Context>,
    surface: Option<egl::Surface>,
    wl_egl_window: *mut c_void,
    destroy_window: Option<WlEglWindowDestroy>,
}

impl Setup<'_> {
    /// The context, the window surface and the `wl_egl_window`, now the window's.
    fn finish(mut self) -> Result<(egl::Context, egl::Surface, *mut c_void)> {
        let (Some(context), Some(surface)) = (self.context.take(), self.surface.take()) else {
            bail!("the EGL setup is incomplete");
        };
        let window = std::mem::replace(&mut self.wl_egl_window, std::ptr::null_mut());
        Ok((context, surface, window))
    }
}

impl Drop for Setup<'_> {
    fn drop(&mut self) {
        if let Some(context) = self.context
            && self.egl.get_current_context() == Some(context)
        {
            let _ = self.egl.make_current(self.display, None, None, None);
        }
        if let Some(surface) = self.surface.take() {
            let _ = self.egl.destroy_surface(self.display, surface);
        }
        if let Some(context) = self.context.take() {
            let _ = self.egl.destroy_context(self.display, context);
        }
        if let (false, Some(destroy)) = (self.wl_egl_window.is_null(), self.destroy_window) {
            // SAFETY: made by `new` for this surface; its EGL surface is gone.
            unsafe { destroy(self.wl_egl_window) };
        }
        // The display stays initialized: the fallback (wgpu's own EGL) shares it.
    }
}

fn has_extension(extensions: &str, name: &str) -> bool {
    extensions.split_whitespace().any(|e| e == name)
}

/// An opaque (or alpha) RGB888 GL ES 3 window config; with `pbuffer`, one that also makes
/// pbuffers, when the display has one. Returns the config and whether it makes pbuffers.
fn choose_config(
    egl: &Egl,
    display: egl::Display,
    alpha: bool,
    pbuffer: bool,
) -> Result<(egl::Config, bool)> {
    let surface_types: &[(egl::Int, bool)] = if pbuffer {
        &[
            (egl::WINDOW_BIT | egl::PBUFFER_BIT, true),
            (egl::WINDOW_BIT, false),
        ]
    } else {
        &[(egl::WINDOW_BIT, false)]
    };
    for &(surface_type, pbuffer) in surface_types {
        let attributes = [
            egl::SURFACE_TYPE,
            surface_type,
            egl::RENDERABLE_TYPE,
            EGL_OPENGL_ES3_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            if alpha { 8 } else { 0 },
            egl::NONE,
        ];
        if let Some(config) = egl
            .choose_first_config(display, &attributes)
            .map_err(|e| anyhow!("eglChooseConfig: {e}"))?
        {
            return Ok((config, pbuffer));
        }
    }
    bail!("no GL ES 3 window config")
}

/// Whether this thread has no current EGL context (for tests of the undo).
#[doc(hidden)]
pub fn no_current_context() -> Result<bool> {
    // SAFETY: loads libEGL.so.1, whose initializers have no preconditions.
    let egl = unsafe { Egl::load_required_from_filename("libEGL.so.1") }
        .map_err(|e| anyhow!("loading libEGL.so.1: {e}"))?;
    Ok(egl.get_current_context().is_none())
}

/// A Wayland surface drawn by GPUI through a platform-owned EGL context.
pub struct EglWindow {
    gpu: Option<Gpu>,
    atlas: Arc<gpui_wgpu::WgpuAtlas>,
    size: Size<DevicePixels>,
    frame: FrameState,
    support: DamageSupport,
    set_damage_region: Option<SetDamageRegionFn>,
    swap_with_damage: Option<SwapWithDamageFn>,
    read_pixels: Option<GlReadPixels>,
    gl_version: String,
    display: egl::Display,
    context: egl::Context,
    config: egl::Config,
    /// `None` while released (the app is paused).
    surface: Option<egl::Surface>,
    /// `EGL_KHR_surfaceless_context`: the context can be current without a surface.
    surfaceless: bool,
    /// Whether `config` makes pbuffers (only looked for without surfaceless contexts).
    pbuffer_config: bool,
    /// The 1×1 pbuffer the context is current on while there is no window surface, without
    /// surfaceless contexts.
    parked: Option<egl::Surface>,
    /// Shut down: nothing is left to draw with.
    closed: bool,
    wl_surface: *mut c_void,
    wl_egl_window: *mut c_void,
    egl: Arc<Egl>,
    libraries: Libraries,
}

fn load(name: &str) -> Result<libloading::Library> {
    // SAFETY: a system library whose initializers have no preconditions.
    unsafe { libloading::Library::new(name) }.with_context(|| format!("loading {name}"))
}

impl EglWindow {
    /// Creates the EGL context and window surface on `wl_surface` of `wl_display`, `size`
    /// pixels, and a GPUI renderer drawing into it. The context stays current on this thread.
    ///
    /// # Safety
    ///
    /// `wl_display` and `wl_surface` must be live Wayland objects that outlive the window.
    pub unsafe fn new(
        wl_display: *mut c_void,
        wl_surface: *mut c_void,
        size: Size<DevicePixels>,
        options: &EglOptions,
    ) -> Result<Self> {
        anyhow::ensure!(
            !wl_display.is_null() && !wl_surface.is_null(),
            "no Wayland display or surface"
        );
        anyhow::ensure!(size.width.0 > 0 && size.height.0 > 0, "an empty window");
        // SAFETY: loads libEGL.so.1, whose initializers have no preconditions.
        let egl = Arc::new(
            unsafe { Egl::load_required_from_filename("libEGL.so.1") }
                .map_err(|e| anyhow!("loading libEGL.so.1: {e}"))?,
        );
        let gles = load("libGLESv2.so.2")?;
        let wayland_egl = load("libwayland-egl.so.1")?;
        let client_extensions = egl
            .query_string(None, egl::EXTENSIONS)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();

        // The display: EGL 1.5's platform display when offered, else eglGetDisplay on the
        // wl_display, as flutter-tizen and DALi call it.
        let display = match egl.upcast::<egl::EGL1_5>() {
            Some(egl15)
                if client_extensions
                    .split_whitespace()
                    .any(|e| e == "EGL_EXT_platform_wayland") =>
            {
                // SAFETY: a live wl_display (the caller's contract) for the Wayland platform.
                unsafe {
                    egl15.get_platform_display(
                        EGL_PLATFORM_WAYLAND_KHR,
                        wl_display,
                        &[egl::ATTRIB_NONE],
                    )
                }
                .map_err(|e| anyhow!("eglGetPlatformDisplay: {e}"))?
            }
            // SAFETY: as above; eglGetDisplay takes the native display.
            _ => unsafe { egl.get_display(wl_display) }
                .ok_or_else(|| anyhow!("eglGetDisplay failed"))?,
        };
        egl.initialize(display)
            .map_err(|e| anyhow!("eglInitialize: {e}"))?;
        // SAFETY: the library's documented entry points.
        let (create, destroy) = unsafe {
            (
                *wayland_egl.get::<WlEglWindowCreate>(b"wl_egl_window_create\0")?,
                *wayland_egl.get::<WlEglWindowDestroy>(b"wl_egl_window_destroy\0")?,
            )
        };
        // Everything made from here on is undone if a later step fails. Locals declared after
        // `setup` (wgpu's objects) are dropped before it, while the context is still current.
        let mut setup = Setup {
            egl: &egl,
            display,
            context: None,
            surface: None,
            wl_egl_window: std::ptr::null_mut(),
            destroy_window: Some(destroy),
        };
        let extensions = egl
            .query_string(Some(display), egl::EXTENSIONS)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        egl.bind_api(egl::OPENGL_ES_API)
            .map_err(|e| anyhow!("eglBindAPI: {e}"))?;

        // The driver's shader cache, before any context exists.
        if let Some(dir) = &options.blob_cache_dir
            && extensions
                .split_whitespace()
                .any(|e| e == "EGL_ANDROID_blob_cache")
        {
            match BlobCache::open(dir, crate::blob_cache::DEFAULT_MAX_BYTES) {
                Ok(cache) => {
                    let _ = BLOB_CACHE.set(cache);
                    if let Some(set_funcs) = egl.get_proc_address("eglSetBlobCacheFuncsANDROID") {
                        // SAFETY: the extension's documented entry point.
                        let set_funcs: SetBlobCacheFuncsFn =
                            unsafe { std::mem::transmute(set_funcs) };
                        // SAFETY: a live display; the callbacks are static.
                        unsafe { set_funcs(display.as_ptr(), blob_set, blob_get) };
                        BLOB_CACHE_REGISTERED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                Err(err) => log::warn!("no EGL blob cache in {}: {err}", dir.display()),
            }
        }

        let surfaceless = has_extension(&extensions, "EGL_KHR_surfaceless_context");
        let (config, pbuffer_config) = choose_config(&egl, display, options.alpha, !surfaceless)?;
        let context = egl
            .create_context(
                display,
                config,
                None,
                &[EGL_CONTEXT_MAJOR_VERSION, 3, egl::NONE],
            )
            .map_err(|e| anyhow!("eglCreateContext (GL ES 3): {e}"))?;
        setup.context = Some(context);
        // SAFETY: a live wl_surface (the caller's contract) and a positive size.
        let wl_egl_window = unsafe { create(wl_surface, size.width.0, size.height.0) };
        anyhow::ensure!(!wl_egl_window.is_null(), "wl_egl_window_create failed");
        setup.wl_egl_window = wl_egl_window;
        // SAFETY: the window was just made for this display's surface.
        let surface = unsafe { egl.create_window_surface(display, config, wl_egl_window, None) }
            .map_err(|e| anyhow!("eglCreateWindowSurface: {e}"))?;
        setup.surface = Some(surface);
        egl.make_current(display, Some(surface), Some(surface), Some(context))
            .map_err(|e| anyhow!("eglMakeCurrent: {e}"))?;
        // Frames are paced by the platform's frame callbacks: a swap never waits.
        let _ = egl.swap_interval(display, 0);

        let support = DamageSupport::from_extensions(&extensions);
        // SAFETY: the extensions' documented entry points, used only when offered.
        let set_damage_region = support
            .partial_update
            .then(|| egl.get_proc_address("eglSetDamageRegionKHR"))
            .flatten()
            .map(|f| unsafe { std::mem::transmute::<extern "system" fn(), SetDamageRegionFn>(f) });
        let swap_with_damage = support
            .swap_with_damage_name()
            .and_then(|name| egl.get_proc_address(name))
            // SAFETY: as above.
            .map(|f| unsafe { std::mem::transmute::<extern "system" fn(), SwapWithDamageFn>(f) });

        // wgpu on the current context. GL ES core functions come from libGLESv2 (an EGL 1.4
        // eglGetProcAddress may not return them), extensions from eglGetProcAddress.
        let proc_address = |name: &str| -> *const c_void {
            let symbol = format!("{name}\0");
            // SAFETY: looking a symbol up; the pointer is only called through glow's signatures.
            if let Ok(found) = unsafe { gles.get::<*const c_void>(symbol.as_bytes()) } {
                return *found;
            }
            egl.get_proc_address(name)
                .map_or(std::ptr::null(), |f| f as *const c_void)
        };
        // SAFETY: the context is current on this thread and stays so while wgpu uses it.
        let exposed = unsafe {
            wgpu::hal::gles::Adapter::new_external(proc_address, wgpu::GlBackendOptions::default())
        }
        .ok_or_else(|| anyhow!("wgpu rejected the GL ES context"))?;
        let read_pixels = {
            let found = proc_address("glReadPixels");
            // SAFETY: GL ES's documented entry point.
            (!found.is_null())
                .then(|| unsafe { std::mem::transmute::<*const c_void, GlReadPixels>(found) })
        };
        // An instance with no backend of its own: the adapter comes from the context.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::empty(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        // SAFETY: the adapter was exposed from this thread's current context.
        let adapter = unsafe { instance.create_adapter_from_hal(exposed) };
        let gl_version = adapter.get_info().driver_info;
        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("dxn_gpui_tizen_egl"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                ..Default::default()
            }))
            .context("creating the wgpu device on the EGL context")?;
        let target_desc = wgpu::TextureDescriptor {
            label: Some("default_framebuffer"),
            size: wgpu::Extent3d {
                width: size.width.0 as u32,
                height: size.height.0 as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        // SAFETY: the default framebuffer of the current context, as wgpu's GL backend wraps it.
        let target = unsafe {
            device.create_texture_from_hal::<wgpu::hal::api::Gles>(
                wgpu::hal::gles::Texture::default_framebuffer(FORMAT),
                &target_desc,
                wgpu::wgt::TextureUses::UNINITIALIZED,
            )
        };
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let renderer = WgpuExternalRenderer::new(instance, adapter, device.clone(), queue, FORMAT)?;
        let atlas = Arc::clone(renderer.sprite_atlas());
        if options.fail_at_end {
            bail!("failing on request, once everything is made");
        }
        let (context, surface, wl_egl_window) = setup.finish()?;
        log::info!("platform-owned EGL: {gl_version}; damage {support:?}");
        Ok(Self {
            gpu: Some(Gpu {
                renderer,
                view,
                _target: target,
                device,
            }),
            atlas,
            size,
            frame: FrameState::default(),
            support,
            set_damage_region,
            swap_with_damage,
            read_pixels,
            gl_version,
            display,
            context,
            config,
            surface: Some(surface),
            surfaceless,
            pbuffer_config,
            parked: None,
            closed: false,
            wl_surface,
            wl_egl_window,
            egl,
            libraries: Libraries {
                _gles: gles,
                wayland_egl,
            },
        })
    }

    /// The `GL_VERSION` of the context.
    pub fn gl_version(&self) -> &str {
        &self.gl_version
    }

    /// What the display offers for partial presents.
    pub fn damage_support(&self) -> DamageSupport {
        self.support
    }

    /// The renderer's sprite atlas (GPUI's `PlatformWindow::sprite_atlas`).
    pub fn sprite_atlas(&self) -> Arc<gpui_wgpu::WgpuAtlas> {
        Arc::clone(&self.atlas)
    }

    /// Frees the window surface and the target-sized textures (the app is paused): the atlas's
    /// textures, the texture the flipped frame is drawn through and the path intermediates. The
    /// context, wgpu and the renderer stay; the next frame is drawn whole.
    pub fn release_surface(&mut self) {
        if self.surface.is_none() {
            return;
        }
        if self.make_current().is_ok()
            && let Some(gpu) = &mut self.gpu
        {
            self.atlas.clear();
            gpu.renderer.release_targets();
            // wgpu deletes what it dropped once the GPU is done with it.
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        self.drop_window_surface();
    }

    /// Makes the window surface again on the same `wl_surface` (the app resumed); the next frame
    /// is drawn whole.
    pub fn rebuild_surface(&mut self) -> Result<()> {
        anyhow::ensure!(!self.closed, "the window is shut down");
        if self.surface.is_some() {
            return Ok(());
        }
        // SAFETY: the library's documented entry point.
        let create = unsafe {
            *self
                .libraries
                .wayland_egl
                .get::<WlEglWindowCreate>(b"wl_egl_window_create\0")?
        };
        // SAFETY: the wl_surface outlives the window (the contract of `new`).
        let window = unsafe { create(self.wl_surface, self.size.width.0, self.size.height.0) };
        anyhow::ensure!(!window.is_null(), "wl_egl_window_create failed");
        self.wl_egl_window = window;
        // SAFETY: the window was just made for this display's surface.
        let surface = match unsafe {
            self.egl
                .create_window_surface(self.display, self.config, window, None)
        } {
            Ok(surface) => surface,
            Err(e) => {
                self.destroy_wl_egl_window();
                bail!("eglCreateWindowSurface: {e}");
            }
        };
        self.surface = Some(surface);
        self.make_current()?;
        let _ = self.egl.swap_interval(self.display, 0);
        self.invalidate();
        Ok(())
    }

    /// Destroys the window surface and its `wl_egl_window`, the context staying current without
    /// them; any damage region ends with the surface.
    fn drop_window_surface(&mut self) {
        let Some(surface) = self.surface.take() else {
            return;
        };
        self.bind_without_window();
        let _ = self.egl.destroy_surface(self.display, surface);
        self.destroy_wl_egl_window();
        self.frame = FrameState::default();
    }

    /// Makes the context current without the window surface: surfaceless when the display offers
    /// it, else on a 1×1 pbuffer; failing both, nothing stays current. Whether the context is
    /// current.
    fn bind_without_window(&mut self) -> bool {
        if self.surfaceless
            && self
                .egl
                .make_current(self.display, None, None, Some(self.context))
                .is_ok()
        {
            return true;
        }
        if self.parked.is_none() && self.pbuffer_config {
            self.parked = self
                .egl
                .create_pbuffer_surface(
                    self.display,
                    self.config,
                    &[egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE],
                )
                .ok();
        }
        if let Some(parked) = self.parked
            && self
                .egl
                .make_current(self.display, Some(parked), Some(parked), Some(self.context))
                .is_ok()
        {
            return true;
        }
        log::warn!("no surface to keep the EGL context current on");
        let _ = self.egl.make_current(self.display, None, None, None);
        false
    }

    fn destroy_wl_egl_window(&mut self) {
        if self.wl_egl_window.is_null() {
            return;
        }
        // SAFETY: the library's documented entry point.
        if let Ok(destroy) = unsafe {
            self.libraries
                .wayland_egl
                .get::<WlEglWindowDestroy>(b"wl_egl_window_destroy\0")
        } {
            // SAFETY: made by this window, and its EGL surface is gone.
            unsafe { destroy(self.wl_egl_window) };
        }
        self.wl_egl_window = std::ptr::null_mut();
    }

    /// Makes the context current: on the window surface, else surfaceless or on the pbuffer.
    pub fn make_current(&self) -> Result<()> {
        anyhow::ensure!(!self.closed, "the window is shut down");
        let surface = self.surface.or(self.parked);
        anyhow::ensure!(
            surface.is_some() || self.surfaceless,
            "no surface to make the EGL context current on"
        );
        self.egl
            .make_current(self.display, surface, surface, Some(self.context))
            .map_err(|e| anyhow!("eglMakeCurrent: {e}"))
    }

    /// The next frame is drawn whole (the atlas was cleared).
    pub fn invalidate(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            gpu.renderer.invalidate();
        }
    }

    /// Frees everything while the Wayland objects are still alive (the window is closing); the
    /// window draws nothing afterwards. Dropping the window does the same.
    pub fn shutdown(&mut self) {
        if self.closed {
            return;
        }
        // On a TV this is the evidence that the driver uses the shader cache (dlog).
        if let Some(counts) = blob_cache_counts() {
            log::info!("EGL blob cache: {counts:?}");
        }
        let current = if self.surface.is_some() {
            self.make_current().is_ok()
        } else {
            self.bind_without_window()
        };
        if let Some(gpu) = self.gpu.take() {
            if current {
                // wgpu deletes its GL objects while the context is current.
                drop(gpu);
            } else {
                // Without a current context GL calls are undefined: the context's destruction
                // frees the objects instead.
                log::warn!("dropping the EGL window without a current context");
                std::mem::forget(gpu);
            }
        }
        let _ = self.egl.make_current(self.display, None, None, None);
        if let Some(surface) = self.surface.take() {
            let _ = self.egl.destroy_surface(self.display, surface);
        }
        if let Some(parked) = self.parked.take() {
            let _ = self.egl.destroy_surface(self.display, parked);
        }
        let _ = self.egl.destroy_context(self.display, self.context);
        self.destroy_wl_egl_window();
        // EGL keeps a display per native display: a later connection at the same address must
        // not find this one's state.
        let _ = self.egl.terminate(self.display);
        self.closed = true;
    }

    /// Draws `scene` and presents it with damage when the display allows.
    pub fn draw(&mut self, scene: &Scene) -> Result<FrameReport> {
        self.draw_presenting(scene, || {})
    }

    /// Draws `scene` like [`draw`](Self::draw), calling `before_swap` only when a swap follows
    /// (the platform commits its window state with the frame).
    pub fn draw_presenting(
        &mut self,
        scene: &Scene,
        before_swap: impl FnOnce(),
    ) -> Result<FrameReport> {
        self.draw_inspecting(scene, before_swap, |_| {})
            .map(|(report, ())| report)
    }

    /// Draws `scene` like [`draw`](Self::draw), reading the back buffer's RGBA pixels (bottom row
    /// first) before the swap, for the diagnostic app and tests.
    #[doc(hidden)]
    pub fn draw_and_read(&mut self, scene: &Scene) -> Result<(FrameReport, Vec<u8>)> {
        let read = self.read_pixels;
        let size = self.size;
        self.draw_inspecting(
            scene,
            || {},
            |_| {
                let mut pixels = vec![0u8; (size.width.0 * size.height.0 * 4) as usize];
                if let Some(read) = read {
                    // SAFETY: the context is current; the buffer holds width × height RGBA pixels.
                    unsafe {
                        read(
                            0,
                            0,
                            size.width.0,
                            size.height.0,
                            GL_RGBA,
                            GL_UNSIGNED_BYTE,
                            pixels.as_mut_ptr().cast(),
                        )
                    };
                }
                pixels
            },
        )
    }

    fn draw_inspecting<R>(
        &mut self,
        scene: &Scene,
        before_swap: impl FnOnce(),
        inspect: impl FnOnce(&Self) -> R,
    ) -> Result<(FrameReport, R)> {
        let result = self.draw_frame(scene, before_swap, inspect);
        if result.is_err() && self.frame.failed() == Recovery::NewSurface {
            log::warn!("a frame failed with its damage region in force: a new window surface");
            self.drop_window_surface();
            if let Err(err) = self.rebuild_surface() {
                log::error!("making the window surface again failed: {err:#}");
            }
        }
        result
    }

    fn draw_frame<R>(
        &mut self,
        scene: &Scene,
        before_swap: impl FnOnce(),
        inspect: impl FnOnce(&Self) -> R,
    ) -> Result<(FrameReport, R)> {
        let surface = self
            .surface
            .ok_or_else(|| anyhow!("the window surface is released (paused)"))?;
        self.make_current()?;
        let queried = self
            .egl
            .query_surface(self.display, surface, EGL_BUFFER_AGE_EXT)
            .ok();
        let age = self.support.age(queried);
        let support = self.support;
        let set_damage_region = self.set_damage_region;
        let (display, surface_ptr) = (self.display.as_ptr(), surface.as_ptr());
        let mut plan = None;
        let mut region_set = false;
        let size = self.size;
        let gpu = self
            .gpu
            .as_mut()
            .ok_or_else(|| anyhow!("the window is shut down"))?;
        let drawn = gpu
            .renderer
            .draw(scene, &gpu.view, size, true, age, |damage| {
                let frame_plan = egl_damage::plan(&support, &damage.damage, &damage.redraw);
                if let (Some(rects), Some(set)) = (&frame_plan.set_damage_region, set_damage_region)
                {
                    // SAFETY: the current display and surface; `rects` holds `len / 4` rectangles.
                    let ok = unsafe {
                        set(
                            display,
                            surface_ptr,
                            rects.as_ptr(),
                            (rects.len() / 4) as egl::Int,
                        )
                    };
                    if ok == egl::TRUE {
                        region_set = true;
                    } else {
                        // Without a region the whole buffer may change: the frame stays right.
                        log::warn!("eglSetDamageRegionKHR failed");
                    }
                }
                plan = Some(frame_plan);
            });
        if region_set {
            self.frame.region_set();
        }
        let frame = drawn?;
        let inspected = inspect(self);
        let plan = plan.ok_or_else(|| anyhow!("the renderer did not report its damage"))?;
        let (presented, damaged_swap) = match &plan.swap {
            Swap::None => (false, false),
            Swap::WithDamage(rects) if self.swap_with_damage.is_some() => {
                let swap = self.swap_with_damage.expect("checked");
                before_swap();
                // SAFETY: as above.
                let ok = unsafe {
                    swap(
                        display,
                        surface_ptr,
                        rects.as_ptr(),
                        (rects.len() / 4) as egl::Int,
                    )
                };
                if ok != egl::TRUE {
                    bail!("eglSwapBuffersWithDamage: {:?}", self.egl.get_error());
                }
                (true, true)
            }
            _ => {
                before_swap();
                self.egl
                    .swap_buffers(self.display, surface)
                    .map_err(|e| anyhow!("eglSwapBuffers: {e}"))?;
                (true, false)
            }
        };
        if presented {
            self.frame.swapped();
        }
        Ok((
            FrameReport {
                presented,
                age,
                damage: frame.damage,
                redraw: frame.redraw,
                damaged_swap,
            },
            inspected,
        ))
    }
}

impl Drop for EglWindow {
    fn drop(&mut self) {
        self.shutdown();
    }
}
