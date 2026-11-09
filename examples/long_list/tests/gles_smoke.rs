//! The GL ES 3.0 path on desktop Linux (`SHELL-4`): the desktop binary under Xvfb with
//! `WGPU_BACKEND=gl`, `MESA_GLES_VERSION_OVERRIDE=3.0` and no storage buffers
//! (`MESA_EXTENSION_OVERRIDE`, all set by `scripts/lima-run.sh`) and
//! the local gpui-fast patch draws 60 frames, exits 0, and logs that instance data goes through
//! textures (gpui-fast change 1).
#![cfg(target_os = "linux")]

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// gpui-fast's change-1 log line (`fast/gles.rs`).
const INSTANCE_TEXTURES: &str = "instance textures and WebGL2 limits";

#[test]
fn draws_through_instance_textures_without_vertex_storage() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_example_long_list"))
        .args(["--exit-after-frames", "60"])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the desktop binary starts");
    // Read both pipes while the binary runs, so a full pipe never blocks it.
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let out = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        text
    });
    let err = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });
    let deadline = Instant::now() + Duration::from_secs(120);
    let status = loop {
        if let Some(status) = child.try_wait().expect("waiting for the binary") {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("the binary did not exit within 120 s");
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let output = out.join().unwrap_or_default() + &err.join().unwrap_or_default();
    assert!(status.success(), "exit status {status}; output:\n{output}");
    assert!(
        output.contains(INSTANCE_TEXTURES),
        "the change-1 log line is missing (WGPU_BACKEND={:?}, MESA_EXTENSION_OVERRIDE={:?}); output:\n{output}",
        std::env::var("WGPU_BACKEND").ok(),
        std::env::var("MESA_EXTENSION_OVERRIDE").ok(),
    );
}
