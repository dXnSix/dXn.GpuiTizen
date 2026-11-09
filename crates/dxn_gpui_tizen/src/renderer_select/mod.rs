//! Which renderer draws the window (`EGL-4`, design `d24-platform-egl`).
//!
//! - [`Renderer::Auto`] (the default): the platform-owned EGL path when its context and wgpu's
//!   adapter on it come up, else wgpu's own surface; the reason for falling back is logged once.
//! - [`Renderer::PlatformEgl`]: the EGL path only; its failure fails the window.
//! - [`Renderer::WgpuSurface`]: wgpu's own surface (`SHELL-2`), never trying EGL.
//!
//! `DXN_GPUI_TIZEN_RENDERER` (`auto`, `egl`, `wgpu`) overrides
//! [`RunOptions::renderer`](crate::RunOptions::renderer).

/// The renderer an app asks for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Renderer {
    #[default]
    Auto,
    PlatformEgl,
    WgpuSurface,
}

/// Overrides [`RunOptions::renderer`](crate::RunOptions::renderer).
pub const ENV: &str = "DXN_GPUI_TIZEN_RENDERER";

impl Renderer {
    /// The value of [`ENV`]: `auto`, `egl` (or `platform-egl`), `wgpu` (or `wgpu-surface`).
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "egl" | "platform-egl" | "platform_egl" => Some(Self::PlatformEgl),
            "wgpu" | "wgpu-surface" | "wgpu_surface" => Some(Self::WgpuSurface),
            _ => None,
        }
    }
}

/// The renderer to use: the environment's when it parses, else the options'; and a warning for
/// an environment value that does not.
pub fn requested(option: Renderer, env: Option<&str>) -> (Renderer, Option<String>) {
    match env {
        None => (option, None),
        Some(value) => match Renderer::parse(value) {
            Some(renderer) => (renderer, None),
            None => (
                option,
                Some(format!(
                    "{ENV}={value:?} is not auto, egl or wgpu; using {option:?}"
                )),
            ),
        },
    }
}

/// What the window draws with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    PlatformEgl,
    /// wgpu's surface, with the reason when the EGL path was tried and failed.
    WgpuSurface {
        fallback: Option<String>,
    },
    /// The requested EGL path failed.
    Failed(String),
}

/// Whether to try the EGL path for `requested`.
pub fn tries_egl(requested: Renderer) -> bool {
    requested != Renderer::WgpuSurface
}

/// The choice for `requested`, given the EGL path's outcome when it was tried.
pub fn choose(requested: Renderer, egl: Option<Result<(), String>>) -> Choice {
    match (requested, egl) {
        (Renderer::WgpuSurface, _) | (_, None) => Choice::WgpuSurface { fallback: None },
        (_, Some(Ok(()))) => Choice::PlatformEgl,
        (Renderer::Auto, Some(Err(reason))) => Choice::WgpuSurface {
            fallback: Some(reason),
        },
        (Renderer::PlatformEgl, Some(Err(reason))) => Choice::Failed(reason),
    }
}

/// Logs a fallback reason once per process (windows are rebuilt on resume).
#[derive(Debug, Default)]
pub struct FallbackLog {
    logged: bool,
}

impl FallbackLog {
    /// The message to log for `choice`, the first time it falls back.
    pub fn message(&mut self, choice: &Choice) -> Option<String> {
        match choice {
            Choice::WgpuSurface {
                fallback: Some(reason),
            } if !self.logged => {
                self.logged = true;
                Some(format!(
                    "the platform-owned EGL path failed ({reason}); drawing through wgpu's surface"
                ))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
