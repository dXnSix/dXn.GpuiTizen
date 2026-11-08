//! The diagnostic report: a versioned JSON document about one TV, its collectors' parsers, and the
//! launch-step timer. Everything here is platform-neutral and host-tested (`report::tests`); the
//! EGL / GL ES queries that fill [`EglInfo`] and [`GlesInfo`] live in `crate::tizen` (run-time
//! loading, never linking).

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Bumped whenever a field changes meaning or is removed.
pub const REPORT_VERSION: u32 = 1;

/// The EGL extensions the renderer plan depends on; the report flags each one.
pub const FLAGGED_EGL_EXTENSIONS: &[&str] = &[
    "EGL_ANDROID_blob_cache",
    "EGL_KHR_partial_update",
    "EGL_EXT_buffer_age",
    "EGL_KHR_swap_buffers_with_damage",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub version: u32,
    /// Whether every collector and the probe have finished; a report served before that (only
    /// when the server's wait times out) says so.
    pub complete: bool,
    /// The device model (`http://tizen.org/system/model_name`), when known.
    pub model: Option<String>,
    /// The platform version (`http://tizen.org/feature/platform.version`), when known.
    pub platform_version: Option<String>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub egl: Option<EglInfo>,
    pub gles: Option<GlesInfo>,
    /// The measured interval between presented frames, in milliseconds (median).
    pub frame_interval_ms: Option<f64>,
    pub scheduling: SchedulingInfo,
    pub probe: ProbeReport,
    /// Launch steps, in milliseconds since process start.
    pub steps: Vec<Step>,
    /// Errors of collectors that could not run.
    pub errors: Vec<String>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            version: REPORT_VERSION,
            ..Self::default()
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CpuInfo {
    /// `usize` width of this build: 32 on the armv7 TV build.
    pub pointer_width: u32,
    pub processors: u32,
    pub model_name: Option<String>,
    pub hardware: Option<String>,
    pub implementer: Option<String>,
    pub part: Option<String>,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// `/proc/self/smaps_rollup`, in KiB.
    pub rss_kb: Option<u64>,
    pub pss_kb: Option<u64>,
    pub swap_kb: Option<u64>,
    /// `/proc/meminfo`, in KiB.
    pub total_kb: Option<u64>,
    pub available_kb: Option<u64>,
    pub free_kb: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EglInfo {
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub client_extensions: Vec<String>,
    pub display_extensions: Vec<String>,
    /// Each of [`FLAGGED_EGL_EXTENSIONS`] and whether the display (or client) list has it.
    pub flagged: Vec<(String, bool)>,
}

impl EglInfo {
    /// Fills [`EglInfo::flagged`] from the extension lists.
    pub fn flag(mut self) -> Self {
        self.flagged = flag_extensions(&self.client_extensions, &self.display_extensions);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GlesInfo {
    pub version: Option<String>,
    pub renderer: Option<String>,
    pub vendor: Option<String>,
    pub shading_language_version: Option<String>,
    pub extensions: Vec<String>,
    pub max_vertex_shader_storage_blocks: Option<i64>,
    pub max_texture_size: Option<i64>,
}

/// What raising the main thread's priority returned.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SchedulingInfo {
    /// `setpriority(PRIO_PROCESS, 0, -10)`: return value and `errno`.
    pub setpriority: Option<CallResult>,
    /// `sched_setattr(0, SCHED_FIFO 1)` on the main thread: return value and `errno`.
    pub sched_setattr: Option<CallResult>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallResult {
    pub ret: i64,
    pub errno: i32,
}

/// The wgpu surface probe: one entry per stage, in order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProbeReport {
    pub stages: Vec<StageOutcome>,
    /// Whether every stage succeeded.
    pub ok: bool,
    pub frames_presented: u32,
    pub adapter: Option<String>,
    /// Whether the device was created with WebGL2 downlevel limits (no vertex storage buffers).
    pub webgl2_limits: Option<bool>,
    pub surface_format: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StageOutcome {
    pub stage: String,
    pub ms: f64,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    /// Milliseconds since process start.
    pub ms: f64,
}

/// Splits a space-separated extension string (`EGL_EXTENSIONS`, `GL_EXTENSIONS`).
pub fn parse_extensions(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

/// Flags [`FLAGGED_EGL_EXTENSIONS`] present in either list.
pub fn flag_extensions(client: &[String], display: &[String]) -> Vec<(String, bool)> {
    FLAGGED_EGL_EXTENSIONS
        .iter()
        .map(|name| {
            let present = client.iter().chain(display).any(|e| e == name);
            ((*name).to_owned(), present)
        })
        .collect()
}

fn key_value(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    Some((key.trim(), value.trim()))
}

/// Parses `/proc/cpuinfo` (ARM and x86 layouts).
pub fn parse_cpuinfo(text: &str, pointer_width: u32) -> CpuInfo {
    let mut info = CpuInfo {
        pointer_width,
        ..CpuInfo::default()
    };
    for (key, value) in text.lines().filter_map(key_value) {
        match key {
            "processor" => info.processors += 1,
            "model name" | "Processor" if info.model_name.is_none() => {
                info.model_name = Some(value.to_owned());
            }
            "Hardware" => info.hardware = Some(value.to_owned()),
            "CPU implementer" if info.implementer.is_none() => {
                info.implementer = Some(value.to_owned());
            }
            "CPU part" if info.part.is_none() => info.part = Some(value.to_owned()),
            "Features" | "flags" if info.features.is_empty() => {
                info.features = parse_extensions(value);
            }
            _ => {}
        }
    }
    info
}

/// The value of a `Key:   123 kB` line.
fn kb(value: &str) -> Option<u64> {
    value.split_whitespace().next()?.parse().ok()
}

/// Parses `/proc/self/smaps_rollup` into `memory` (RSS, PSS, swap).
pub fn parse_smaps_rollup(text: &str, memory: &mut MemoryInfo) {
    for (key, value) in text.lines().filter_map(key_value) {
        match key {
            "Rss" => memory.rss_kb = kb(value),
            "Pss" => memory.pss_kb = kb(value),
            "Swap" => memory.swap_kb = kb(value),
            _ => {}
        }
    }
}

/// Parses `/proc/meminfo` into `memory` (total, available, free).
pub fn parse_meminfo(text: &str, memory: &mut MemoryInfo) {
    for (key, value) in text.lines().filter_map(key_value) {
        match key {
            "MemTotal" => memory.total_kb = kb(value),
            "MemAvailable" => memory.available_kb = kb(value),
            "MemFree" => memory.free_kb = kb(value),
            _ => {}
        }
    }
}

/// The process's start time from `/proc/self/stat` (field 22, clock ticks since boot). The
/// command field may contain spaces and parentheses, so fields are counted after the last `)`.
pub fn parse_stat_starttime(stat: &str) -> Option<u64> {
    let rest = &stat[stat.rfind(')')? + 1..];
    // After `)` come fields 3 (state) onwards; starttime is field 22.
    rest.split_whitespace().nth(22 - 3)?.parse().ok()
}

/// The time since boot from `/proc/uptime`'s first field.
pub fn parse_uptime(text: &str) -> Option<Duration> {
    let seconds: f64 = text.split_whitespace().next()?.parse().ok()?;
    (seconds.is_finite() && seconds >= 0.0).then(|| Duration::from_secs_f64(seconds))
}

/// How long the process has been running: uptime minus its start time.
pub fn process_age(uptime: Duration, starttime_ticks: u64, ticks_per_second: u64) -> Duration {
    if ticks_per_second == 0 {
        return Duration::ZERO;
    }
    let started = Duration::from_secs_f64(starttime_ticks as f64 / ticks_per_second as f64);
    uptime.saturating_sub(started)
}

/// Records launch steps in milliseconds since process start. The clock is injected (a function
/// returning the elapsed time since the timer was created) so tests can drive it; `offset` is the
/// process's age when the timer was created.
pub struct StepTimer<C: Fn() -> Duration> {
    offset: Duration,
    clock: C,
    steps: Vec<Step>,
}

impl<C: Fn() -> Duration> StepTimer<C> {
    pub fn new(offset: Duration, clock: C) -> Self {
        Self {
            offset,
            clock,
            steps: Vec::new(),
        }
    }

    /// Milliseconds since process start, now.
    pub fn now_ms(&self) -> f64 {
        (self.offset + (self.clock)()).as_secs_f64() * 1000.0
    }

    /// Records `name` at the current time and returns its time.
    pub fn mark(&mut self, name: &str) -> f64 {
        let ms = self.now_ms();
        self.steps.push(Step {
            name: name.to_owned(),
            ms,
        });
        ms
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}

/// A real-time step timer: [`std::time::Instant`] since creation.
pub fn instant_timer(offset: Duration) -> StepTimer<impl Fn() -> Duration> {
    let start = std::time::Instant::now();
    StepTimer::new(offset, move || start.elapsed())
}

/// The median of frame intervals (milliseconds) between consecutive present times.
pub fn median_interval_ms(present_times_ms: &[f64]) -> Option<f64> {
    let mut intervals: Vec<f64> = present_times_ms.windows(2).map(|w| w[1] - w[0]).collect();
    if intervals.is_empty() {
        return None;
    }
    intervals.sort_by(f64::total_cmp);
    let mid = intervals.len() / 2;
    Some(if intervals.len() % 2 == 0 {
        (intervals[mid - 1] + intervals[mid]) / 2.0
    } else {
        intervals[mid]
    })
}

#[cfg(test)]
mod tests;
