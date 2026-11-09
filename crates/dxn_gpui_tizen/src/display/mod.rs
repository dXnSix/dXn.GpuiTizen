//! The one display: the UI plane's size reported by the window (1920x1080 on every TV), the scale
//! factor (2.0 unless [`RunOptions::scale_factor`] or `DXN_GPUI_TIZEN_SCALE` says otherwise,
//! accepting 1.0 to 4.0) and the refresh rate measured from frame callbacks (60 Hz until then).

use std::collections::VecDeque;
use std::time::Duration;

/// The UI plane's size when the window reports none.
pub const DEFAULT_SIZE: (u32, u32) = (1920, 1080);
/// The scale factor TVs get by default (Android TV lays 1080p out at density 2).
pub const DEFAULT_SCALE: f32 = 2.0;
/// Overrides the scale factor (and `RunOptions`).
pub const SCALE_ENV: &str = "DXN_GPUI_TIZEN_SCALE";
/// Accepted scale factors.
pub const SCALE_RANGE: std::ops::RangeInclusive<f32> = 1.0..=4.0;
/// The refresh rate until one is measured.
pub const DEFAULT_REFRESH_HZ: f32 = 60.0;

/// How an app runs on Tizen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunOptions {
    /// Overrides [`DEFAULT_SCALE`]; `DXN_GPUI_TIZEN_SCALE` overrides this.
    pub scale_factor: Option<f32>,
    /// Narrows the grabbed TV keys (`crate::keygrab`), by Tizen or GPUI key name; `None` grabs
    /// the color, media and channel keys.
    pub grab_keys: Option<Vec<String>>,
    /// The font families loaded into the text system (`crate::fonts`); `None` loads the TV's
    /// UI family and its fallbacks.
    pub font_families: Option<Vec<String>>,
    /// What draws the window (`crate::renderer_select`); `DXN_GPUI_TIZEN_RENDERER` overrides it.
    pub renderer: crate::renderer_select::Renderer,
}

/// The scale factor and the overrides it rejected (for a warning in the log).
#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    pub factor: f32,
    pub rejected: Vec<String>,
}

fn accepted(value: f32) -> bool {
    value.is_finite() && SCALE_RANGE.contains(&value)
}

/// The default, then `RunOptions`, then the environment variable; each override outside 1.0-4.0
/// (or unparsable) is rejected and the previous level stays.
pub fn scale(options: &RunOptions, env: Option<&str>) -> Scale {
    let mut factor = DEFAULT_SCALE;
    let mut rejected = Vec::new();
    if let Some(value) = options.scale_factor {
        if accepted(value) {
            factor = value;
        } else {
            rejected.push(format!(
                "RunOptions::scale_factor {value} is outside 1.0-4.0"
            ));
        }
    }
    if let Some(text) = env {
        match text.trim().parse::<f32>() {
            Ok(value) if accepted(value) => factor = value,
            _ => rejected.push(format!(
                "{SCALE_ENV}={text:?} is not a scale from 1.0 to 4.0"
            )),
        }
    }
    Scale { factor, rejected }
}

/// The display's size in pixels: what the window reported, or [`DEFAULT_SIZE`].
pub fn size(reported: Option<(i32, i32)>) -> (u32, u32) {
    match reported {
        Some((w, h)) if w > 0 && h > 0 => (w as u32, h as u32),
        _ => DEFAULT_SIZE,
    }
}

/// The logical size GPUI lays out in.
pub fn logical_size(pixels: (u32, u32), scale: f32) -> (f32, f32) {
    (pixels.0 as f32 / scale, pixels.1 as f32 / scale)
}

/// The refresh rate from the measured frame-callback interval; implausible intervals (outside
/// 20-240 Hz) and none keep [`DEFAULT_REFRESH_HZ`].
pub fn refresh_hz(measured_interval: Option<Duration>) -> f32 {
    measured_interval
        .map(|interval| 1.0 / interval.as_secs_f32())
        .filter(|hz| hz.is_finite() && (20.0..=240.0).contains(hz))
        .unwrap_or(DEFAULT_REFRESH_HZ)
}

/// Frame-callback intervals the meter needs before it reports a measured rate.
pub const REFRESH_SAMPLES: usize = 30;

/// Measures the refresh rate from Ecore_Wl2 frame-callback timestamps.
///
/// Only intervals between consecutive frames count: the shell says whether the frame that
/// produced a callback was drawn straight from the previous callback (`back_to_back`); an idle
/// gap or a pause breaks the chain. The rate comes from a low percentile of the last
/// [`REFRESH_SAMPLES`] intervals, so an occasional missed vsync (a 33 ms interval at 60 Hz) does
/// not halve it. Until the window is full it reports [`DEFAULT_REFRESH_HZ`].
#[derive(Clone, Debug, Default)]
pub struct RefreshMeter {
    last: Option<Duration>,
    intervals: VecDeque<Duration>,
}

impl RefreshMeter {
    /// A frame callback arrived at `timestamp` (the compositor's time, any monotonic origin).
    pub fn frame_callback(&mut self, timestamp: Duration, back_to_back: bool) {
        if let (Some(last), true) = (self.last, back_to_back)
            && timestamp > last
        {
            if self.intervals.len() == REFRESH_SAMPLES {
                self.intervals.pop_front();
            }
            self.intervals.push_back(timestamp - last);
        }
        self.last = Some(timestamp);
    }

    /// The app was paused: the next callback starts a new chain (the samples are kept).
    pub fn pause(&mut self) {
        self.last = None;
    }

    /// The measured interval: the 20th percentile of the window, once it is full.
    pub fn interval(&self) -> Option<Duration> {
        if self.intervals.len() < REFRESH_SAMPLES {
            return None;
        }
        let mut sorted: Vec<Duration> = self.intervals.iter().copied().collect();
        sorted.sort();
        Some(sorted[sorted.len() / 5])
    }

    /// The refresh rate (see [`refresh_hz`]).
    pub fn hz(&self) -> f32 {
        refresh_hz(self.interval())
    }
}

#[cfg(test)]
mod tests;
