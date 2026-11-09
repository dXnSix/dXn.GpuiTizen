//! What draws the window (`EGL-4`, design `d24-platform-egl`): the platform-owned EGL path or
//! wgpu's own surface, as `crate::renderer_select` decides.

use std::sync::Arc;

use gpui::{GpuSpecs, PlatformAtlas, Scene};
use gpui_wgpu::{WgpuRenderer, WgpuSurfaceConfig};

use super::platform::Shared;
use super::window::RawWindow;
use crate::egl::{EglOptions, EglWindow};
use crate::renderer_select::{self, Choice};

/// What came of drawing a frame.
pub(crate) enum Drawn {
    Presented,
    /// Nothing changed: no swap, no frame callback.
    Nothing,
    /// GPUI drew, but presenting failed.
    Failed,
}

pub(crate) enum Backend {
    Surface(Box<WgpuRenderer>),
    Egl(Box<EglWindow>),
}

impl Backend {
    /// The renderer the app asked for, falling back from `Auto`'s EGL path to wgpu's surface.
    pub(crate) fn open(
        shared: &Shared,
        raw: &RawWindow,
        config: WgpuSurfaceConfig,
    ) -> anyhow::Result<Self> {
        let env = std::env::var(renderer_select::ENV).ok();
        let (requested, warning) = renderer_select::requested(shared.renderer, env.as_deref());
        if let Some(warning) = warning {
            log::warn!("{warning}");
        }
        let mut egl = None;
        let outcome = renderer_select::tries_egl(requested).then(|| {
            let options = EglOptions {
                alpha: false,
                blob_cache_dir: shared
                    .cache_dir
                    .as_deref()
                    .map(|dir| dir.join(crate::blob_cache::DIR_NAME)),
                fail_at_end: false,
            };
            // SAFETY: the window's live wl_display and wl_surface, which outlive the renderer
            // (it is destroyed before the window is freed).
            match unsafe {
                EglWindow::new(raw.display_ptr(), raw.surface_ptr(), config.size, &options)
            } {
                Ok(window) => {
                    egl = Some(window);
                    Ok(())
                }
                Err(err) => Err(format!("{err:#}")),
            }
        });
        let choice = renderer_select::choose(requested, outcome);
        if let Some(message) = shared.renderer_fallback.borrow_mut().message(&choice) {
            log::warn!("{message}");
        }
        match (choice, egl) {
            (Choice::PlatformEgl, Some(window)) => Ok(Self::Egl(Box::new(window))),
            (Choice::Failed(reason), _) => {
                anyhow::bail!("the platform-owned EGL path failed: {reason}")
            }
            _ => {
                let renderer = WgpuRenderer::new(shared.gpu.clone(), raw, config, None)?;
                Ok(Self::Surface(Box::new(renderer)))
            }
        }
    }

    /// Whether the renderer asks for the next frame in full.
    pub(crate) fn needs_redraw(&mut self) -> bool {
        match self {
            Self::Surface(renderer) => renderer.needs_redraw(),
            Self::Egl(_) => false,
        }
    }

    /// Draws `scene`; `commit` runs before the frame is presented, and only then.
    pub(crate) fn draw(&mut self, scene: &Scene, commit: impl FnOnce()) -> Drawn {
        match self {
            Self::Surface(renderer) => {
                commit();
                if renderer.draw(scene) {
                    Drawn::Presented
                } else {
                    Drawn::Failed
                }
            }
            Self::Egl(window) => match window.draw_presenting(scene, commit) {
                Ok(report) if report.presented => Drawn::Presented,
                Ok(_) => Drawn::Nothing,
                Err(err) => {
                    log::error!("drawing through EGL failed: {err:#}");
                    Drawn::Failed
                }
            },
        }
    }

    pub(crate) fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        match self {
            Self::Surface(renderer) => renderer.sprite_atlas().clone(),
            Self::Egl(window) => window.sprite_atlas(),
        }
    }

    /// Whether the window can turn translucent for holes (the EGL path with a switchable
    /// config); wgpu's own surface keeps the alpha mode it was made with.
    pub(crate) fn translucency_available(&self) -> bool {
        match self {
            Self::Surface(_) => false,
            Self::Egl(window) => window.translucency_available(),
        }
    }

    /// Gives the window's surface an alpha channel or takes it away; whether it now has `alpha`.
    pub(crate) fn set_translucent(&mut self, alpha: bool) -> bool {
        match self {
            Self::Surface(_) => !alpha,
            Self::Egl(window) => match window.set_alpha(alpha) {
                Ok(()) => true,
                Err(err) => {
                    log::warn!("the window cannot turn translucent: {err:#}");
                    window.alpha() == alpha
                }
            },
        }
    }

    /// The wgpu context the window draws with (dynamic textures are made from it).
    pub(crate) fn wgpu_context(
        &self,
        gpu: &gpui_wgpu::GpuContext,
    ) -> Option<gpui_wgpu::WgpuContext> {
        match self {
            Self::Surface(_) => gpu.borrow().clone(),
            Self::Egl(window) => window.wgpu_context(),
        }
    }

    pub(crate) fn gpu_specs(&self) -> Option<GpuSpecs> {
        match self {
            Self::Surface(renderer) => renderer.gpu_specs(),
            Self::Egl(_) => None,
        }
    }

    /// Pause: frees the surface, the target-sized textures and the atlas textures (GPUI keeps
    /// the atlas object).
    pub(crate) fn release(&mut self) {
        match self {
            Self::Surface(renderer) => {
                renderer.unconfigure_surface();
                renderer.sprite_atlas().clear();
            }
            Self::Egl(window) => window.release_surface(),
        }
    }

    /// Resume: a new surface on the same handles; the next frame is drawn whole.
    pub(crate) fn rebuild(
        &mut self,
        raw: &RawWindow,
        config: WgpuSurfaceConfig,
        gpu: &gpui_wgpu::GpuContext,
    ) {
        match self {
            Self::Surface(renderer) => {
                let instance = gpu
                    .borrow()
                    .as_ref()
                    .map(|context| context.instance.clone());
                let Some(instance) = instance else {
                    log::error!("no wgpu context to rebuild the surface with");
                    return;
                };
                if let Err(err) = renderer.replace_surface(raw, config, &instance) {
                    log::error!("rebuilding the surface failed: {err:#}");
                }
            }
            Self::Egl(window) => {
                if let Err(err) = window.rebuild_surface() {
                    log::error!("rebuilding the EGL surface failed: {err:#}");
                }
            }
        }
    }

    /// Before the window is freed: the renderer's EGL objects go first.
    pub(crate) fn destroy(&mut self) {
        match self {
            Self::Surface(renderer) => renderer.destroy(),
            Self::Egl(window) => window.shutdown(),
        }
    }
}
