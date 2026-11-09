//! The video example on the desktop (through `gpui-fast-platform`), with a placeholder where the
//! video plays. On Tizen the library is the app and this binary does nothing. `--exit-after-frames <n>` quits after n frames; `RUST_LOG`
//! (`info` by default in the Lima runs) sends GPUI's log to stderr.

#[cfg(not(target_vendor = "tizen"))]
fn main() {
    struct Stderr;
    impl log::Log for Stderr {
        fn enabled(&self, m: &log::Metadata<'_>) -> bool {
            m.level() <= log::max_level()
        }
        fn log(&self, r: &log::Record<'_>) {
            if self.enabled(r.metadata()) {
                eprintln!("{} {}: {}", r.level(), r.target(), r.args());
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: Stderr = Stderr;
    if let Ok(level) = std::env::var("RUST_LOG")
        .unwrap_or_default()
        .parse::<log::LevelFilter>()
    {
        let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(level));
    }
    let mut args = std::env::args().skip(1);
    let mut exit_after = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--exit-after-frames" => exit_after = args.next().and_then(|n| n.parse::<u32>().ok()),
            other => {
                eprintln!(
                    "unknown argument {other}; usage: example_video [--exit-after-frames <n>]"
                );
                std::process::exit(2);
            }
        }
    }
    gpui_platform::application().run(move |cx| {
        example_video::launch(cx);
        if let (Some(frames), Some(window)) = (exit_after, cx.windows().first().copied()) {
            let _ = window.update(cx, |_, window, _| {
                example_video::quit_after_frames(window, frames)
            });
        }
    });
}

#[cfg(target_vendor = "tizen")]
fn main() {}
