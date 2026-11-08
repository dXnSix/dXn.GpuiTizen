//! The counter on the desktop (through `gpui-fast-platform`). On Tizen the library is the app
//! and this binary does nothing. `--exit-after-frames <n>` quits after n frames.

#[cfg(not(target_vendor = "tizen"))]
fn main() {
    let mut args = std::env::args().skip(1);
    let mut exit_after = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--exit-after-frames" => exit_after = args.next().and_then(|n| n.parse::<u32>().ok()),
            other => {
                eprintln!(
                    "unknown argument {other}; usage: example_counter [--exit-after-frames <n>]"
                );
                std::process::exit(2);
            }
        }
    }
    gpui_platform::application().run(move |cx| {
        example_counter::launch(cx);
        if let (Some(frames), Some(window)) = (exit_after, cx.windows().first().copied()) {
            let _ = window.update(cx, |_, window, _| {
                example_counter::quit_after_frames(window, frames)
            });
        }
    });
}

#[cfg(target_vendor = "tizen")]
fn main() {}
