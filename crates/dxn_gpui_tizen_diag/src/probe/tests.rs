use std::cell::Cell;
use std::rc::Rc;

use super::*;

/// A backend that fails at one stage (or none) and counts calls.
#[derive(Default)]
struct Fake {
    fail_at: Option<Stage>,
    /// Fail on this present (1-based), when `fail_at` is `Present`.
    fail_on_frame: u32,
    vertex_storage: bool,
    presents: u32,
    device_webgl2: Option<bool>,
    calls: Vec<&'static str>,
}

impl Fake {
    fn result(&mut self, stage: Stage) -> Result<(), String> {
        self.calls.push(stage.name());
        if self.fail_at == Some(stage)
            && (stage != Stage::Present || self.presents == self.fail_on_frame)
        {
            Err(format!("fake {} error", stage.name()))
        } else {
            Ok(())
        }
    }
}

impl Backend for Fake {
    fn instance(&mut self) -> Result<(), String> {
        self.result(Stage::Instance)
    }
    fn surface(&mut self) -> Result<(), String> {
        self.result(Stage::Surface)
    }
    fn adapter(&mut self) -> Result<AdapterInfo, String> {
        self.result(Stage::Adapter)?;
        Ok(AdapterInfo {
            name: "Mali-G52 (GL)".into(),
            vertex_storage: self.vertex_storage,
        })
    }
    fn device(&mut self, webgl2_limits: bool) -> Result<(), String> {
        self.device_webgl2 = Some(webgl2_limits);
        self.result(Stage::Device)
    }
    fn configure(&mut self) -> Result<String, String> {
        self.result(Stage::Configure)?;
        Ok("Rgba8UnormSrgb".into())
    }
    fn present(&mut self) -> Result<(), String> {
        self.presents += 1;
        self.result(Stage::Present)
    }
}

/// A clock that advances 2 ms per reading.
fn ticking_clock() -> impl Fn() -> f64 {
    let now = Rc::new(Cell::new(0.0));
    move || {
        let t = now.get();
        now.set(t + 2.0);
        t
    }
}

#[test]
fn success_presents_every_frame_and_reports_each_stage() {
    let mut probe = Probe::new(
        Fake {
            vertex_storage: false,
            ..Fake::default()
        },
        5,
        ticking_clock(),
    );
    let mut steps = 0;
    while probe.step() {
        steps += 1;
    }
    assert_eq!(steps, 5 + 5 - 1, "one call per stage and per frame");
    assert!(probe.is_done());
    let report = probe.report();
    assert!(report.ok);
    assert_eq!(report.frames_presented, 5);
    let stages: Vec<&str> = report.stages.iter().map(|s| s.stage.as_str()).collect();
    assert_eq!(
        stages,
        [
            "instance",
            "surface",
            "adapter",
            "device",
            "configure",
            "present"
        ]
    );
    assert!(
        report
            .stages
            .iter()
            .all(|s| s.error.is_none() && s.ms > 0.0)
    );
    assert_eq!(report.adapter.as_deref(), Some("Mali-G52 (GL)"));
    assert_eq!(
        report.webgl2_limits,
        Some(true),
        "no vertex storage: WebGL2 limits"
    );
    assert_eq!(probe.backend().device_webgl2, Some(true));
    assert_eq!(report.surface_format.as_deref(), Some("Rgba8UnormSrgb"));
    assert!(probe.frame_interval_ms().is_some());
    assert!(!probe.step(), "a finished probe does nothing");
}

#[test]
fn vertex_storage_keeps_downlevel_limits() {
    let mut probe = Probe::new(
        Fake {
            vertex_storage: true,
            ..Fake::default()
        },
        1,
        ticking_clock(),
    );
    probe.run_to_end();
    assert_eq!(probe.report().webgl2_limits, Some(false));
    assert_eq!(probe.backend().device_webgl2, Some(false));
}

#[test]
fn a_failure_at_each_stage_stops_the_probe_and_names_the_stage() {
    for (index, &stage) in Stage::ALL.iter().enumerate() {
        let fake = Fake {
            fail_at: Some(stage),
            fail_on_frame: 3,
            ..Fake::default()
        };
        let mut probe = Probe::new(fake, 10, ticking_clock());
        probe.run_to_end();
        let report = probe.report();
        assert!(!report.ok, "{stage:?}");
        assert_eq!(
            report.stages.len(),
            index + 1,
            "{stage:?}: stages up to the failed one"
        );
        let last = report.stages.last().unwrap();
        assert_eq!(last.stage, stage.name());
        let error = last.error.as_deref().unwrap();
        assert!(
            error.starts_with(&format!("{} failed:", stage.name())),
            "{error}"
        );
        assert!(error.contains("fake"), "{error}");
        assert!(report.stages[..index].iter().all(|s| s.error.is_none()));
        // Nothing runs after the failed stage.
        let calls = &probe.backend().calls;
        assert_eq!(calls.last().copied(), Some(stage.name()), "{calls:?}");
        if stage == Stage::Present {
            assert_eq!(
                report.frames_presented, 2,
                "frames before the failed one count"
            );
        } else {
            assert_eq!(report.frames_presented, 0);
        }
    }
}

#[test]
fn the_report_survives_serialization_in_both_cases() {
    for fail_at in [None, Some(Stage::Surface)] {
        let mut probe = Probe::new(
            Fake {
                fail_at,
                ..Fake::default()
            },
            2,
            ticking_clock(),
        );
        probe.run_to_end();
        let mut report = crate::report::Report::new();
        report.probe = probe.report();
        report.frame_interval_ms = probe.frame_interval_ms();
        let back = crate::report::Report::from_json(&report.to_json()).unwrap();
        assert_eq!(back.probe, report.probe);
        assert_eq!(back.probe.ok, fail_at.is_none());
    }
}

#[test]
fn zero_frames_ends_after_configure() {
    let mut probe = Probe::new(Fake::default(), 0, ticking_clock());
    probe.run_to_end();
    let report = probe.report();
    assert!(report.ok);
    assert_eq!(report.stages.len(), 5);
    assert_eq!(probe.backend().presents, 0);
}
