use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, UNIX_EPOCH};

use super::*;
use crate::report::{EglInfo, GlesInfo, StageOutcome, Step, parse_extensions};

fn server(wait: Duration) -> (Shared, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let shared = Shared::new();
    serve(listener, shared.clone(), wait).unwrap();
    (shared, addr)
}

fn sample() -> Report {
    let mut report = Report::new();
    report.model = Some("QE55S90D AE/XU".into());
    report.cpu.processors = 4;
    report.cpu.pointer_width = 32;
    report.gles = Some(GlesInfo {
        version: Some("OpenGL ES 3.2".into()),
        renderer: Some("Mali-G52".into()),
        max_vertex_shader_storage_blocks: Some(0),
        ..GlesInfo::default()
    });
    report.egl = Some(
        EglInfo {
            version: Some("1.4".into()),
            display_extensions: parse_extensions("EGL_EXT_buffer_age"),
            ..EglInfo::default()
        }
        .flag(),
    );
    report.complete = true;
    report.probe.ok = true;
    report.probe.frames_presented = 120;
    report.frame_interval_ms = Some(16.68);
    report.steps = vec![
        Step {
            name: "entry".into(),
            ms: 410.0,
        },
        Step {
            name: "first-present".into(),
            ms: 980.0,
        },
    ];
    report
}

#[test]
fn serves_the_report_then_the_log_lines() {
    let (shared, addr) = server(Duration::from_secs(5));
    shared.log("window 1920x1080");
    shared.log("two\nlines");
    let report = sample();
    shared.set_report(&report);
    let pulled = pull(addr, Duration::from_secs(5)).unwrap();
    assert_eq!(pulled.report, report);
    assert_eq!(pulled.log, ["window 1920x1080", "two lines"]);
    // One client at a time, but the server keeps serving.
    shared.log("later");
    let again = pull(addr, Duration::from_secs(5)).unwrap();
    assert_eq!(again.log.last().map(String::as_str), Some("later"));
}

#[test]
fn a_client_before_the_report_waits_for_it() {
    let (shared, addr) = server(Duration::from_secs(10));
    let client = std::thread::spawn(move || pull(addr, Duration::from_secs(10)));
    std::thread::sleep(Duration::from_millis(200));
    let mut report = sample();
    report.model = Some("late".into());
    shared.set_report(&report);
    let pulled = client.join().unwrap().unwrap();
    assert_eq!(pulled.report.model.as_deref(), Some("late"));
}

#[test]
fn no_report_in_time_closes_without_one() {
    let (_shared, addr) = server(Duration::from_millis(100));
    let err = pull(addr, Duration::from_secs(5)).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof, "{err}");
}

#[test]
fn a_dropped_connection_does_not_stop_the_server() {
    let (shared, addr) = server(Duration::from_secs(5));
    // A client that leaves before the report exists, and one that leaves without reading.
    drop(TcpStream::connect(addr).unwrap());
    let mut big = sample();
    big.errors = vec!["x".repeat(4096); 64];
    shared.set_report(&big);
    drop(TcpStream::connect(addr).unwrap());
    let mut partial = TcpStream::connect(addr).unwrap();
    let mut first = [0u8; 16];
    partial.read_exact(&mut first).unwrap();
    drop(partial);
    let pulled = pull(addr, Duration::from_secs(5)).unwrap();
    assert_eq!(pulled.report.errors.len(), 64);
}

#[test]
fn port_from_the_launch_extra() {
    assert_eq!(port_from_extra(None), DEFAULT_PORT);
    assert_eq!(port_from_extra(Some("47200")), 47200);
    assert_eq!(port_from_extra(Some(" 8080 ")), 8080);
    assert_eq!(port_from_extra(Some("0")), DEFAULT_PORT);
    assert_eq!(port_from_extra(Some("abc")), DEFAULT_PORT);
    assert_eq!(DEFAULT_PORT, 47123);
}

#[test]
fn file_name_and_summary() {
    let time = UNIX_EPOCH + Duration::from_secs(1_793_916_311); // 2026-11-05T22:05:11Z
    assert_eq!(utc_timestamp(time), "20261105T220511Z");
    assert_eq!(utc_timestamp(UNIX_EPOCH), "19700101T000000Z");
    assert_eq!(
        utc_timestamp(UNIX_EPOCH + Duration::from_secs(951_782_400)),
        "20000229T000000Z"
    );
    assert_eq!(
        report_file_name(&sample(), time),
        "20261105T220511Z-QE55S90D-AE-XU.json"
    );
    assert_eq!(
        report_file_name(&Report::new(), time),
        "20261105T220511Z-unknown.json"
    );

    let text = summary(&sample());
    for expected in [
        "OpenGL ES 3.2",
        "vertex storage blocks 0",
        "+ EGL_EXT_buffer_age",
        "- EGL_ANDROID_blob_cache",
        "probe  ok  frames 120  interval 16.68 ms",
        "entry 410 > first-present 980",
    ] {
        assert!(text.contains(expected), "{expected} missing from\n{text}");
    }
    let mut failed = sample();
    failed.probe.ok = false;
    failed.probe.stages = vec![StageOutcome {
        stage: "surface".into(),
        ms: 1.0,
        error: Some("surface failed: EGL_BAD_MATCH".into()),
    }];
    assert!(summary(&failed).contains("probe  surface failed: EGL_BAD_MATCH"));
    assert!(text.lines().count() <= 24, "one screen");
}

#[test]
fn a_partial_report_waits_for_the_complete_one() {
    let (shared, addr) = server(Duration::from_secs(10));
    let mut partial = sample();
    partial.complete = false;
    partial.probe = Default::default();
    shared.set_report(&partial);
    let client = std::thread::spawn(move || pull(addr, Duration::from_secs(10)));
    std::thread::sleep(Duration::from_millis(200));
    shared.set_report(&sample());
    let pulled = client.join().unwrap().unwrap();
    assert!(pulled.report.complete);
    assert!(pulled.report.probe.ok);
}

#[test]
fn a_timeout_serves_the_latest_partial_report_marked_running() {
    let (shared, addr) = server(Duration::from_millis(150));
    let mut partial = sample();
    partial.complete = false;
    partial.probe = Default::default();
    shared.set_report(&partial);
    let pulled = pull(addr, Duration::from_secs(5)).unwrap();
    assert!(!pulled.report.complete);
    let text = summary(&pulled.report);
    assert!(text.contains("probe  running"), "{text}");
}

#[test]
fn sdb_forward_arguments() {
    assert_eq!(
        sdb_forward_args(47123),
        ["forward", "tcp:47123", "tcp:47123"]
    );
}
