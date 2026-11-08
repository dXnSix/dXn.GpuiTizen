//! The wgpu surface probe as a sequence of stages over a small [`Backend`] trait: instance,
//! surface, adapter, device, configure, then a fixed number of presented frames. [`Probe::step`]
//! runs one stage (one frame while presenting) per call, so the Tizen shell drives it from Ecore
//! timer ticks and the main loop never blocks for long. A failed stage is recorded with its name
//! and error text and ends the probe; it never panics and never keeps the report from being served.
//! The real wgpu backend lives in `crate::tizen`; `probe::tests` uses a fake.

use crate::report::{ProbeReport, StageOutcome, median_interval_ms};

/// Frames the probe presents.
pub const DEFAULT_FRAMES: u32 = 120;

/// The probe's stages, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Instance,
    Surface,
    Adapter,
    Device,
    Configure,
    Present,
}

impl Stage {
    pub const ALL: [Stage; 6] = [
        Stage::Instance,
        Stage::Surface,
        Stage::Adapter,
        Stage::Device,
        Stage::Configure,
        Stage::Present,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Stage::Instance => "instance",
            Stage::Surface => "surface",
            Stage::Adapter => "adapter",
            Stage::Device => "device",
            Stage::Configure => "configure",
            Stage::Present => "present",
        }
    }
}

/// What the adapter stage learns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterInfo {
    pub name: String,
    /// Whether vertex shaders may use storage buffers; without them the device is created with
    /// WebGL2 downlevel limits.
    pub vertex_storage: bool,
}

/// The operations the probe runs. Errors are text: the stage name is added by the probe.
pub trait Backend {
    fn instance(&mut self) -> Result<(), String>;
    fn surface(&mut self) -> Result<(), String>;
    fn adapter(&mut self) -> Result<AdapterInfo, String>;
    /// Creates the device, with WebGL2 downlevel limits when `webgl2_limits`.
    fn device(&mut self, webgl2_limits: bool) -> Result<(), String>;
    /// Configures a Fifo surface; returns the chosen format's name.
    fn configure(&mut self) -> Result<String, String>;
    /// Acquires, clears and presents one frame.
    fn present(&mut self) -> Result<(), String>;
}

/// The probe's state between [`Probe::step`] calls.
pub struct Probe<B, C> {
    backend: B,
    clock: C,
    frames: u32,
    next: usize,
    done: bool,
    report: ProbeReport,
    present_times: Vec<f64>,
    present_ms: f64,
}

impl<B: Backend, C: Fn() -> f64> Probe<B, C> {
    /// A probe presenting `frames` frames; `clock` returns milliseconds on any monotonic scale.
    pub fn new(backend: B, frames: u32, clock: C) -> Self {
        Self {
            backend,
            clock,
            frames,
            next: 0,
            done: false,
            report: ProbeReport::default(),
            present_times: Vec::new(),
            present_ms: 0.0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Runs the next stage (or presents the next frame). Returns `true` while work remains.
    pub fn step(&mut self) -> bool {
        if self.done {
            return false;
        }
        let stage = Stage::ALL[self.next];
        let start = (self.clock)();
        let result = match stage {
            Stage::Instance => self.backend.instance(),
            Stage::Surface => self.backend.surface(),
            Stage::Adapter => self.backend.adapter().map(|info| {
                self.report.adapter = Some(info.name);
                self.report.webgl2_limits = Some(!info.vertex_storage);
            }),
            Stage::Device => {
                let webgl2 = self.report.webgl2_limits.unwrap_or(true);
                self.backend.device(webgl2)
            }
            Stage::Configure => self.backend.configure().map(|format| {
                self.report.surface_format = Some(format);
            }),
            Stage::Present => {
                let result = self.backend.present();
                if result.is_ok() {
                    self.report.frames_presented += 1;
                    self.present_times.push((self.clock)());
                }
                result
            }
        };
        let ms = (self.clock)() - start;
        if stage == Stage::Present {
            self.present_ms += ms;
        }
        let presenting_done =
            stage == Stage::Present && self.report.frames_presented >= self.frames;
        match result {
            Err(error) => {
                let ms = if stage == Stage::Present {
                    self.present_ms
                } else {
                    ms
                };
                self.push(stage, ms, Some(format!("{} failed: {error}", stage.name())));
                self.done = true;
            }
            Ok(()) if stage == Stage::Present => {
                if presenting_done {
                    self.push(stage, self.present_ms, None);
                    self.report.ok = true;
                    self.done = true;
                }
            }
            Ok(()) => {
                self.push(stage, ms, None);
                self.next += 1;
                if Stage::ALL[self.next] == Stage::Present && self.frames == 0 {
                    self.report.ok = true;
                    self.done = true;
                }
            }
        }
        !self.done
    }

    fn push(&mut self, stage: Stage, ms: f64, error: Option<String>) {
        self.report.stages.push(StageOutcome {
            stage: stage.name().to_owned(),
            ms,
            error,
        });
    }

    /// Runs every remaining stage (for tests and blocking callers).
    pub fn run_to_end(&mut self) {
        while self.step() {}
    }

    /// The report so far.
    pub fn report(&self) -> ProbeReport {
        self.report.clone()
    }

    /// The median interval between presented frames.
    pub fn frame_interval_ms(&self) -> Option<f64> {
        median_interval_ms(&self.present_times)
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }
}

#[cfg(test)]
mod tests;
