//! Partial presents (`EGL-3`, design `d24-platform-egl`): the renderer's damage as EGL calls.
//!
//! The renderer (gpui_wgpu's `WgpuExternalRenderer`) keeps the damage history and, given the
//! back buffer's age, reports before drawing the frame's damage and the region it redraws, in GL's
//! bottom-left coordinates. This module decides what EGL is told:
//!
//! - the age passed to the renderer: `EGL_BUFFER_AGE_EXT` when the display offers
//!   `EGL_EXT_buffer_age` and the query succeeds, else 0 (a full frame);
//! - `eglSetDamageRegionKHR` with the redraw region before drawing, with `EGL_KHR_partial_update`;
//! - `eglSwapBuffersWithDamageKHR` (or `EXT`) with the frame's damage after drawing, else a plain
//!   `eglSwapBuffers`;
//! - no swap at all when the renderer drew nothing: the back buffer holds an older frame.
//!
//! A damage region stays in force until the frame is swapped (`EGL_KHR_partial_update`), and a
//! second region in the same frame is rejected. A frame that fails after its region was set
//! cannot be swapped (it holds a half-drawn frame), so [`FrameState`] has the window surface made
//! again: its buffers start over with age 0 and no region, and the next frame is drawn whole.

use gpui::{Bounds, DevicePixels};

/// What the display offers for partial presents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageSupport {
    pub buffer_age: bool,
    pub partial_update: bool,
    /// `EGL_KHR_swap_buffers_with_damage`.
    pub swap_with_damage_khr: bool,
    /// `EGL_EXT_swap_buffers_with_damage`.
    pub swap_with_damage_ext: bool,
}

fn has(extensions: &str, name: &str) -> bool {
    extensions.split_whitespace().any(|e| e == name)
}

impl DamageSupport {
    /// From the display's `EGL_EXTENSIONS` string.
    pub fn from_extensions(extensions: &str) -> Self {
        Self {
            buffer_age: has(extensions, "EGL_EXT_buffer_age"),
            partial_update: has(extensions, "EGL_KHR_partial_update"),
            swap_with_damage_khr: has(extensions, "EGL_KHR_swap_buffers_with_damage"),
            swap_with_damage_ext: has(extensions, "EGL_EXT_swap_buffers_with_damage"),
        }
    }

    /// The age to draw for: the queried `EGL_BUFFER_AGE_EXT`, or 0 when unknown.
    pub fn age(&self, queried: Option<i32>) -> u32 {
        match queried {
            Some(age) if self.buffer_age && age > 0 => age as u32,
            _ => 0,
        }
    }

    /// The swap entry point to load, if any (`eglSwapBuffersWithDamageKHR` preferred).
    pub fn swap_with_damage_name(&self) -> Option<&'static str> {
        if self.swap_with_damage_khr {
            Some("eglSwapBuffersWithDamageKHR")
        } else if self.swap_with_damage_ext {
            Some("eglSwapBuffersWithDamageEXT")
        } else {
            None
        }
    }
}

/// EGL's rectangle array: `x, y, width, height` per rectangle.
pub fn egl_rects(rects: &[Bounds<DevicePixels>]) -> Vec<i32> {
    rects
        .iter()
        .flat_map(|r| [r.origin.x.0, r.origin.y.0, r.size.width.0, r.size.height.0])
        .collect()
}

/// The EGL calls of one frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// `eglSetDamageRegionKHR` before drawing.
    pub set_damage_region: Option<Vec<i32>>,
    /// Present the frame.
    pub swap: Swap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Swap {
    /// Nothing was drawn: keep presenting the last frame.
    None,
    /// `eglSwapBuffers`.
    Full,
    /// `eglSwapBuffersWithDamage*` with these rectangles.
    WithDamage(Vec<i32>),
}

/// The calls for a frame whose renderer reported `damage` and `redraw` (target coordinates).
pub fn plan(
    support: &DamageSupport,
    damage: &[Bounds<DevicePixels>],
    redraw: &[Bounds<DevicePixels>],
) -> Plan {
    if redraw.is_empty() {
        return Plan {
            set_damage_region: None,
            swap: Swap::None,
        };
    }
    let set_damage_region = support.partial_update.then(|| egl_rects(redraw));
    let swap = if support.swap_with_damage_name().is_some() && !damage.is_empty() {
        Swap::WithDamage(egl_rects(damage))
    } else {
        Swap::Full
    };
    Plan {
        set_damage_region,
        swap,
    }
}

/// Whether the frame being drawn has a damage region in force.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameState {
    region_open: bool,
}

/// What follows a frame that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recovery {
    /// Nothing is in force: the next frame starts as usual.
    None,
    /// A region is in force for a frame that cannot be swapped: make the surface again.
    NewSurface,
}

impl FrameState {
    /// `eglSetDamageRegionKHR` succeeded for this frame.
    pub fn region_set(&mut self) {
        self.region_open = true;
    }

    /// The frame was swapped: the region ends with it.
    pub fn swapped(&mut self) {
        self.region_open = false;
    }

    /// Whether a region is in force.
    pub fn region_open(&self) -> bool {
        self.region_open
    }

    /// The frame failed before its swap.
    pub fn failed(&mut self) -> Recovery {
        if std::mem::take(&mut self.region_open) {
            Recovery::NewSurface
        } else {
            Recovery::None
        }
    }
}

#[cfg(test)]
mod tests;
