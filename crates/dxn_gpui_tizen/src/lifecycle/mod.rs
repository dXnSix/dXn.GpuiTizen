//! The app lifecycle: `ui_app_main`'s callbacks (create, resume, pause, terminate, app_control)
//! become effects for the shell. Pause stops frames and releases GPU resources, resume rebuilds
//! them and forces a full redraw, terminate runs GPUI's quit callbacks once. Repeated or
//! out-of-order callbacks produce nothing. The launch request's extras are kept as a string map.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Before `create`.
    Launching,
    Created,
    Resumed,
    Paused,
    Terminated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Open the window and start GPUI (`create`).
    Start,
    /// Stop asking for frames.
    StopFrames,
    /// Free the surface, surface-bound resources and atlas textures (the shell unconfigures the
    /// renderer's surface and clears its atlas; design `d36-pause-unconfigure`).
    ReleaseGpu,
    /// Rebuild the renderer on the same window handles.
    RebuildGpu,
    /// Redraw everything (`force_render`).
    ForceRedraw,
    /// Run GPUI's quit callbacks.
    Quit,
    /// A launch request arrived; its extras are in [`Lifecycle::extras`].
    AppControl,
}

#[derive(Clone, Debug)]
pub struct Lifecycle {
    state: State,
    extras: BTreeMap<String, String>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    pub fn new() -> Self {
        Self {
            state: State::Launching,
            extras: BTreeMap::new(),
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// The extras of the latest launch request.
    pub fn extras(&self) -> &BTreeMap<String, String> {
        &self.extras
    }

    pub fn create(&mut self) -> Vec<Effect> {
        match self.state {
            State::Launching => {
                self.state = State::Created;
                vec![Effect::Start]
            }
            _ => Vec::new(),
        }
    }

    /// The first resume after create only records the state: the window's first frame is forced
    /// anyway. A resume after a pause rebuilds the GPU resources and redraws everything.
    pub fn resume(&mut self) -> Vec<Effect> {
        match self.state {
            State::Created => {
                self.state = State::Resumed;
                Vec::new()
            }
            State::Paused => {
                self.state = State::Resumed;
                vec![Effect::RebuildGpu, Effect::ForceRedraw]
            }
            _ => Vec::new(),
        }
    }

    pub fn pause(&mut self) -> Vec<Effect> {
        match self.state {
            State::Created | State::Resumed => {
                self.state = State::Paused;
                vec![Effect::StopFrames, Effect::ReleaseGpu]
            }
            _ => Vec::new(),
        }
    }

    pub fn terminate(&mut self) -> Vec<Effect> {
        match self.state {
            State::Terminated => Vec::new(),
            State::Launching => {
                self.state = State::Terminated;
                Vec::new()
            }
            _ => {
                self.state = State::Terminated;
                vec![Effect::StopFrames, Effect::Quit]
            }
        }
    }

    /// A launch request with its extras (the latest request replaces earlier extras).
    pub fn app_control(
        &mut self,
        extras: impl IntoIterator<Item = (String, String)>,
    ) -> Vec<Effect> {
        if self.state == State::Terminated {
            return Vec::new();
        }
        self.extras = extras.into_iter().collect();
        vec![Effect::AppControl]
    }
}

#[cfg(test)]
mod tests;
