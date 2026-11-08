//! The one display: the UI plane at the window's size, refresh from the measured frame callbacks.

use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use gpui::{Bounds, DisplayId, Pixels, PlatformDisplay, Point, px, size};
use uuid::Uuid;

use super::platform::Shared;
use crate::display;

#[derive(Debug)]
pub(crate) struct TizenDisplay {
    pixels: (u32, u32),
    scale: f32,
    refresh_hz: f32,
}

impl TizenDisplay {
    pub(crate) fn new(shared: &Rc<Shared>) -> Self {
        Self {
            pixels: shared.display_size.get(),
            scale: shared.scale,
            refresh_hz: shared.refresh.borrow().hz(),
        }
    }
}

impl PlatformDisplay for TizenDisplay {
    fn id(&self) -> DisplayId {
        DisplayId::new(1)
    }

    fn uuid(&self) -> Result<Uuid> {
        let name = format!("tizen-ui-plane-{}x{}", self.pixels.0, self.pixels.1);
        Ok(Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()))
    }

    fn bounds(&self) -> Bounds<Pixels> {
        let (w, h) = display::logical_size(self.pixels, self.scale);
        Bounds {
            origin: Point::default(),
            size: size(px(w), px(h)),
        }
    }

    fn refresh_interval(&self) -> Option<Duration> {
        gpui::refresh_interval_from_hz(f64::from(self.refresh_hz))
    }
}
