//! The platform-owned EGL path on a real Wayland compositor (`EGL-5`): Weston's headless backend
//! with Mesa's software EGL in the Lima VM, `scripts/lima-run.sh --weston`.
//!
//! `scripts/lima-run.sh --weston cargo test -p dxn_gpui_tizen --test egl_wayland`
#![cfg(target_os = "linux")]

use std::ffi::c_void;
use std::path::PathBuf;

use dxn_gpui_tizen::egl::{
    EglOptions, EglWindow, blob_cache_callbacks, blob_cache_counts, blob_cache_registered,
    no_current_context,
};
use gpui::{
    BorderStyle, Bounds, ContentMask, Corners, DevicePixels, Edges, Hsla, Quad, ScaledPixels,
    Scene, Size, point, size,
};
use wayland_client::protocol::{wl_compositor, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle, delegate_noop};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

const WIDTH: i32 = 96;
const HEIGHT: i32 = 64;
const SIZE: Size<DevicePixels> = Size {
    width: DevicePixels(WIDTH),
    height: DevicePixels(HEIGHT),
};

#[derive(Default)]
struct State {
    compositor: Option<wl_compositor::WlCompositor>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    configured: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, 4, qh, ()));
                }
                "xdg_wm_base" => state.wm_base = Some(registry.bind(name, 1, qh, ())),
                _ => {}
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            state.configured = true;
        }
    }
}

delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore xdg_toplevel::XdgToplevel);

/// A mapped toplevel surface on the compositor of `WAYLAND_DISPLAY`.
struct Surface {
    connection: Connection,
    queue: EventQueue<State>,
    state: State,
    surface: wl_surface::WlSurface,
    _xdg: (xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel),
}

impl Surface {
    fn new() -> Self {
        let connection = Connection::connect_to_env().expect("WAYLAND_DISPLAY (run with --weston)");
        let mut queue = connection.new_event_queue();
        let qh = queue.handle();
        connection.display().get_registry(&qh, ());
        let mut state = State::default();
        queue.roundtrip(&mut state).unwrap();
        let compositor = state.compositor.clone().expect("wl_compositor");
        let wm_base = state.wm_base.clone().expect("xdg_wm_base");
        let surface = compositor.create_surface(&qh, ());
        let xdg_surface = wm_base.get_xdg_surface(&surface, &qh, ());
        let toplevel = xdg_surface.get_toplevel(&qh, ());
        toplevel.set_title("dxn_gpui_tizen egl_wayland".into());
        surface.commit();
        while !state.configured {
            queue.blocking_dispatch(&mut state).unwrap();
        }
        Self {
            connection,
            queue,
            state,
            surface,
            _xdg: (xdg_surface, toplevel),
        }
    }

    fn display_ptr(&self) -> *mut c_void {
        self.connection.backend().display_ptr().cast()
    }

    fn surface_ptr(&self) -> *mut c_void {
        self.surface.id().as_ptr().cast()
    }

    /// Handles what the compositor sent (pings, configures).
    fn dispatch(&mut self) {
        let _ = self.queue.dispatch_pending(&mut self.state);
        let _ = self.connection.flush();
    }
}

fn cache_dir() -> PathBuf {
    std::env::temp_dir().join(format!("dxn-egl-blob-cache-{}", std::process::id()))
}

fn window(surface: &Surface) -> EglWindow {
    let options = EglOptions {
        blob_cache_dir: Some(cache_dir()),
        ..EglOptions::default()
    };
    // SAFETY: the surface and its connection outlive the window (dropped first by the tests).
    unsafe { EglWindow::new(surface.display_ptr(), surface.surface_ptr(), SIZE, &options) }
        .expect("the platform-owned EGL path comes up under Weston")
}

fn quad(x: f32, y: f32, w: f32, h: f32, color: Hsla) -> Quad {
    let bounds = Bounds {
        origin: point(ScaledPixels(x), ScaledPixels(y)),
        size: size(ScaledPixels(w), ScaledPixels(h)),
    };
    Quad {
        order: 0,
        border_style: BorderStyle::Solid,
        bounds,
        content_mask: ContentMask { bounds },
        background: color.into(),
        border_color: color,
        corner_radii: Corners::default(),
        border_widths: Edges::default(),
    }
}

/// A black window with a block 24×16 at (8, 4), near its top-left corner.
fn scene(block: Hsla) -> Scene {
    let mut scene = Scene::default();
    scene.insert_primitive(quad(0., 0., WIDTH as f32, HEIGHT as f32, gpui::black()));
    scene.insert_primitive(quad(8., 4., 24., 16., block));
    scene.finish();
    scene
}

/// The pixel at (x, y), top-left origin, from GL's bottom-up read-back.
fn pixel(pixels: &[u8], x: i32, y: i32) -> [u8; 4] {
    let row = HEIGHT - 1 - y;
    let at = ((row * WIDTH + x) * 4) as usize;
    [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
}

#[test]
fn the_context_is_gl_es_3_and_the_blob_cache_is_registered() {
    let mut surface = Surface::new();
    {
        let mut first = window(&surface);
        assert!(
            first.gl_version().contains("OpenGL ES 3"),
            "{}",
            first.gl_version()
        );
        first.draw(&scene(gpui::red())).unwrap();
        surface.dispatch();
    }
    assert!(
        blob_cache_registered(),
        "Mesa offers EGL_ANDROID_blob_cache: the store is registered on the display"
    );
    // The callbacks EGL was given reach the store: a value set through them comes back.
    let (set, get) = blob_cache_callbacks();
    let key = b"egl_wayland test key";
    let value = b"a compiled program";
    // SAFETY: valid buffers of the given sizes, as a driver passes them.
    unsafe {
        set(
            key.as_ptr().cast(),
            key.len() as isize,
            value.as_ptr().cast(),
            value.len() as isize,
        )
    };
    let mut out = [0u8; 64];
    // SAFETY: as above; `out` is writable for its length.
    let size = unsafe {
        get(
            key.as_ptr().cast(),
            key.len() as isize,
            out.as_mut_ptr().cast(),
            out.len() as isize,
        )
    };
    assert_eq!(&out[..size as usize], value);
    // A second context: when the driver uses the cache, it finds what the first stored.
    let before = blob_cache_counts().unwrap();
    let mut second = window(&surface);
    second.draw(&scene(gpui::red())).unwrap();
    surface.dispatch();
    let after = blob_cache_counts().unwrap();
    if before.sets > 1 {
        assert!(after.hits > before.hits, "{after:?}");
    } else {
        // Mesa 25's llvmpipe on Wayland keeps its own disk cache and never calls the
        // callbacks; TV drivers are checked on the device.
        eprintln!("the driver did not call the blob cache: {after:?}");
    }
}

#[test]
fn a_small_change_swaps_with_damage() {
    let mut surface = Surface::new();
    let mut window = window(&surface);
    let support = window.damage_support();
    let first = window.draw(&scene(gpui::red())).unwrap();
    assert!(first.presented);
    assert_eq!(first.age, 0, "the first frame is drawn whole");
    let mut reports = Vec::new();
    for i in 0..6 {
        surface.dispatch();
        let color = if i % 2 == 0 {
            gpui::blue()
        } else {
            gpui::red()
        };
        reports.push(window.draw(&scene(color)).unwrap());
    }
    for report in &reports {
        assert!(report.presented, "every frame changed: {report:?}");
        assert_eq!(report.damage.len(), 1, "the block only: {report:?}");
        let damage = report.damage[0];
        assert!(
            damage.size.width.0 <= 30 && damage.size.height.0 <= 22,
            "{damage:?}"
        );
    }
    if support.buffer_age {
        let aged: Vec<_> = reports.iter().filter(|r| r.age > 0).collect();
        assert!(!aged.is_empty(), "Mesa reports buffer age: {reports:?}");
        for report in aged {
            assert!(
                report.redraw.iter().all(|r| r.size.width.0 < WIDTH),
                "a partial redraw for an aged buffer: {report:?}"
            );
            assert_eq!(
                report.damaged_swap,
                support.swap_with_damage_name().is_some(),
                "{report:?}"
            );
        }
    } else {
        eprintln!("Mesa reports no EGL_EXT_buffer_age here: full frames only");
    }
    // An unchanged frame is not swapped.
    let same = window.draw(&scene(gpui::red())).unwrap();
    let same = if reports.last().is_some_and(|r| r.damage.is_empty()) {
        same
    } else {
        window.draw(&scene(gpui::red())).unwrap()
    };
    assert!(same.damage.is_empty(), "{same:?}");
}

#[test]
fn read_back_pixels_match_the_frame() {
    let mut surface = Surface::new();
    let mut window = window(&surface);
    let (_, pixels) = window.draw_and_read(&scene(gpui::red())).unwrap();
    surface.dispatch();
    assert_eq!(
        pixel(&pixels, 16, 10),
        [255, 0, 0, 255],
        "the block, upright"
    );
    assert_eq!(pixel(&pixels, 60, 40), [0, 0, 0, 255], "the background");
    assert_eq!(
        pixel(&pixels, 16, HEIGHT - 1 - 10),
        [0, 0, 0, 255],
        "not mirrored to the bottom"
    );
    // A second frame changes the block; whatever the buffer age, the read-back is the new frame.
    for _ in 0..3 {
        let (_, pixels) = window.draw_and_read(&scene(gpui::blue())).unwrap();
        surface.dispatch();
        assert_eq!(pixel(&pixels, 16, 10), [0, 0, 255, 255]);
        assert_eq!(pixel(&pixels, 60, 40), [0, 0, 0, 255]);
        let (_, pixels) = window.draw_and_read(&scene(gpui::red())).unwrap();
        surface.dispatch();
        assert_eq!(pixel(&pixels, 16, 10), [255, 0, 0, 255]);
    }
}

#[test]
fn a_failed_setup_leaves_nothing_behind() {
    let mut surface = Surface::new();
    let options = EglOptions {
        blob_cache_dir: Some(cache_dir()),
        fail_at_end: true,
        ..EglOptions::default()
    };
    // SAFETY: the surface and its connection outlive the attempt.
    let failed =
        unsafe { EglWindow::new(surface.display_ptr(), surface.surface_ptr(), SIZE, &options) };
    assert!(failed.is_err(), "the test hook fails the setup");
    assert!(
        no_current_context().unwrap(),
        "the failed setup's context is not left current"
    );
    surface.dispatch();
    // The same wl_surface takes another EGL window, as the fallback's would.
    let mut window = window(&surface);
    let (report, pixels) = window.draw_and_read(&scene(gpui::red())).unwrap();
    surface.dispatch();
    assert!(report.presented);
    assert_eq!(pixel(&pixels, 16, 10), [255, 0, 0, 255]);
}

#[test]
fn pause_and_resume_draw_the_frame_whole() {
    let mut surface = Surface::new();
    let mut window = window(&surface);
    let (_, before) = window.draw_and_read(&scene(gpui::red())).unwrap();
    surface.dispatch();
    window.draw(&scene(gpui::blue())).unwrap();
    surface.dispatch();

    window.release_surface();
    assert!(
        !no_current_context().unwrap(),
        "the context stays current without the window surface"
    );
    assert!(
        window.draw(&scene(gpui::red())).is_err(),
        "nothing is drawn while paused"
    );
    surface.dispatch();

    window.rebuild_surface().unwrap();
    let (report, after) = window.draw_and_read(&scene(gpui::red())).unwrap();
    surface.dispatch();
    assert!(report.presented);
    assert_eq!(
        report.age, 0,
        "a new surface's buffers have no age: {report:?}"
    );
    assert_eq!(
        report.redraw,
        [Bounds {
            origin: point(DevicePixels(0), DevicePixels(0)),
            size: SIZE
        }],
        "drawn whole after resume"
    );
    assert!(after == before, "the resumed frame matches a fresh frame");
    // Partial frames work again after resume.
    let next = window.draw(&scene(gpui::blue())).unwrap();
    assert!(next.presented && next.damage.len() == 1, "{next:?}");
}
