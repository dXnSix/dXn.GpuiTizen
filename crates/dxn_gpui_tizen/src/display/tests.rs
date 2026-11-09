use std::time::Duration;

use super::*;

#[test]
fn defaults() {
    let scale = scale(&RunOptions::default(), None);
    assert_eq!(
        scale,
        Scale {
            factor: 2.0,
            rejected: Vec::new()
        }
    );
    assert_eq!(size(None), (1920, 1080));
    assert_eq!(logical_size(size(None), scale.factor), (960.0, 540.0));
    assert_eq!(refresh_hz(None), 60.0);
}

#[test]
fn run_options_then_the_environment_override() {
    let options = RunOptions {
        scale_factor: Some(1.5),
        ..RunOptions::default()
    };
    assert_eq!(scale(&options, None).factor, 1.5);
    assert_eq!(
        scale(&options, Some("3")).factor,
        3.0,
        "the environment wins"
    );
    assert_eq!(scale(&RunOptions::default(), Some(" 1.0 ")).factor, 1.0);
    assert_eq!(scale(&RunOptions::default(), Some("4.0")).factor, 4.0);
    assert_eq!(logical_size((1920, 1080), 1.5), (1280.0, 720.0));
}

#[test]
fn values_outside_the_range_are_rejected() {
    let options = RunOptions {
        scale_factor: Some(0.5),
        ..RunOptions::default()
    };
    let scale = scale(&options, None);
    assert_eq!(scale.factor, 2.0);
    assert_eq!(scale.rejected.len(), 1);
    for bad in ["0.9", "4.01", "abc", "", "NaN", "inf", "-2"] {
        let s = super::scale(
            &RunOptions {
                scale_factor: Some(1.25),
                ..RunOptions::default()
            },
            Some(bad),
        );
        assert_eq!(s.factor, 1.25, "{bad:?} keeps RunOptions' value");
        assert!(s.rejected[0].contains(SCALE_ENV), "{bad:?}");
    }
    assert_eq!(
        super::scale(
            &RunOptions {
                scale_factor: Some(f32::NAN),
                ..RunOptions::default()
            },
            None
        )
        .factor,
        2.0
    );
}

#[test]
fn sizes_and_refresh() {
    assert_eq!(size(Some((3840, 2160))), (3840, 2160));
    assert_eq!(size(Some((0, 1080))), (1920, 1080));
    assert_eq!(size(Some((-1, -1))), (1920, 1080));
    assert!((refresh_hz(Some(Duration::from_micros(16_667))) - 60.0).abs() < 0.01);
    assert!((refresh_hz(Some(Duration::from_micros(8_333))) - 120.0).abs() < 0.1);
    assert!((refresh_hz(Some(Duration::from_millis(20))) - 50.0).abs() < 0.01);
    assert_eq!(
        refresh_hz(Some(Duration::from_secs(1))),
        60.0,
        "1 Hz is implausible"
    );
    assert_eq!(refresh_hz(Some(Duration::ZERO)), 60.0);
}

fn feed(meter: &mut RefreshMeter, start_us: u64, intervals_us: &[u64]) -> u64 {
    let mut t = start_us;
    meter.frame_callback(Duration::from_micros(t), true);
    for interval in intervals_us {
        t += interval;
        meter.frame_callback(Duration::from_micros(t), true);
    }
    t
}

#[test]
fn the_meter_reports_60_hz_until_its_window_fills() {
    let mut meter = RefreshMeter::default();
    feed(&mut meter, 0, &[16_667; REFRESH_SAMPLES - 1]);
    assert_eq!(meter.interval(), None, "warming up");
    assert_eq!(meter.hz(), 60.0);
    let mut meter = RefreshMeter::default();
    feed(&mut meter, 0, &[8_333; REFRESH_SAMPLES]);
    assert!((meter.hz() - 120.0).abs() < 0.1, "{}", meter.hz());
}

#[test]
fn steady_60_hz_and_an_occasional_missed_vsync() {
    let mut meter = RefreshMeter::default();
    let mut intervals = vec![16_667; REFRESH_SAMPLES];
    for i in (0..REFRESH_SAMPLES).step_by(6) {
        intervals[i] = 33_333;
    }
    feed(&mut meter, 1_000_000, &intervals);
    assert!(
        (meter.hz() - 60.0).abs() < 0.1,
        "a missed vsync does not halve the rate: {}",
        meter.hz()
    );
}

#[test]
fn idle_gaps_pauses_and_stale_timestamps_add_no_sample() {
    let mut meter = RefreshMeter::default();
    let t = feed(&mut meter, 0, &[16_667; REFRESH_SAMPLES - 1]);
    assert_eq!(meter.interval(), None, "one sample short");
    // The app idled for two seconds; that frame was not drawn back to back.
    meter.frame_callback(Duration::from_micros(t + 2_000_000), false);
    assert_eq!(meter.interval(), None, "an idle gap is not a sample");
    // A pause breaks the chain even when the next callback is marked back to back.
    meter.pause();
    meter.frame_callback(Duration::from_micros(t + 9_000_000), true);
    assert_eq!(
        meter.interval(),
        None,
        "the interval across a pause is not a sample"
    );
    // A timestamp that does not advance is ignored.
    meter.frame_callback(Duration::from_micros(t + 9_000_000), true);
    assert_eq!(
        meter.interval(),
        None,
        "a repeated timestamp is not a sample"
    );
    // One real interval fills the window.
    meter.frame_callback(Duration::from_micros(t + 9_016_667), true);
    assert_eq!(meter.interval(), Some(Duration::from_micros(16_667)));
}
