//! EGL and GL ES queries through run-time loading (`dlopen`): EGL and GL ES are never linked.
//! A pbuffer context on the app's Wayland display answers the GL ES questions; the display is
//! left initialized for wgpu, which uses the same native display.

use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

use libloading::Library;

use crate::report::{EglInfo, GlesInfo, parse_extensions};

type EGLDisplay = *mut c_void;
type EGLConfig = *mut c_void;
type EGLContext = *mut c_void;
type EGLSurface = *mut c_void;
type EGLint = i32;
type EGLBoolean = u32;
type EGLenum = u32;

const EGL_NO_DISPLAY: EGLDisplay = std::ptr::null_mut();
const EGL_VENDOR: EGLint = 0x3053;
const EGL_VERSION: EGLint = 0x3054;
const EGL_EXTENSIONS: EGLint = 0x3055;
const EGL_NONE: EGLint = 0x3038;
const EGL_SURFACE_TYPE: EGLint = 0x3033;
const EGL_PBUFFER_BIT: EGLint = 0x0001;
const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
const EGL_OPENGL_ES2_BIT: EGLint = 0x0004;
const EGL_WIDTH: EGLint = 0x3057;
const EGL_HEIGHT: EGLint = 0x3056;
const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;
const EGL_OPENGL_ES_API: EGLenum = 0x30A0;
const EGL_PLATFORM_WAYLAND_KHR: EGLenum = 0x31D8;

const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;
const GL_SHADING_LANGUAGE_VERSION: u32 = 0x8B8C;
const GL_MAX_TEXTURE_SIZE: u32 = 0x0D33;
const GL_MAX_VERTEX_SHADER_STORAGE_BLOCKS: u32 = 0x90D6;

type GetDisplay = unsafe extern "C" fn(*mut c_void) -> EGLDisplay;
type GetPlatformDisplayExt =
    unsafe extern "C" fn(EGLenum, *mut c_void, *const EGLint) -> EGLDisplay;
type GetProcAddress = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type Initialize = unsafe extern "C" fn(EGLDisplay, *mut EGLint, *mut EGLint) -> EGLBoolean;
type QueryString = unsafe extern "C" fn(EGLDisplay, EGLint) -> *const c_char;
type ChooseConfig = unsafe extern "C" fn(
    EGLDisplay,
    *const EGLint,
    *mut EGLConfig,
    EGLint,
    *mut EGLint,
) -> EGLBoolean;
type BindApi = unsafe extern "C" fn(EGLenum) -> EGLBoolean;
type CreateContext =
    unsafe extern "C" fn(EGLDisplay, EGLConfig, EGLContext, *const EGLint) -> EGLContext;
type CreatePbufferSurface =
    unsafe extern "C" fn(EGLDisplay, EGLConfig, *const EGLint) -> EGLSurface;
type MakeCurrent =
    unsafe extern "C" fn(EGLDisplay, EGLSurface, EGLSurface, EGLContext) -> EGLBoolean;
type DestroySurface = unsafe extern "C" fn(EGLDisplay, EGLSurface) -> EGLBoolean;
type DestroyContext = unsafe extern "C" fn(EGLDisplay, EGLContext) -> EGLBoolean;
type GetError = unsafe extern "C" fn() -> EGLint;
type GlGetString = unsafe extern "C" fn(u32) -> *const u8;
type GlGetIntegerv = unsafe extern "C" fn(u32, *mut i32);
type GlGetError = unsafe extern "C" fn() -> u32;

/// EGL and GL ES stay loaded for the whole process: the display stays initialized for wgpu, and
/// the driver may run threads, so they are never unloaded.
static EGL: OnceLock<Result<Library, String>> = OnceLock::new();
static GLES: OnceLock<Result<Library, String>> = OnceLock::new();

fn loaded(
    cell: &'static OnceLock<Result<Library, String>>,
    names: &[&str],
) -> Result<&'static Library, String> {
    cell.get_or_init(|| open(names))
        .as_ref()
        .map_err(Clone::clone)
}

fn open(names: &[&str]) -> Result<Library, String> {
    let mut errors = Vec::new();
    for name in names {
        // SAFETY: loading the platform's EGL / GL ES runs only their initializers, which have no
        // preconditions.
        match unsafe { Library::new(name) } {
            Ok(library) => return Ok(library),
            Err(err) => errors.push(format!("{name}: {err}")),
        }
    }
    Err(errors.join("; "))
}

fn sym<T: Copy>(library: &Library, name: &[u8]) -> Result<T, String> {
    // SAFETY: each call site names an EGL / GL ES entry point with the matching C signature `T`.
    unsafe { library.get::<T>(name) }
        .map(|s| *s)
        .map_err(|e| format!("{}: {e}", String::from_utf8_lossy(name)))
}

fn string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: EGL and GL ES return NUL-terminated static strings; NULL was handled above.
    let text = unsafe { CStr::from_ptr(ptr) };
    Some(text.to_string_lossy().into_owned())
}

/// Queries EGL on `wl_display` and GL ES through a pbuffer context.
pub fn query(wl_display: *mut c_void) -> (Option<EglInfo>, Option<GlesInfo>, Vec<String>) {
    let mut errors = Vec::new();
    let egl = match loaded(&EGL, &["libEGL.so.1", "libEGL.so"]) {
        Ok(library) => library,
        Err(err) => {
            errors.push(format!("loading EGL: {err}"));
            return (None, None, errors);
        }
    };
    match query_with(egl, wl_display, &mut errors) {
        Ok((egl_info, gles_info)) => (Some(egl_info), gles_info, errors),
        Err(err) => {
            errors.push(err);
            (None, None, errors)
        }
    }
}

fn query_with(
    egl: &Library,
    wl_display: *mut c_void,
    errors: &mut Vec<String>,
) -> Result<(EglInfo, Option<GlesInfo>), String> {
    let query_string: QueryString = sym(egl, b"eglQueryString\0")?;
    let get_error: GetError = sym(egl, b"eglGetError\0")?;
    let get_proc: GetProcAddress = sym(egl, b"eglGetProcAddress\0")?;
    // SAFETY: EGL_NO_DISPLAY with EGL_EXTENSIONS asks for the client extensions (EGL 1.5 or
    // EGL_EXT_client_extensions); NULL means unsupported.
    let client = string(unsafe { query_string(EGL_NO_DISPLAY, EGL_EXTENSIONS) });
    let client_extensions = client.as_deref().map(parse_extensions).unwrap_or_default();

    // EGL 1.4 drivers may lack eglGetPlatformDisplay; try the EXT entry point, then eglGetDisplay
    // on the wl_display (as flutter-tizen and DALi do).
    let mut display = EGL_NO_DISPLAY;
    if client_extensions
        .iter()
        .any(|e| e == "EGL_EXT_platform_wayland" || e == "EGL_KHR_platform_wayland")
    {
        // SAFETY: the name is NUL-terminated; the result is the EXT function or NULL.
        let ptr = unsafe { get_proc(c"eglGetPlatformDisplayEXT".as_ptr()) };
        if !ptr.is_null() {
            // SAFETY: eglGetPlatformDisplayEXT has this signature.
            let f: GetPlatformDisplayExt = unsafe { std::mem::transmute(ptr) };
            // SAFETY: a valid wl_display and an empty attribute list.
            display = unsafe { f(EGL_PLATFORM_WAYLAND_KHR, wl_display, [EGL_NONE].as_ptr()) };
        }
    }
    if display == EGL_NO_DISPLAY {
        let get_display: GetDisplay = sym(egl, b"eglGetDisplay\0")?;
        // SAFETY: eglGetDisplay takes the native display (a wl_display on Wayland).
        display = unsafe { get_display(wl_display) };
    }
    if display == EGL_NO_DISPLAY {
        // SAFETY: no preconditions.
        return Err(format!("no EGL display (error {:#x})", unsafe {
            get_error()
        }));
    }
    let initialize: Initialize = sym(egl, b"eglInitialize\0")?;
    let (mut major, mut minor) = (0, 0);
    // SAFETY: a valid display and out pointers; initializing twice is allowed.
    if unsafe { initialize(display, &mut major, &mut minor) } == 0 {
        // SAFETY: no preconditions.
        return Err(format!("eglInitialize failed (error {:#x})", unsafe {
            get_error()
        }));
    }
    // SAFETY: an initialized display.
    let egl_info = unsafe {
        EglInfo {
            vendor: string(query_string(display, EGL_VENDOR)),
            version: string(query_string(display, EGL_VERSION)),
            client_extensions,
            display_extensions: string(query_string(display, EGL_EXTENSIONS))
                .as_deref()
                .map(parse_extensions)
                .unwrap_or_default(),
            flagged: Vec::new(),
        }
        .flag()
    };
    let gles = match gles_query(egl, display) {
        Ok(gles) => Some(gles),
        Err(err) => {
            errors.push(format!("GL ES query: {err}"));
            None
        }
    };
    Ok((egl_info, gles))
}

fn gles_query(egl: &Library, display: EGLDisplay) -> Result<GlesInfo, String> {
    let choose: ChooseConfig = sym(egl, b"eglChooseConfig\0")?;
    let bind: BindApi = sym(egl, b"eglBindAPI\0")?;
    let create_context: CreateContext = sym(egl, b"eglCreateContext\0")?;
    let create_pbuffer: CreatePbufferSurface = sym(egl, b"eglCreatePbufferSurface\0")?;
    let make_current: MakeCurrent = sym(egl, b"eglMakeCurrent\0")?;
    let destroy_surface: DestroySurface = sym(egl, b"eglDestroySurface\0")?;
    let destroy_context: DestroyContext = sym(egl, b"eglDestroyContext\0")?;
    let attribs = [
        EGL_SURFACE_TYPE,
        EGL_PBUFFER_BIT,
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT,
        EGL_NONE,
    ];
    let mut config: EGLConfig = std::ptr::null_mut();
    let mut count = 0;
    // SAFETY: valid display, attribute list and out pointers for one config.
    if unsafe { choose(display, attribs.as_ptr(), &mut config, 1, &mut count) } == 0 || count == 0 {
        return Err("no pbuffer config".into());
    }
    // SAFETY: no preconditions.
    unsafe { bind(EGL_OPENGL_ES_API) };
    let mut context = std::ptr::null_mut();
    for version in [3, 2] {
        // SAFETY: valid display and config; EGL_NO_CONTEXT share; a terminated attribute list.
        context = unsafe {
            create_context(
                display,
                config,
                std::ptr::null_mut(),
                [EGL_CONTEXT_CLIENT_VERSION, version, EGL_NONE].as_ptr(),
            )
        };
        if !context.is_null() {
            break;
        }
    }
    if context.is_null() {
        return Err("no GL ES context".into());
    }
    // SAFETY: valid display and config; a terminated attribute list.
    let surface = unsafe {
        create_pbuffer(
            display,
            config,
            [EGL_WIDTH, 16, EGL_HEIGHT, 16, EGL_NONE].as_ptr(),
        )
    };
    // SAFETY: the surface and context belong to `display`.
    let current =
        !surface.is_null() && unsafe { make_current(display, surface, surface, context) } != 0;
    let result = if current {
        read_gles()
    } else {
        Err("eglMakeCurrent failed".into())
    };
    // SAFETY: release and destroy what this function created; the display stays initialized.
    unsafe {
        make_current(
            display,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if !surface.is_null() {
            destroy_surface(display, surface);
        }
        destroy_context(display, context);
    }
    result
}

fn read_gles() -> Result<GlesInfo, String> {
    let gl = loaded(&GLES, &["libGLESv2.so.2", "libGLESv2.so"])?;
    let get_string: GlGetString = sym(gl, b"glGetString\0")?;
    let get_integer: GlGetIntegerv = sym(gl, b"glGetIntegerv\0")?;
    let get_error: GlGetError = sym(gl, b"glGetError\0")?;
    // SAFETY: a context is current on this thread.
    let gl_string = |name| string(unsafe { get_string(name) }.cast());
    let integer = |name| {
        let mut value = 0;
        // SAFETY: a context is current; `value` takes one integer.
        unsafe {
            get_error();
            get_integer(name, &mut value);
            (get_error() == 0).then_some(i64::from(value))
        }
    };
    Ok(GlesInfo {
        version: gl_string(GL_VERSION),
        renderer: gl_string(GL_RENDERER),
        vendor: gl_string(GL_VENDOR),
        shading_language_version: gl_string(GL_SHADING_LANGUAGE_VERSION),
        extensions: gl_string(GL_EXTENSIONS)
            .as_deref()
            .map(parse_extensions)
            .unwrap_or_default(),
        max_vertex_shader_storage_blocks: integer(GL_MAX_VERTEX_SHADER_STORAGE_BLOCKS),
        max_texture_size: integer(GL_MAX_TEXTURE_SIZE),
    })
}
