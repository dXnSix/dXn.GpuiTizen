//! Pause and resume as GPUI sees them: the window's activation (`on_active_status_change`,
//! `is_active`) and visibility (`on_visibility_change`, `visibility`) follow the app lifecycle.
//! A paused app is inactive and hidden, a resumed one active and visible; a launched app starts
//! active and visible.
//!
//! The reports are placed among the lifecycle's effects ([`WindowStatus::steps`]):
//!
//! - Pause: frames stop first, then GPUI hears the window is inactive and hidden, so nothing an
//!   observer does draws while the surface goes away; the GPU resources are released after.
//! - Resume: the GPU resources are rebuilt, GPUI hears the window is visible and active, then
//!   the full redraw runs, so the first frame after resume already shows the active window.
//! - Terminate from a shown app: frames stop, the window turns inactive and hidden, then GPUI's
//!   quit callbacks run. A paused app has nothing left to report.
//!
//! The status changes only when a report is made, and each change is reported once.

use crate::lifecycle::Effect;

/// A change GPUI hears of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// `on_active_status_change(active)`; `is_active` returns it from now on.
    Active(bool),
    /// `on_visibility_change`: `true` for `WindowVisibility::Visible`, `false` for `Hidden`;
    /// `visibility` returns it from now on.
    Visible(bool),
}

/// What the shell performs, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// A lifecycle effect.
    Lifecycle(Effect),
    /// A report to the window's callbacks (after its status changed).
    Report(Report),
}

/// The window's activation and visibility as GPUI was last told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowStatus {
    active: bool,
    visible: bool,
}

impl Default for WindowStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowStatus {
    /// A launched app: active and visible.
    pub const fn new() -> Self {
        Self {
            active: true,
            visible: true,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The lifecycle's `effects` with the reports they imply placed among them: the reports for a
    /// hidden window right after [`Effect::StopFrames`], those for a shown one right before
    /// [`Effect::ForceRedraw`].
    pub fn steps(&mut self, effects: Vec<Effect>) -> Vec<Step> {
        let mut steps = Vec::with_capacity(effects.len() + 2);
        for effect in effects {
            match effect {
                Effect::StopFrames => {
                    steps.push(Step::Lifecycle(effect));
                    self.hide(&mut steps);
                }
                Effect::ForceRedraw => {
                    self.show(&mut steps);
                    steps.push(Step::Lifecycle(effect));
                }
                _ => steps.push(Step::Lifecycle(effect)),
            }
        }
        steps
    }

    fn hide(&mut self, steps: &mut Vec<Step>) {
        if self.active {
            self.active = false;
            steps.push(Step::Report(Report::Active(false)));
        }
        if self.visible {
            self.visible = false;
            steps.push(Step::Report(Report::Visible(false)));
        }
    }

    fn show(&mut self, steps: &mut Vec<Step>) {
        if !self.visible {
            self.visible = true;
            steps.push(Step::Report(Report::Visible(true)));
        }
        if !self.active {
            self.active = true;
            steps.push(Step::Report(Report::Active(true)));
        }
    }
}

#[cfg(test)]
mod tests;
