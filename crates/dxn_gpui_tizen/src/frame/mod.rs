//! The frame scheduler: when the window may ask GPUI to draw.
//!
//! A frame is drawn only when one was asked for (GPUI invalidated the window) and the
//! compositor's frame callback for the previous frame has arrived, or none is pending. The first
//! frame after the window is created, and the first after a resume, are forced. Requests made
//! while a frame callback is pending coalesce into one frame. Nothing is drawn while paused. The
//! scheduler only returns what to do; the Tizen shell does it.

/// What the shell does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEffect {
    /// Ask GPUI to draw now (`force_render` for a forced frame), present, then ask the
    /// compositor for a frame callback (`ecore_wl2_window_frame_callback_add`).
    Draw {
        force_render: bool,
        /// The pause epoch the draw belongs to: a draw posted before a pause is dropped.
        epoch: u32,
    },
}

#[derive(Clone, Debug)]
pub struct FrameScheduler {
    requested: bool,
    forced: bool,
    callback_pending: bool,
    paused: bool,
    frames: u64,
    epoch: u32,
    failures: u32,
}

impl Default for FrameScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameScheduler {
    /// A scheduler for a new window: its first frame is forced.
    pub fn new() -> Self {
        Self {
            requested: false,
            forced: true,
            callback_pending: false,
            paused: false,
            frames: 0,
            epoch: 0,
            failures: 0,
        }
    }

    /// Frames drawn so far.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Whether the compositor's callback for the last frame is still outstanding.
    pub fn callback_pending(&self) -> bool {
        self.callback_pending
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// GPUI wants a frame.
    pub fn request(&mut self) -> Option<FrameEffect> {
        self.requested = true;
        self.next()
    }

    /// The compositor's frame callback for the previous frame arrived.
    pub fn frame_callback(&mut self) -> Option<FrameEffect> {
        self.callback_pending = false;
        self.next()
    }

    /// The window is ready to draw (shown, or its renderer was rebuilt): try the forced frame.
    pub fn ready(&mut self) -> Option<FrameEffect> {
        self.next()
    }

    /// The frame [`FrameEffect::Draw`] asked for was not presented, so no frame callback will
    /// come for it. `retry` when GPUI drew but the renderer could not present: the frame is drawn
    /// again, forced, because GPUI already cleared its dirty state. Otherwise GPUI found nothing to
    /// draw. Returns the next frame: the retry, or a request GPUI made meanwhile.
    pub fn not_presented(&mut self, retry: bool) -> Option<FrameEffect> {
        self.callback_pending = false;
        if retry {
            self.forced = true;
            self.failures = self.failures.saturating_add(1);
        }
        self.next()
    }

    /// A frame was presented: failures stop counting.
    pub fn presented(&mut self) {
        self.failures = 0;
    }

    /// The renderer needs the next frame rendered in full (it lost or cleared state): force one.
    pub fn force(&mut self) -> Option<FrameEffect> {
        self.forced = true;
        self.next()
    }

    /// How long to wait before the next frame after consecutive failed presents: none for the
    /// first few, then 100 ms, so a persistent failure does not spin the main loop.
    pub fn retry_delay(&self) -> std::time::Duration {
        if self.failures <= 3 {
            std::time::Duration::ZERO
        } else {
            std::time::Duration::from_millis(100)
        }
    }

    /// A draw that was posted to the main loop is about to run: it is kept unless the app paused
    /// since it was decided.
    pub fn posted(&mut self, effect: Option<FrameEffect>) -> Option<FrameEffect> {
        match effect {
            Some(FrameEffect::Draw { epoch, .. }) if epoch != self.epoch || self.paused => None,
            other => other,
        }
    }

    /// The app was paused: no frames until resume. A pending callback may never come.
    pub fn pause(&mut self) {
        self.paused = true;
        self.callback_pending = false;
        self.epoch = self.epoch.wrapping_add(1);
    }

    /// The app was resumed: the next frame is forced.
    pub fn resume(&mut self) -> Option<FrameEffect> {
        self.paused = false;
        self.forced = true;
        self.next()
    }

    fn next(&mut self) -> Option<FrameEffect> {
        if self.paused || self.callback_pending || !(self.requested || self.forced) {
            return None;
        }
        let force_render = self.forced;
        self.requested = false;
        self.forced = false;
        self.callback_pending = true;
        self.frames += 1;
        Some(FrameEffect::Draw {
            force_render,
            epoch: self.epoch,
        })
    }
}

#[cfg(test)]
mod tests;
