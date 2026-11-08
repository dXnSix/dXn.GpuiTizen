//! The probe's real backend: wgpu with only its GL backend, on the window's Wayland handles.

use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, PoisonError};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle, RawWindowHandle,
    WaylandDisplayHandle, WaylandWindowHandle,
};

use crate::probe::{AdapterInfo, Backend};

/// The `wl_display`, as the display handle wgpu's GL backend needs for EGL.
#[derive(Debug)]
struct WlDisplay(NonNull<c_void>);

// SAFETY: the pointer is only handed to wgpu, which uses the wl_display from the thread that owns
// it (the main thread, where the probe runs); Ecore keeps the display alive for the app's life.
unsafe impl Send for WlDisplay {}
// SAFETY: as above; the handle is never dereferenced here.
unsafe impl Sync for WlDisplay {}

impl HasDisplayHandle for WlDisplay {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(self.0));
        // SAFETY: the wl_display outlives every wgpu object of the probe.
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

pub struct WgpuBackend {
    display: NonNull<c_void>,
    surface_ptr: NonNull<c_void>,
    width: u32,
    height: u32,
    instance: Option<wgpu::Instance>,
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<(wgpu::Device, wgpu::Queue)>,
    configured: bool,
    frame: u32,
    /// Errors wgpu reports outside an error scope (its default handler would panic, and with
    /// `panic = "abort"` kill the app before the report is served).
    uncaptured: Arc<Mutex<Vec<String>>>,
}

impl WgpuBackend {
    pub fn new(
        display: NonNull<c_void>,
        surface: NonNull<c_void>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            display,
            surface_ptr: surface,
            width,
            height,
            instance: None,
            surface: None,
            adapter: None,
            device: None,
            configured: false,
            frame: 0,
            uncaptured: Arc::default(),
        }
    }

    /// Runs `work` inside a validation and an out-of-memory error scope and turns a captured
    /// error, or one wgpu reported uncaptured meanwhile, into a stage failure.
    fn scoped<T>(&self, work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        let Some((device, _)) = self.device.as_ref() else {
            return work();
        };
        let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let result = work();
        let validation = pollster::block_on(validation.pop());
        let oom = pollster::block_on(oom.pop());
        let uncaptured: Vec<String> = std::mem::take(
            &mut *self
                .uncaptured
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        match (result, validation.or(oom)) {
            (Err(err), _) => Err(err),
            (Ok(_), Some(err)) => Err(err.to_string()),
            (Ok(_), None) if !uncaptured.is_empty() => Err(uncaptured.join("; ")),
            (Ok(value), None) => Ok(value),
        }
    }
}

fn missing(what: &str) -> String {
    format!("no {what} (an earlier stage did not run)")
}

impl Backend for WgpuBackend {
    fn instance(&mut self) -> Result<(), String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: Some(Box::new(WlDisplay(self.display))),
        });
        self.instance = Some(instance);
        Ok(())
    }

    fn surface(&mut self) -> Result<(), String> {
        let instance = self.instance.as_ref().ok_or_else(|| missing("instance"))?;
        let target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                self.display,
            ))),
            raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(self.surface_ptr)),
        };
        // SAFETY: the wl_display and wl_surface come from Ecore_Wl2 and live as long as the window,
        // which the app keeps until it terminates, after the probe has finished.
        let surface =
            unsafe { instance.create_surface_unsafe(target) }.map_err(|e| e.to_string())?;
        self.surface = Some(surface);
        Ok(())
    }

    fn adapter(&mut self) -> Result<AdapterInfo, String> {
        let instance = self.instance.as_ref().ok_or_else(|| missing("instance"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: self.surface.as_ref(),
            apply_limit_buckets: false,
        }))
        .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        let vertex_storage = adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::VERTEX_STORAGE)
            && adapter.limits().max_storage_buffers_per_shader_stage > 0;
        self.adapter = Some(adapter);
        Ok(AdapterInfo {
            name: format!("{} ({:?}, {})", info.name, info.backend, info.driver_info),
            vertex_storage,
        })
    }

    fn device(&mut self, webgl2_limits: bool) -> Result<(), String> {
        let adapter = self.adapter.as_ref().ok_or_else(|| missing("adapter"))?;
        let limits = if webgl2_limits {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::downlevel_defaults()
        }
        .using_resolution(adapter.limits())
        .using_alignment(adapter.limits());
        let device = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("dxn_gpui_tizen_diag"),
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        }))
        .map_err(|e| e.to_string())?;
        let uncaptured = Arc::clone(&self.uncaptured);
        device
            .0
            .on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
                uncaptured
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(error.to_string());
            }));
        self.device = Some(device);
        Ok(())
    }

    fn configure(&mut self) -> Result<String, String> {
        let surface = self.surface.as_ref().ok_or_else(|| missing("surface"))?;
        let adapter = self.adapter.as_ref().ok_or_else(|| missing("adapter"))?;
        let (device, _) = self.device.as_ref().ok_or_else(|| missing("device"))?;
        let mut config = surface
            .get_default_config(adapter, self.width, self.height)
            .ok_or("the surface is not supported by the adapter")?;
        let caps = surface.get_capabilities(adapter);
        config.present_mode = wgpu::PresentMode::Fifo;
        if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            config.alpha_mode = wgpu::CompositeAlphaMode::Opaque;
        }
        let described = format!("{:?} {:?}", config.format, config.alpha_mode);
        self.scoped(|| {
            surface.configure(device, &config);
            Ok(())
        })?;
        self.configured = true;
        Ok(described)
    }

    fn present(&mut self) -> Result<(), String> {
        let shade = f64::from(self.frame % 120) / 120.0;
        self.frame += 1;
        self.scoped(|| self.present_frame(shade))
    }
}

impl WgpuBackend {
    fn present_frame(&self, shade: f64) -> Result<(), String> {
        let surface = self.surface.as_ref().ok_or_else(|| missing("surface"))?;
        let (device, queue) = self.device.as_ref().ok_or_else(|| missing("device"))?;
        if !self.configured {
            return Err(missing("configured surface"));
        }
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            other => return Err(format!("acquiring a frame: {other:?}")),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("probe"),
        });
        // A slowly changing clear colour makes the presents visible on the TV.
        {
            let attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2 + 0.5 * shade,
                        b: 0.4,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            };
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("probe clear"),
                color_attachments: &[Some(attachment)],
                ..Default::default()
            });
        }
        queue.submit([encoder.finish()]);
        queue.present(frame);
        Ok(())
    }
}
