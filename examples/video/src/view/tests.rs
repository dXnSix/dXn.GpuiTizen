use gpui::TestAppContext;

use super::*;

#[test]
fn focus_moves_along_the_row_and_stops_at_its_ends() {
    let mut controls = Controls::default();
    assert_eq!(controls.focused_control(), Control::PlayPause);
    controls.previous();
    assert_eq!(controls.focused_control(), Control::Rewind);
    controls.previous();
    assert_eq!(
        controls.focused_control(),
        Control::Rewind,
        "stops at the first"
    );
    for _ in 0..5 {
        controls.next();
    }
    assert_eq!(
        controls.focused_control(),
        Control::Mute,
        "stops at the last"
    );
}

#[test]
fn controls_send_their_commands_and_relabel() {
    let mut controls = Controls::default();
    assert_eq!(
        controls.label(Control::PlayPause),
        "Pause",
        "the clip starts playing"
    );
    assert_eq!(controls.activate(), Command::Pause);
    assert_eq!(controls.label(Control::PlayPause), "Play");
    assert_eq!(controls.activate(), Command::Play);
    controls.previous();
    assert_eq!(controls.activate(), Command::Rewind);
    controls.next();
    controls.next();
    assert_eq!(controls.activate(), Command::Forward);
    controls.next();
    assert_eq!(controls.label(Control::Mute), "Mute");
    assert_eq!(controls.activate(), Command::Mute(true));
    assert_eq!(controls.label(Control::Mute), "Unmute");
    assert_eq!(controls.activate(), Command::Mute(false));
}

fn app(cx: &mut TestAppContext) -> (gpui::Entity<VideoApp>, &mut gpui::VisualTestContext) {
    cx.update(|cx| cx.bind_keys(bindings()));
    let (app, cx) = cx.add_window_view(|_window, cx| VideoApp::new(cx));
    cx.update(|window, cx| window.focus(&app.focus_handle(cx), cx));
    (app, cx)
}

#[gpui::test]
fn the_remote_drives_the_control_row(cx: &mut TestAppContext) {
    let (app, cx) = app(cx);
    cx.simulate_keystrokes("enter");
    cx.read_entity(&app, |a, _| {
        assert_eq!(a.sent, [Command::Pause]);
        assert!(!a.controls.playing);
    });
    cx.simulate_keystrokes("right right enter left left left enter");
    cx.read_entity(&app, |a, _| {
        assert_eq!(
            a.sent,
            [Command::Pause, Command::Mute(true), Command::Rewind]
        );
        assert_eq!(a.controls.focused_control(), Control::Rewind);
    });
}

#[gpui::test]
fn the_desktop_shows_a_placeholder_and_escape_quits(cx: &mut TestAppContext) {
    let (app, cx) = app(cx);
    cx.read_entity(&app, |a, _| {
        assert_eq!(a.status.as_ref(), "Video plays on the TV")
    });
    let view = app.clone();
    cx.draw(
        gpui::point(px(0.), px(0.)),
        gpui::size(px(960.), px(540.)),
        move |_, _| gpui::div().size_full().child(view),
    );
    cx.simulate_keystrokes("escape");
    cx.read_entity(&app, |a, _| assert!(a.quit_requested));
}

#[test]
fn play_pause_follows_what_the_player_did_on_its_own() {
    let mut controls = Controls::default();
    assert_eq!(controls.label(Control::PlayPause), "Pause");
    // The system interrupted the player: it no longer plays.
    controls.follow_player(false);
    assert_eq!(controls.label(Control::PlayPause), "Play");
    assert_eq!(
        controls.activate(),
        Command::Play,
        "the first press asks to play, which opens the media again"
    );
    // Playback started on its own (a play queued during preparing or seeking).
    controls.follow_player(false);
    controls.follow_player(true);
    assert_eq!(controls.activate(), Command::Pause);
}
