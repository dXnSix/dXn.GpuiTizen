//! `capi-media-player` behind [`crate::video`] (`VID-2`, `VID-3`): the [`Backend`] over
//! `player_h`, the player's callbacks posted from its threads into the process's [`Inbox`], and
//! [`VideoPlayer`], the GPUI entity apps hold, with its two outputs:
//!
//! - **Overlay**: the player draws on an exported `Ecore_Wl2_Subsurface` below the window
//!   (`PLAYER_DISPLAY_TYPE_OVERLAY_SYNC_UI`; the plain overlay type takes an Evas object, which a
//!   native window does not have). The video element paints a hole where the video is and moves
//!   the exported surface there; the window turns translucent while its scene has holes, and the
//!   surface's changes are committed with the window's frame.
//! - **Texture**: decoded frames (`PLAYER_DISPLAY_TYPE_NONE` and the decoded-frame callback) are
//!   converted to RGBA on the player's thread ([`crate::video_frames`]) and drawn into a
//!   `gpui_wgpu` dynamic texture when the element paints.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CString, c_int, c_void};
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow};
use dxn_gpui_tizen_sys as sys;
use gpui::{
    AsyncApp, Bounds, Context, Corners, Entity, EventEmitter, IntoElement, Pixels, Styled,
    WeakEntity, Window, canvas,
};
use gpui_wgpu::{WgpuContext, WgpuDynamicTexture, wgpu};

use super::main_loop::EcoreLoop;
use crate::dispatcher::MainLoop;
use crate::video::{
    Backend, DeviceRect, Event, Inbox, Notice, Output, OverlayOp, OverlayPlacement, Player,
    PlayerId, State, VideoError,
};
use crate::video_frames::{self, FrameSlot, Matrix, PixelFormat, Plane, RgbaFrame, YuvFrame};

static INBOX: OnceLock<Arc<Inbox>> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// The main thread's handlers of each live player's events.
    static HANDLERS: RefCell<HashMap<PlayerId, Handler>> = RefCell::new(HashMap::new());
}

type Handler = Rc<RefCell<Box<dyn FnMut(Event)>>>;

/// Routes a player's event to its handler (main thread); events of destroyed players are dropped.
/// The handler may destroy its player, which removes it from the map while it runs.
fn route(id: PlayerId, event: Event) {
    let handler = HANDLERS.with(|handlers| handlers.borrow().get(&id).cloned());
    if let Some(handler) = handler {
        (handler.borrow_mut())(event);
    }
}

/// Makes the inbox post to Ecore's main loop (the platform's start).
pub(crate) fn start(main_loop: &Arc<EcoreLoop>) {
    let main_loop = Arc::clone(main_loop);
    let _ = INBOX.set(Inbox::new(move |job| main_loop.post(job), route));
}

/// What the player's callbacks get as user data: the player's id and, for the texture output,
/// the slot its decoded frames go to.
struct Sink {
    id: PlayerId,
    frames: Option<Arc<FrameSlot>>,
    /// The decoded planes, copied out so the packet goes back to the decoder at once.
    scratch: std::sync::Mutex<Vec<u8>>,
}

/// The sink behind a callback's user data.
///
/// # Safety
///
/// `data` is the `Sink` the player was registered with; it is freed only after
/// `player_destroy`, after which the player calls back no more.
unsafe fn sink<'a>(data: *mut c_void) -> &'a Sink {
    // SAFETY: the caller's contract.
    unsafe { &*data.cast::<Sink>() }
}

fn post(data: *mut c_void, event: Event) {
    if let Some(inbox) = INBOX.get() {
        // SAFETY: called only from the player's callbacks with their user data.
        inbox.post(unsafe { sink(data) }.id, event);
    }
}

/// The planes of a decoded packet, as `video_frames` takes them.
///
/// # Safety
///
/// `packet` is a live packet; the returned slices borrow its memory until it is unreferenced.
unsafe fn decoded<'a>(
    packet: sys::media_packet_h,
) -> Option<(PixelFormat, usize, usize, Vec<Plane<'a>>)> {
    let mut format = ptr::null_mut();
    // SAFETY: a live packet and an out pointer.
    if unsafe { sys::media_packet_get_format(packet, &mut format) } != 0 || format.is_null() {
        return None;
    }
    let (mut mime, mut width, mut height, mut avg, mut max) = (0, 0, 0, 0, 0);
    // SAFETY: the packet's format and out pointers; the format is unreferenced right after.
    let info = unsafe {
        let info = sys::media_format_get_video_info(
            format,
            &mut mime,
            &mut width,
            &mut height,
            &mut avg,
            &mut max,
        );
        sys::media_format_unref(format);
        info
    };
    if info != 0 || width <= 0 || height <= 0 {
        return None;
    }
    let (pixel_format, count) = match mime {
        sys::MEDIA_FORMAT_NV12 => (PixelFormat::Nv12, 2),
        sys::MEDIA_FORMAT_NV21 => (PixelFormat::Nv21, 2),
        sys::MEDIA_FORMAT_I420 => (PixelFormat::I420, 3),
        _ => {
            static WARNED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                log::warn!("decoded video frames in format {mime:#x} cannot be drawn as a texture");
            }
            return None;
        }
    };
    let mut planes = Vec::with_capacity(count);
    for index in 0..count as c_int {
        let (mut data, mut stride, mut rows) = (ptr::null_mut(), 0, 0);
        // SAFETY: a live packet, a plane index below its count and out pointers.
        let ok = unsafe {
            sys::media_packet_get_video_plane_data_ptr(packet, index, &mut data) == 0
                && sys::media_packet_get_video_stride_width(packet, index, &mut stride) == 0
                && sys::media_packet_get_video_stride_height(packet, index, &mut rows) == 0
        };
        if !ok || data.is_null() || stride <= 0 || rows <= 0 {
            return None;
        }
        // SAFETY: the plane holds `stride × rows` bytes for the packet's lifetime.
        let data =
            unsafe { std::slice::from_raw_parts(data.cast::<u8>(), (stride * rows) as usize) };
        planes.push(Plane {
            data,
            stride: stride as usize,
        });
    }
    Some((pixel_format, width as usize, height as usize, planes))
}

/// `player_media_packet_video_decoded_cb`: when the main thread wants a frame, copies its planes
/// out, releases the packet at once (`player.h`: the decoder needs it back as quickly as
/// possible), then converts the copy.
unsafe extern "C" fn on_decoded(packet: sys::media_packet_h, data: *mut c_void) {
    if packet.is_null() {
        return;
    }
    // SAFETY: called by the player with the user data it was registered with.
    let sink = unsafe { sink(data) };
    let Some(slot) = &sink.frames else {
        // SAFETY: the player hands the packet over; it is used no more.
        unsafe { sys::media_packet_unref(packet) };
        return;
    };
    let mut scratch = sink
        .scratch
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let copied = if slot.wants_frame() {
        // SAFETY: the packet is live until it is unreferenced below; the planes are copied first.
        unsafe { decoded(packet) }.map(|(format, width, height, planes)| {
            scratch.clear();
            let spans: Vec<(usize, usize, usize)> = planes
                .iter()
                .map(|plane| {
                    let start = scratch.len();
                    scratch.extend_from_slice(plane.data);
                    (start, plane.data.len(), plane.stride)
                })
                .collect();
            (format, width, height, spans)
        })
    } else {
        None
    };
    // SAFETY: the player hands the packet over; it is used no more.
    unsafe { sys::media_packet_unref(packet) };
    let Some((format, width, height, spans)) = copied else {
        return;
    };
    let planes: Vec<Plane<'_>> = spans
        .iter()
        .map(|&(start, len, stride)| Plane {
            data: &scratch[start..start + len],
            stride,
        })
        .collect();
    let frame = YuvFrame {
        format,
        width,
        height,
        planes: &planes,
    };
    let mut pixels = slot.buffer();
    if video_frames::to_rgba(&frame, Matrix::for_height(height), &mut pixels).is_ok()
        && slot.offer(RgbaFrame {
            width: width as u32,
            height: height as u32,
            pixels,
        })
        && let Some(inbox) = INBOX.get()
    {
        inbox.post(sink.id, Event::Frame);
    }
}

unsafe extern "C" fn on_prepared(data: *mut c_void) {
    post(data, Event::Prepared);
}
unsafe extern "C" fn on_seek_completed(data: *mut c_void) {
    post(data, Event::SeekCompleted);
}
unsafe extern "C" fn on_completed(data: *mut c_void) {
    post(data, Event::Completed);
}
unsafe extern "C" fn on_error(code: c_int, data: *mut c_void) {
    post(data, Event::Error(code));
}
unsafe extern "C" fn on_buffering(percent: c_int, data: *mut c_void) {
    post(data, Event::Buffering(percent));
}
unsafe extern "C" fn on_interrupted(_code: sys::player_interrupted_code_e, data: *mut c_void) {
    post(data, Event::Interrupted);
}

fn check(code: c_int) -> Result<(), i32> {
    if code == 0 { Ok(()) } else { Err(code) }
}

/// A `player_h` and the user data its callbacks carry.
pub(crate) struct Capi {
    handle: sys::player_h,
    sink: *mut Sink,
    id: PlayerId,
}

impl Capi {
    /// Creates a player whose events go to `handler` on the main thread and, for the texture
    /// output, whose decoded frames go to `frames`.
    fn create(handler: Box<dyn FnMut(Event)>, frames: Option<Arc<FrameSlot>>) -> Result<Self> {
        INBOX
            .get()
            .ok_or_else(|| anyhow!("video needs the Tizen platform running"))?;
        let mut handle = ptr::null_mut();
        // SAFETY: an out pointer for the new handle.
        check(unsafe { sys::player_create(&mut handle) })
            .map_err(|code| anyhow!("player_create: {code}"))?;
        let id = PlayerId(NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let sink = Box::into_raw(Box::new(Sink {
            id,
            frames,
            scratch: std::sync::Mutex::new(Vec::new()),
        }));
        let data = sink.cast::<c_void>();
        // SAFETY: a live handle; the callbacks are static and `data` outlives the player.
        let registered = unsafe {
            [
                sys::player_set_completed_cb(handle, Some(on_completed), data),
                sys::player_set_error_cb(handle, Some(on_error), data),
                sys::player_set_buffering_cb(handle, Some(on_buffering), data),
                sys::player_set_interrupted_cb(handle, Some(on_interrupted), data),
            ]
        };
        let capi = Self { handle, sink, id };
        if let Some(code) = registered.into_iter().find(|code| *code != 0) {
            drop(capi);
            return Err(anyhow!("registering the player's callbacks: {code}"));
        }
        HANDLERS.with(|handlers| {
            handlers
                .borrow_mut()
                .insert(id, Rc::new(RefCell::new(handler)))
        });
        Ok(capi)
    }

    /// The raw handle, for the outputs (`player_set_display`, decoded frames).
    pub(crate) fn handle(&self) -> sys::player_h {
        self.handle
    }

    fn data(&self) -> *mut c_void {
        self.sink.cast()
    }
}

impl Backend for Capi {
    fn set_uri(&mut self, uri: &str) -> Result<(), i32> {
        let uri = CString::new(uri).map_err(|_| -22)?;
        // SAFETY: a live handle and a NUL-terminated string the player copies.
        check(unsafe { sys::player_set_uri(self.handle, uri.as_ptr()) })
    }
    fn prepare_async(&mut self) -> Result<(), i32> {
        // SAFETY: as above; `data` outlives the player.
        check(unsafe { sys::player_prepare_async(self.handle, Some(on_prepared), self.data()) })
    }
    fn unprepare(&mut self) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_unprepare(self.handle) })
    }
    fn start(&mut self) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_start(self.handle) })
    }
    fn pause(&mut self) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_pause(self.handle) })
    }
    fn stop(&mut self) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_stop(self.handle) })
    }
    fn seek(&mut self, milliseconds: i32, accurate: bool) -> Result<(), i32> {
        // SAFETY: a live handle; `data` outlives the player.
        check(unsafe {
            sys::player_set_play_position(
                self.handle,
                milliseconds,
                accurate,
                Some(on_seek_completed),
                self.data(),
            )
        })
    }
    fn set_volume(&mut self, volume: f32) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_set_volume(self.handle, volume, volume) })
    }
    fn set_muted(&mut self, muted: bool) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_set_mute(self.handle, muted) })
    }
    fn set_looping(&mut self, looping: bool) -> Result<(), i32> {
        // SAFETY: a live handle.
        check(unsafe { sys::player_set_looping(self.handle, looping) })
    }
    fn position(&mut self) -> Result<i32, i32> {
        let mut ms = 0;
        // SAFETY: a live handle and an out pointer.
        check(unsafe { sys::player_get_play_position(self.handle, &mut ms) })?;
        Ok(ms)
    }
    fn duration(&mut self) -> Result<i32, i32> {
        let mut ms = 0;
        // SAFETY: a live handle and an out pointer.
        check(unsafe { sys::player_get_duration(self.handle, &mut ms) })?;
        Ok(ms)
    }
    fn state(&mut self) -> Result<State, i32> {
        let mut state = sys::PLAYER_STATE_NONE;
        // SAFETY: a live handle and an out pointer.
        check(unsafe { sys::player_get_state(self.handle, &mut state) })?;
        Ok(match state {
            sys::PLAYER_STATE_READY => State::Ready,
            sys::PLAYER_STATE_PLAYING => State::Playing,
            sys::PLAYER_STATE_PAUSED => State::Paused,
            sys::PLAYER_STATE_NONE => State::Destroyed,
            _ => State::Idle,
        })
    }
    fn destroy(&mut self) {
        if self.handle.is_null() {
            return;
        }
        HANDLERS.with(|handlers| handlers.borrow_mut().remove(&self.id));
        // SAFETY: a live handle, destroyed once; the player calls back no more afterwards.
        unsafe { sys::player_destroy(self.handle) };
        self.handle = ptr::null_mut();
        // SAFETY: made by `create` with `Box::into_raw`; nothing uses it after the destroy.
        drop(unsafe { Box::from_raw(self.sink) });
        self.sink = ptr::null_mut();
    }
}

impl Drop for Capi {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// The exported video surface under the window.
struct Overlay {
    subsurface: *mut sys::Ecore_Wl2_Subsurface,
    placement: OverlayPlacement,
}

impl Overlay {
    /// An exported subsurface below `window`, given to the player before it prepares.
    fn new(window: *mut sys::Ecore_Wl2_Window, player: sys::player_h) -> Result<Self> {
        // SAFETY: a live window.
        let subsurface = unsafe { sys::ecore_wl2_subsurface_new(window) };
        anyhow::ensure!(!subsurface.is_null(), "ecore_wl2_subsurface_new failed");
        let overlay = Self {
            subsurface,
            placement: OverlayPlacement::default(),
        };
        // SAFETY: a live subsurface; the handle string lives as long as it.
        let handle = unsafe {
            if sys::ecore_wl2_subsurface_export(subsurface) == sys::EINA_FALSE {
                return Err(anyhow!("ecore_wl2_subsurface_export failed"));
            }
            sys::ecore_wl2_subsurface_exported_surface_place_below(subsurface, ptr::null_mut());
            sys::ecore_wl2_subsurface_exported_surface_handle_get(subsurface)
        };
        anyhow::ensure!(!handle.is_null(), "the exported surface has no handle");
        // SAFETY: an idle player and the exported shell handle `player.h` asks for with
        // PLAYER_DISPLAY_TYPE_OVERLAY_SYNC_UI.
        let code = unsafe {
            sys::player_set_display(
                player,
                sys::PLAYER_DISPLAY_TYPE_OVERLAY_SYNC_UI,
                handle.cast_mut().cast(),
            )
        };
        anyhow::ensure!(code == 0, "player_set_display (overlay): {code}");
        Ok(overlay)
    }

    fn apply(&mut self, ops: Vec<OverlayOp>) {
        for op in ops {
            // SAFETY: a live subsurface.
            unsafe {
                match op {
                    OverlayOp::Move { x, y } => {
                        sys::ecore_wl2_subsurface_exported_surface_move(self.subsurface, x, y)
                    }
                    OverlayOp::Resize { width, height } => {
                        sys::ecore_wl2_subsurface_exported_surface_resize(
                            self.subsurface,
                            width,
                            height,
                        )
                    }
                    OverlayOp::Show => {
                        sys::ecore_wl2_subsurface_exported_surface_show(self.subsurface)
                    }
                    OverlayOp::Hide => {
                        sys::ecore_wl2_subsurface_exported_surface_hide(self.subsurface)
                    }
                }
            }
        }
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        // SAFETY: made by `new`; the player using it is destroyed first (field order).
        unsafe { sys::ecore_wl2_subsurface_del(self.subsurface) };
    }
}

/// Draws RGBA frames into a dynamic texture: a full-target triangle sampling the upload.
struct Blit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    upload: Option<(wgpu::Texture, u32, u32)>,
}

const BLIT_SHADER: &str = r#"
@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;
struct Varying {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
@vertex
fn vs(@builtin(vertex_index) index: u32) -> Varying {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Varying;
    out.position = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}
@fragment
fn fs(input: Varying) -> @location(0) vec4<f32> {
    return textureSample(frame, frame_sampler, input.uv);
}
"#;

impl Blit {
    fn new(context: &WgpuContext) -> Self {
        let device = &context.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("video_blit"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("video_blit"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("video_blit"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("video_blit"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.color_texture_format(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("video_blit"),
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
            upload: None,
        }
    }

    /// Uploads `frame` and draws it over `target`, on the context's queue.
    fn draw(&mut self, context: &WgpuContext, frame: &RgbaFrame, target: &wgpu::TextureView) {
        let (device, queue) = (&context.device, &context.queue);
        if !matches!(&self.upload, Some((_, w, h)) if (*w, *h) == (frame.width, frame.height)) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("video_frame"),
                size: wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.upload = Some((texture, frame.width, frame.height));
        }
        let Some((texture, _, _)) = &self.upload else {
            return;
        };
        queue.write_texture(
            texture.as_image_copy(),
            &frame.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame.width * 4),
                rows_per_image: None,
            },
            texture.size(),
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("video_blit"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("video_blit"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("video_blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
}

/// Decoded frames drawn as a dynamic texture.
struct TextureOutput {
    frames: Arc<FrameSlot>,
    /// The texture, its size, and the context it was made from.
    target: Option<(WgpuDynamicTexture, (u32, u32))>,
    blit: Option<Blit>,
}

impl TextureOutput {
    /// Draws the waiting frame (if any) into the texture, then paints the texture at `bounds`.
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window) {
        if let Some(frame) = self.frames.take() {
            if let Err(err) = self.draw(&frame, window) {
                log::warn!("drawing a video frame: {err:#}");
            }
            self.frames.recycle(frame.pixels);
        }
        if let Some((texture, _)) = &self.target
            && let Err(err) = texture.paint(window, bounds)
        {
            log::warn!("painting the video texture: {err:#}");
        }
    }

    fn draw(&mut self, frame: &RgbaFrame, window: &mut Window) -> Result<()> {
        let context = super::app::shared()
            .and_then(|shared| {
                shared
                    .window
                    .borrow()
                    .as_ref()
                    .and_then(|w| w.wgpu_context())
            })
            .ok_or_else(|| anyhow!("no wgpu context to draw video frames with"))?;
        let size = (frame.width, frame.height);
        if self.target.as_ref().is_none_or(|(_, made)| *made != size) {
            let texture = context.create_dynamic_texture(gpui::size(
                gpui::DevicePixels(frame.width as i32),
                gpui::DevicePixels(frame.height as i32),
            ))?;
            self.target = Some((texture, size));
        }
        let blit = self.blit.get_or_insert_with(|| Blit::new(&context));
        let (texture, _) = self.target.as_ref().expect("made above");
        texture.render(window, |view| {
            blit.draw(&context, frame, view);
            Ok(())
        })
    }
}

/// A player and its outputs. The window closes every one before it is freed (an overlay's
/// subsurface belongs to the window), and the player goes before its outputs.
struct Media {
    player: Player<Capi>,
    overlay: Option<Overlay>,
    texture: Option<TextureOutput>,
}

impl Media {
    fn close(&mut self) {
        self.player.destroy();
        self.overlay = None;
        self.texture = None;
    }
}

impl Drop for Media {
    fn drop(&mut self) {
        self.close();
    }
}

thread_local! {
    /// Every open player's media, for the window to close.
    static MEDIA: RefCell<Vec<std::rc::Weak<RefCell<Media>>>> = const { RefCell::new(Vec::new()) };
}

fn each_media(mut f: impl FnMut(&mut Media)) {
    let live: Vec<Rc<RefCell<Media>>> = MEDIA.with(|media| {
        let mut media = media.borrow_mut();
        media.retain(|weak| weak.strong_count() > 0);
        media.iter().filter_map(std::rc::Weak::upgrade).collect()
    });
    for media in live {
        match media.try_borrow_mut() {
            Ok(mut media) => f(&mut media),
            Err(_) => log::warn!("a video player is busy: skipped"),
        }
    }
}

/// The window is closing: every player stops and its subsurface goes while the window exists.
pub(crate) fn close_all() {
    each_media(Media::close);
}

/// After a frame was painted: without holes no video shows through, so the overlays' surfaces
/// are hidden (a video view that stopped painting leaves no video plane behind).
pub(crate) fn end_frame(has_holes: bool) {
    if has_holes {
        return;
    }
    each_media(|media| {
        if let Some(overlay) = &mut media.overlay {
            let ops = overlay.placement.hide();
            overlay.apply(ops);
        }
    });
}

/// A media player for a GPUI app on Tizen: an entity that emits [`Notice`]s on the main thread
/// and paints its video with [`VideoPlayer::element`].
pub struct VideoPlayer {
    media: Option<Rc<RefCell<Media>>>,
    output: Output,
    error: Option<String>,
}

impl EventEmitter<Notice> for VideoPlayer {}

impl VideoPlayer {
    /// A player for `uri` (a path, `file://` or a stream URL), starting to prepare. `output`
    /// `None` takes the overlay where the window can turn translucent, else the texture. When
    /// the media cannot be opened, [`error`](Self::error) says why and every call fails with
    /// [`VideoError::NotOpened`].
    pub fn new(uri: &str, output: Option<Output>, cx: &mut Context<Self>) -> Self {
        let window = super::app::shared().and_then(|shared| shared.window.borrow().clone());
        let output = Output::choose(
            output,
            window.as_ref().is_some_and(|w| w.translucency_available()),
        );
        match window
            .ok_or_else(|| anyhow!("video needs the Tizen window"))
            .and_then(|window| Self::open(uri, output, &window, cx))
        {
            Ok(media) => {
                log::info!("video {uri}: {output:?}");
                let media = Rc::new(RefCell::new(media));
                MEDIA.with(|all| all.borrow_mut().push(Rc::downgrade(&media)));
                Self {
                    media: Some(media),
                    output,
                    error: None,
                }
            }
            Err(err) => {
                log::error!("video {uri}: {err:#}");
                Self {
                    media: None,
                    output,
                    error: Some(format!("{err:#}")),
                }
            }
        }
    }

    fn open(
        uri: &str,
        output: Output,
        window: &super::window::WindowInner,
        cx: &mut Context<Self>,
    ) -> Result<Media> {
        let frames = (output == Output::Texture).then(|| Arc::new(FrameSlot::default()));
        let this: WeakEntity<Self> = cx.weak_entity();
        let mut async_cx: AsyncApp = cx.to_async();
        let capi = Capi::create(
            Box::new(move |event| {
                let _ = this.update(&mut async_cx, |player, cx| player.handle(event, cx));
            }),
            frames.clone(),
        )?;
        let handle = capi.handle();
        // From here on a failure drops `media`: the player is destroyed before its outputs.
        let mut media = Media {
            player: Player::new(capi),
            overlay: None,
            texture: None,
        };
        match output {
            Output::Overlay => media.overlay = Some(Overlay::new(window.ecore(), handle)?),
            Output::Texture => {
                // SAFETY: an idle player; the callback's user data is the player's sink, which
                // outlives it.
                let code = unsafe {
                    let code = sys::player_set_display(
                        handle,
                        sys::PLAYER_DISPLAY_TYPE_NONE,
                        ptr::null_mut(),
                    );
                    if code == 0 {
                        sys::player_set_media_packet_video_frame_decoded_cb(
                            handle,
                            Some(on_decoded),
                            media.player.backend().data(),
                        )
                    } else {
                        code
                    }
                };
                anyhow::ensure!(code == 0, "setting up decoded frames: {code}");
                media.texture = Some(TextureOutput {
                    frames: frames.expect("made for the texture output"),
                    target: None,
                    blit: None,
                });
            }
        }
        media.player.set_uri(uri).context("setting the media")?;
        media.player.prepare().context("preparing the media")?;
        Ok(media)
    }

    /// Why the media could not be opened.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Runs `f` on the player (`VideoError::NotOpened` when the media did not open).
    fn with<R>(
        &self,
        f: impl FnOnce(&mut Player<Capi>) -> Result<R, VideoError>,
    ) -> Result<R, VideoError> {
        let media = self.media.as_ref().ok_or(VideoError::NotOpened)?;
        let mut media = media.try_borrow_mut().map_err(|_| VideoError::NotOpened)?;
        f(&mut media.player)
    }

    fn handle(&mut self, event: Event, cx: &mut Context<Self>) {
        let Some(media) = &self.media else {
            return;
        };
        let (notices, overlay) = {
            let mut media = media.borrow_mut();
            (media.player.handle(event), media.overlay.is_some())
        };
        for notice in notices {
            if notice == Notice::Ready && overlay {
                // `player.h`: the first video buffer shows once the window commits after prepare.
                if let Some(window) = super::app::shared().and_then(|s| s.window.borrow().clone()) {
                    window.commit();
                }
            }
            cx.emit(notice);
        }
        cx.notify();
    }

    /// The output the player draws to.
    pub fn output(&self) -> Output {
        self.output
    }

    /// Paints the video at `bounds`: a hole over the moved video surface (overlay), or the
    /// latest frame (texture).
    pub fn paint(
        &mut self,
        bounds: Bounds<Pixels>,
        corner_radii: Corners<Pixels>,
        window: &mut Window,
    ) {
        let Some(media) = &self.media else {
            return;
        };
        let mut media = media.borrow_mut();
        let media = &mut *media;
        if let Some(overlay) = &mut media.overlay {
            window.paint_hole(bounds, corner_radii);
            let rect = DeviceRect::snapped(
                f32::from(bounds.origin.x),
                f32::from(bounds.origin.y),
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
                window.scale_factor(),
            );
            let ops = overlay.placement.place(rect);
            overlay.apply(ops);
        }
        if let Some(texture) = &mut media.texture {
            texture.paint(bounds, window);
        }
    }

    /// An element that paints `player`'s video over its whole bounds.
    pub fn element(player: Entity<Self>) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, (), window, cx| {
                player.update(cx, |player, _| {
                    player.paint(bounds, Corners::default(), window)
                });
            },
        )
        .size_full()
    }

    pub fn state(&self) -> State {
        self.with(|p| Ok(p.state())).unwrap_or(State::Destroyed)
    }

    /// Prepares the media again after the system unprepared the player (an interruption or a
    /// failed prepare: the state is idle); [`Notice::Ready`] follows.
    pub fn prepare(&mut self, cx: &mut Context<Self>) -> Result<(), VideoError> {
        let result = self.with(|p| p.prepare());
        cx.notify();
        result
    }

    pub fn play(&mut self, cx: &mut Context<Self>) -> Result<(), VideoError> {
        let result = self.with(|p| p.play());
        cx.notify();
        result
    }

    pub fn pause(&mut self, cx: &mut Context<Self>) -> Result<(), VideoError> {
        let result = self.with(|p| p.pause());
        cx.notify();
        result
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) -> Result<(), VideoError> {
        let result = self.with(|p| p.stop());
        cx.notify();
        result
    }

    pub fn seek(&mut self, position: Duration, cx: &mut Context<Self>) -> Result<(), VideoError> {
        let result = self.with(|p| p.seek(position, false));
        cx.notify();
        result
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<(), VideoError> {
        self.with(|p| p.set_volume(volume))
    }

    pub fn set_muted(&mut self, muted: bool) -> Result<(), VideoError> {
        self.with(|p| p.set_muted(muted))
    }

    pub fn set_looping(&mut self, looping: bool) -> Result<(), VideoError> {
        self.with(|p| p.set_looping(looping))
    }

    pub fn position(&mut self) -> Result<Duration, VideoError> {
        self.with(|p| p.position())
    }

    pub fn duration(&mut self) -> Result<Duration, VideoError> {
        self.with(|p| p.duration())
    }
}
