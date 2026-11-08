use super::*;

#[test]
fn launch_pause_resume_terminate() {
    let mut app = Lifecycle::new();
    assert_eq!(app.state(), State::Launching);
    assert_eq!(app.create(), [Effect::Start]);
    assert_eq!(app.resume(), [], "the launch resume only records the state");
    assert_eq!(app.state(), State::Resumed);
    assert_eq!(app.pause(), [Effect::StopFrames, Effect::ReleaseGpu]);
    assert_eq!(app.state(), State::Paused);
    assert_eq!(app.resume(), [Effect::RebuildGpu, Effect::ForceRedraw]);
    assert_eq!(app.terminate(), [Effect::StopFrames, Effect::Quit]);
    assert_eq!(app.state(), State::Terminated);
}

#[test]
fn repeated_and_out_of_order_callbacks_produce_nothing() {
    let mut app = Lifecycle::new();
    assert_eq!(app.resume(), [], "resume before create");
    assert_eq!(app.pause(), [], "pause before create");
    app.create();
    assert_eq!(app.create(), [], "a second create");
    app.resume();
    assert_eq!(app.resume(), [], "resume while resumed");
    app.pause();
    assert_eq!(app.pause(), [], "a second pause");
    app.terminate();
    assert_eq!(
        app.terminate(),
        [],
        "a second terminate runs no quit callbacks"
    );
    assert_eq!(app.resume(), []);
    assert_eq!(app.pause(), []);
    assert_eq!(app.app_control([("k".to_owned(), "v".to_owned())]), []);
}

#[test]
fn pause_straight_after_create_releases_gpu() {
    let mut app = Lifecycle::new();
    app.create();
    assert_eq!(app.pause(), [Effect::StopFrames, Effect::ReleaseGpu]);
    assert_eq!(app.resume(), [Effect::RebuildGpu, Effect::ForceRedraw]);
}

#[test]
fn terminate_from_paused_quits_once() {
    let mut app = Lifecycle::new();
    app.create();
    app.pause();
    assert_eq!(app.terminate(), [Effect::StopFrames, Effect::Quit]);
    let mut never_created = Lifecycle::new();
    assert_eq!(
        never_created.terminate(),
        [],
        "nothing started, nothing to quit"
    );
}

#[test]
fn app_control_extras_are_a_string_map() {
    let mut app = Lifecycle::new();
    app.create();
    let effects = app.app_control([
        ("port".to_owned(), "47200".to_owned()),
        ("deeplink".to_owned(), "nsh://home".to_owned()),
    ]);
    assert_eq!(effects, [Effect::AppControl]);
    assert_eq!(app.extras().get("port").map(String::as_str), Some("47200"));
    assert_eq!(app.extras().len(), 2);
    app.app_control([("deeplink".to_owned(), "nsh://settings".to_owned())]);
    assert_eq!(
        app.extras().get("port"),
        None,
        "the latest request replaces the extras"
    );
    assert_eq!(
        app.extras().get("deeplink").map(String::as_str),
        Some("nsh://settings")
    );
}
