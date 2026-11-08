use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use super::*;

const CPUINFO_ARMV7: &str = include_str!("fixtures/cpuinfo-armv7.txt");
const CPUINFO_X86_64: &str = include_str!("fixtures/cpuinfo-x86_64.txt");
const SMAPS_ROLLUP: &str = include_str!("fixtures/smaps_rollup.txt");
const MEMINFO: &str = include_str!("fixtures/meminfo.txt");

#[test]
fn cpuinfo_of_a_32_bit_arm_tv() {
    let cpu = parse_cpuinfo(CPUINFO_ARMV7, 32);
    assert_eq!(cpu.pointer_width, 32);
    assert_eq!(cpu.processors, 4);
    assert_eq!(
        cpu.model_name.as_deref(),
        Some("ARMv7 Processor rev 4 (v7l)")
    );
    assert_eq!(cpu.hardware.as_deref(), Some("Samsung NIKEM2"));
    assert_eq!(cpu.implementer.as_deref(), Some("0x41"));
    assert_eq!(cpu.part.as_deref(), Some("0xd03"));
    assert!(cpu.features.iter().any(|f| f == "neon"));
    assert!(cpu.features.iter().any(|f| f == "vfpv4"));
}

#[test]
fn cpuinfo_of_the_x86_64_emulator() {
    let cpu = parse_cpuinfo(CPUINFO_X86_64, 64);
    assert_eq!(cpu.processors, 2);
    assert_eq!(
        cpu.model_name.as_deref(),
        Some("Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz")
    );
    assert_eq!(cpu.hardware, None);
    assert!(cpu.features.iter().any(|f| f == "avx2"));
}

#[test]
fn smaps_rollup_and_meminfo() {
    let mut memory = MemoryInfo::default();
    parse_smaps_rollup(SMAPS_ROLLUP, &mut memory);
    parse_meminfo(MEMINFO, &mut memory);
    assert_eq!(memory.rss_kb, Some(48212));
    assert_eq!(memory.pss_kb, Some(31877), "Pss, not Pss_Anon");
    assert_eq!(memory.swap_kb, Some(512), "Swap, not SwapPss");
    assert_eq!(memory.total_kb, Some(1795072));
    assert_eq!(memory.available_kb, Some(350208));
    assert_eq!(memory.free_kb, Some(120344));
    let mut empty = MemoryInfo::default();
    parse_smaps_rollup("", &mut empty);
    assert_eq!(empty, MemoryInfo::default());
}

#[test]
fn extension_lists_and_flags() {
    let display = parse_extensions(
        "EGL_KHR_image_base  EGL_EXT_buffer_age EGL_KHR_partial_update\nEGL_KHR_fence_sync ",
    );
    assert_eq!(
        display,
        [
            "EGL_KHR_image_base",
            "EGL_EXT_buffer_age",
            "EGL_KHR_partial_update",
            "EGL_KHR_fence_sync"
        ]
    );
    let client = parse_extensions("EGL_EXT_platform_base EGL_ANDROID_blob_cache");
    assert_eq!(
        flag_extensions(&client, &display),
        [
            ("EGL_ANDROID_blob_cache".to_owned(), true),
            ("EGL_KHR_partial_update".to_owned(), true),
            ("EGL_EXT_buffer_age".to_owned(), true),
            ("EGL_KHR_swap_buffers_with_damage".to_owned(), false),
        ]
    );
    assert!(parse_extensions("").is_empty());
    let egl = EglInfo {
        display_extensions: display,
        ..EglInfo::default()
    }
    .flag();
    assert_eq!(egl.flagged.len(), FLAGGED_EGL_EXTENSIONS.len());
}

#[test]
fn step_timer_counts_from_process_start() {
    let now = Rc::new(Cell::new(Duration::ZERO));
    let clock = {
        let now = Rc::clone(&now);
        move || now.get()
    };
    let mut timer = StepTimer::new(Duration::from_millis(120), clock);
    assert_eq!(timer.mark("entry"), 120.0);
    now.set(Duration::from_millis(35));
    assert_eq!(timer.mark("create"), 155.0);
    now.set(Duration::from_millis(200));
    timer.mark("window");
    let names: Vec<&str> = timer.steps().iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["entry", "create", "window"]);
    assert_eq!(timer.steps()[2].ms, 320.0);
}

#[test]
fn process_age_from_proc() {
    // A command with spaces and parentheses; starttime (field 22) is 4567.
    let stat = "1234 (Dxn (host) app) S 1 1234 1234 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 9 0 4567 123456 789 4294967295";
    assert_eq!(parse_stat_starttime(stat), Some(4567));
    assert_eq!(parse_stat_starttime("garbage"), None);
    let uptime = parse_uptime("50.25 160.11\n").unwrap();
    assert_eq!(uptime, Duration::from_millis(50250));
    assert_eq!(process_age(uptime, 4567, 100), Duration::from_millis(4580));
    assert_eq!(
        process_age(Duration::from_secs(1), 500, 100),
        Duration::ZERO,
        "saturates"
    );
    assert_eq!(process_age(uptime, 1, 0), Duration::ZERO);
    assert_eq!(parse_uptime("x"), None);
}

#[test]
fn frame_interval_median() {
    assert_eq!(median_interval_ms(&[]), None);
    assert_eq!(median_interval_ms(&[10.0]), None);
    // Intervals 16, 17, 17, 50: the median ignores the one long frame.
    assert_eq!(
        median_interval_ms(&[0.0, 16.0, 33.0, 50.0, 100.0]),
        Some(17.0)
    );
    // Intervals 16, 17, 50: odd count.
    assert_eq!(median_interval_ms(&[0.0, 16.0, 33.0, 83.0]), Some(17.0));
    let vsync = median_interval_ms(&[0.0, 16.7, 33.4, 50.1]).unwrap();
    assert!((vsync - 16.7).abs() < 1e-9, "{vsync}");
}

#[test]
fn json_round_trip() {
    let mut report = Report::new();
    report.model = Some("QE55S90DAE".into());
    report.cpu = parse_cpuinfo(CPUINFO_ARMV7, 32);
    parse_meminfo(MEMINFO, &mut report.memory);
    report.egl = Some(
        EglInfo {
            vendor: Some("ARM".into()),
            version: Some("1.4 Midgard-\"r20p0\"".into()),
            display_extensions: parse_extensions("EGL_KHR_partial_update"),
            ..EglInfo::default()
        }
        .flag(),
    );
    report.gles = Some(GlesInfo {
        version: Some("OpenGL ES 3.2 v1.r20p0".into()),
        max_vertex_shader_storage_blocks: Some(0),
        max_texture_size: Some(8192),
        ..GlesInfo::default()
    });
    report.scheduling.setpriority = Some(CallResult { ret: -1, errno: 13 });
    report.probe.stages.push(StageOutcome {
        stage: "surface".into(),
        ms: 3.5,
        error: Some("EGL_BAD_MATCH".into()),
    });
    report.steps.push(Step {
        name: "entry".into(),
        ms: 412.0,
    });
    let json = report.to_json();
    assert!(json.contains("\"version\":1"), "{json}");
    assert_eq!(Report::from_json(&json).unwrap(), report);
    assert_eq!(Report::new().version, REPORT_VERSION);
}
