use std::cell::RefCell;
use std::sync::Mutex;
use std::thread;

use super::*;

/// A player that records calls and fails the ones it is told to.
#[derive(Default)]
struct Fake {
    calls: Vec<String>,
    fail: Option<(&'static str, i32)>,
    state: Option<State>,
    destroyed: bool,
}

impl Fake {
    fn call(&mut self, name: &'static str) -> Result<(), i32> {
        self.calls.push(name.to_owned());
        match self.fail {
            Some((failing, code)) if failing == name => Err(code),
            _ => Ok(()),
        }
    }
}

impl Backend for Fake {
    fn set_uri(&mut self, uri: &str) -> Result<(), i32> {
        self.calls.push(format!("set_uri {uri}"));
        Ok(())
    }
    fn prepare_async(&mut self) -> Result<(), i32> {
        self.call("prepare_async")
    }
    fn unprepare(&mut self) -> Result<(), i32> {
        self.call("unprepare")
    }
    fn start(&mut self) -> Result<(), i32> {
        self.call("start")
    }
    fn pause(&mut self) -> Result<(), i32> {
        self.call("pause")
    }
    fn stop(&mut self) -> Result<(), i32> {
        self.call("stop")
    }
    fn seek(&mut self, milliseconds: i32, accurate: bool) -> Result<(), i32> {
        self.calls.push(format!("seek {milliseconds} {accurate}"));
        Ok(())
    }
    fn set_volume(&mut self, volume: f32) -> Result<(), i32> {
        self.calls.push(format!("volume {volume}"));
        Ok(())
    }
    fn set_muted(&mut self, muted: bool) -> Result<(), i32> {
        self.calls.push(format!("muted {muted}"));
        Ok(())
    }
    fn set_looping(&mut self, looping: bool) -> Result<(), i32> {
        self.calls.push(format!("looping {looping}"));
        Ok(())
    }
    fn position(&mut self) -> Result<i32, i32> {
        Ok(1500)
    }
    fn duration(&mut self) -> Result<i32, i32> {
        Ok(60_000)
    }
    fn state(&mut self) -> Result<State, i32> {
        self.state.ok_or(-38)
    }
    fn destroy(&mut self) {
        self.destroyed = true;
    }
}

fn ready() -> Player<Fake> {
    let mut player = Player::new(Fake::default());
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    assert_eq!(player.handle(Event::Prepared), [Notice::Ready]);
    player
}

#[test]
fn the_lifecycle_follows_the_player_states() {
    let mut player = Player::new(Fake::default());
    assert_eq!(player.state(), State::Idle);
    assert_eq!(player.prepare(), Err(VideoError::NoUri));
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    assert_eq!(player.state(), State::Preparing);
    assert_eq!(player.handle(Event::Prepared), [Notice::Ready]);
    assert_eq!(player.state(), State::Ready);
    assert_eq!(player.duration(), Ok(Duration::from_secs(60)));
    player.play().unwrap();
    assert_eq!(player.state(), State::Playing);
    assert_eq!(player.position(), Ok(Duration::from_millis(1500)));
    player.pause().unwrap();
    assert_eq!(player.state(), State::Paused);
    player.play().unwrap();
    player.stop().unwrap();
    assert_eq!(player.state(), State::Ready);
    player.unprepare().unwrap();
    assert_eq!(player.state(), State::Idle);
    player.destroy();
    assert_eq!(player.state(), State::Destroyed);
    assert!(player.backend().destroyed);
    assert_eq!(
        player.backend().calls,
        [
            "set_uri res/clip.mp4",
            "prepare_async",
            "start",
            "pause",
            "start",
            "stop",
            "unprepare"
        ]
    );
}

#[test]
fn calls_invalid_in_the_state_never_reach_the_player() {
    let mut player = Player::new(Fake::default());
    for (result, call) in [
        (player.play(), Call::Play),
        (player.pause(), Call::Pause),
        (player.stop(), Call::Stop),
        (player.seek(Duration::from_secs(1), false), Call::Seek),
        (player.position().map(drop), Call::Position),
        (player.duration().map(drop), Call::Duration),
        (player.unprepare(), Call::Unprepare),
    ] {
        assert_eq!(
            result,
            Err(VideoError::WrongState {
                call,
                state: State::Idle
            })
        );
    }
    let mut player = ready();
    assert_eq!(
        player.set_uri("other.mp4"),
        Err(VideoError::WrongState {
            call: Call::SetUri,
            state: State::Ready
        })
    );
    assert_eq!(
        player.prepare(),
        Err(VideoError::WrongState {
            call: Call::Prepare,
            state: State::Ready
        })
    );
    assert_eq!(
        player.pause(),
        Err(VideoError::WrongState {
            call: Call::Pause,
            state: State::Ready
        })
    );
    player.destroy();
    assert_eq!(
        player.play(),
        Err(VideoError::WrongState {
            call: Call::Play,
            state: State::Destroyed
        })
    );
    assert_eq!(player.handle(Event::Prepared), []);
    assert_eq!(
        player.backend().calls,
        ["set_uri res/clip.mp4", "prepare_async"],
        "nothing reached the player"
    );
}

#[test]
fn play_while_preparing_starts_once_prepared() {
    let mut player = Player::new(Fake::default());
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    player.play().unwrap();
    assert_eq!(player.state(), State::Preparing, "not started yet");
    assert!(!player.backend().calls.contains(&"start".to_owned()));
    assert_eq!(
        player.handle(Event::Prepared),
        [Notice::Ready, Notice::Playing]
    );
    assert_eq!(player.state(), State::Playing);

    // A pause while preparing cancels it.
    let mut player = Player::new(Fake::default());
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    player.play().unwrap();
    player.pause().unwrap();
    assert_eq!(player.handle(Event::Prepared), [Notice::Ready]);
    assert_eq!(player.state(), State::Ready);

    // A prepare cancelled by unprepare: its callback changes nothing.
    let mut player = Player::new(Fake::default());
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    player.unprepare().unwrap();
    assert_eq!(player.handle(Event::Prepared), []);
    assert_eq!(player.state(), State::Idle);
}

#[test]
fn seeks_are_serialized_and_the_latest_queued_one_wins() {
    let mut player = ready();
    player.play().unwrap();
    player.seek(Duration::from_secs(10), false).unwrap();
    assert!(player.seeking());
    player.seek(Duration::from_secs(20), false).unwrap();
    player.seek(Duration::from_secs(30), true).unwrap();
    let seeks = |player: &Player<Fake>| {
        player
            .backend()
            .calls
            .iter()
            .filter(|c| c.starts_with("seek"))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(seeks(&player), ["seek 10000 false"], "one in flight");
    assert_eq!(player.handle(Event::SeekCompleted), []);
    assert_eq!(
        seeks(&player),
        ["seek 10000 false", "seek 30000 true"],
        "the latest queued seek, the one in between dropped"
    );
    assert_eq!(player.handle(Event::SeekCompleted), [Notice::Seeked]);
    assert!(!player.seeking());
}

#[test]
fn player_errors_are_typed() {
    assert_eq!(
        PlayerError::from_code(-0x0194_0000 | 0x20 | 0x02),
        PlayerError::InvalidState
    );
    assert_eq!(
        PlayerError::from_code(-0x0194_0000 | 0x20 | 0x04),
        PlayerError::InvalidUri
    );
    assert_eq!(
        PlayerError::from_code(-0x0194_0000 | 0x20 | 0x0c),
        PlayerError::ResourceLimit
    );
    assert_eq!(
        PlayerError::from_code(-0x0194_0000 | 0x20 | 0x12),
        PlayerError::NotAvailable
    );
    assert_eq!(PlayerError::from_code(-22), PlayerError::InvalidParameter);
    assert_eq!(PlayerError::from_code(-13), PlayerError::PermissionDenied);
    assert_eq!(
        PlayerError::from_code(-1_073_741_822),
        PlayerError::FeatureNotSupported
    );
    assert_eq!(PlayerError::from_code(-99_999), PlayerError::Other(-99_999));
    for code in [-12, -22, -2, -38, -28, -13, -105, -1_073_741_822, -99_999] {
        assert_eq!(PlayerError::from_code(code).code(), code);
    }
    for n in 0x01..=0x12 {
        let code = -0x0194_0000 | 0x20 | n;
        assert_eq!(PlayerError::from_code(code).code(), code);
        assert!(!matches!(
            PlayerError::from_code(code),
            PlayerError::Other(_)
        ));
    }

    let mut player = Player::new(Fake {
        fail: Some(("prepare_async", -0x0194_0000 | 0x20 | 0x03)),
        ..Fake::default()
    });
    player.set_uri("broken.bin").unwrap();
    assert_eq!(
        player.prepare(),
        Err(VideoError::Player {
            call: Call::Prepare,
            error: PlayerError::NotSupportedFile
        })
    );
    assert_eq!(
        player.state(),
        State::Idle,
        "a refused call changes nothing"
    );
}

#[test]
fn completion_errors_and_interruptions_update_the_state() {
    let mut player = ready();
    player.play().unwrap();
    assert_eq!(player.handle(Event::Completed), [Notice::Ended]);
    assert_eq!(player.state(), State::Ready, "stopped at the end");

    let mut player = ready();
    player.set_looping(true).unwrap();
    player.play().unwrap();
    assert_eq!(player.handle(Event::Completed), [Notice::Ended]);
    assert_eq!(player.state(), State::Playing, "looping keeps playing");

    let mut player = ready();
    player.play().unwrap();
    player.backend_mut().state = Some(State::Paused);
    assert_eq!(
        player.handle(Event::Error(-0x0194_0000 | 0x20 | 0x06)),
        [Notice::Failed(PlayerError::ConnectionFailed)]
    );
    assert_eq!(player.state(), State::Paused, "the player's own state");

    let mut player = ready();
    player.play().unwrap();
    player.backend_mut().state = Some(State::Idle);
    assert_eq!(
        player.handle(Event::Interrupted),
        [Notice::Interrupted(State::Idle)]
    );
    player.backend_mut().calls.clear();
    player.prepare().unwrap();
    assert_eq!(
        player.backend().calls,
        ["set_uri res/clip.mp4", "prepare_async"],
        "unprepared by the system: the same media prepares again"
    );
    assert_eq!(player.state(), State::Preparing);

    assert_eq!(
        player.handle(Event::Buffering(140)),
        [Notice::Buffering(100)]
    );
}

#[test]
fn volume_mute_and_looping_reach_the_player() {
    let mut player = Player::new(Fake::default());
    player.set_volume(1.5).unwrap();
    player.set_muted(true).unwrap();
    player.set_looping(true).unwrap();
    assert_eq!(
        player.backend().calls,
        ["volume 1", "muted true", "looping true"]
    );
}

thread_local! {
    static ROUTED: RefCell<Vec<(PlayerId, Event, thread::ThreadId)>> = const { RefCell::new(Vec::new()) };
}

fn route(id: PlayerId, event: Event) {
    ROUTED.with(|routed| {
        routed
            .borrow_mut()
            .push((id, event, thread::current().id()))
    });
}

#[test]
fn callbacks_from_player_threads_reach_the_main_thread_in_order() {
    type Job = Box<dyn FnOnce() + Send>;
    let posts: Arc<Mutex<Vec<Job>>> = Arc::default();
    let inbox = {
        let posts = Arc::clone(&posts);
        Inbox::new(move |job| posts.lock().unwrap().push(job), route)
    };
    let threads: Vec<_> = (0..4u64)
        .map(|n| {
            let inbox = Arc::clone(&inbox);
            thread::spawn(move || {
                for percent in 0..25 {
                    inbox.post(PlayerId(n), Event::Buffering(percent));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    let woken = posts.lock().unwrap().len();
    assert!(woken >= 1, "the main loop was woken");
    // The main loop runs the posted drains on this (the main) thread.
    let jobs: Vec<Job> = std::mem::take(&mut *posts.lock().unwrap());
    for job in jobs {
        job();
    }
    let main = thread::current().id();
    let routed = ROUTED.with(|routed| routed.take());
    assert_eq!(routed.len(), 100, "every event, once");
    assert!(
        routed.iter().all(|(_, _, on)| *on == main),
        "on the main thread"
    );
    for n in 0..4 {
        let percents: Vec<i32> = routed
            .iter()
            .filter(|(id, _, _)| *id == PlayerId(n))
            .map(|(_, event, _)| match event {
                Event::Buffering(p) => *p,
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(percents, (0..25).collect::<Vec<_>>(), "player {n} in order");
    }
    // An idle inbox wakes the loop once for a burst.
    inbox.post(PlayerId(9), Event::Completed);
    inbox.post(PlayerId(9), Event::Completed);
    assert_eq!(posts.lock().unwrap().len(), 1, "one wake for the burst");
}

#[test]
fn the_overlay_is_the_default_where_the_window_can_show_it() {
    assert_eq!(Output::choose(None, true), Output::Overlay);
    assert_eq!(Output::choose(None, false), Output::Texture);
    assert_eq!(Output::choose(Some(Output::Texture), true), Output::Texture);
    assert_eq!(
        Output::choose(Some(Output::Overlay), false),
        Output::Overlay
    );

    let mut player = ready();
    assert_eq!(player.handle(Event::Frame), [Notice::Frame]);
}

#[test]
fn the_overlay_surface_hears_only_what_changed() {
    let rect = |x, y, width, height| DeviceRect {
        x,
        y,
        width,
        height,
    };
    let mut placement = OverlayPlacement::default();
    assert_eq!(
        placement.place(rect(100, 50, 1280, 720)),
        [
            OverlayOp::Move { x: 100, y: 50 },
            OverlayOp::Resize {
                width: 1280,
                height: 720
            },
            OverlayOp::Show
        ]
    );
    assert_eq!(placement.place(rect(100, 50, 1280, 720)), [], "unchanged");
    assert_eq!(
        placement.place(rect(0, 0, 1280, 720)),
        [OverlayOp::Move { x: 0, y: 0 }]
    );
    assert_eq!(
        placement.place(rect(0, 0, 1920, 1080)),
        [OverlayOp::Resize {
            width: 1920,
            height: 1080
        }]
    );
    assert_eq!(placement.hide(), [OverlayOp::Hide]);
    assert_eq!(placement.hide(), [], "hidden once");
    assert_eq!(
        placement.place(rect(0, 0, 1920, 1080)),
        [OverlayOp::Show],
        "shown again where it was"
    );
    assert_eq!(placement.place(rect(0, 0, 0, 1080)), [OverlayOp::Hide]);
}

#[test]
fn a_failed_or_abandoned_seek_does_not_block_later_ones() {
    let seeks = |player: &Player<Fake>| {
        player
            .backend()
            .calls
            .iter()
            .filter(|c| c.starts_with("seek"))
            .count()
    };
    let mut player = ready();
    player.play().unwrap();
    player.seek(Duration::from_secs(14), false).unwrap();
    player.seek(Duration::from_secs(20), false).unwrap();
    player.backend_mut().state = Some(State::Playing);
    // `player.h`: a failed seek comes as an error, not as its completion.
    assert_eq!(
        player.handle(Event::Error(-0x0194_0000 | 0x20 | 0x01)),
        [Notice::Failed(PlayerError::SeekFailed)]
    );
    assert!(!player.seeking());
    player.seek(Duration::from_secs(2), false).unwrap();
    assert_eq!(
        seeks(&player),
        2,
        "issued, not queued behind the failed one"
    );

    // A stop forgets the seek in flight too.
    player.stop().unwrap();
    assert!(!player.seeking());
    player.seek(Duration::from_secs(1), false).unwrap();
    assert_eq!(seeks(&player), 3);
}

#[test]
fn play_and_pause_while_seeking_wait_for_the_seeks() {
    let mut player = ready();
    player.play().unwrap();
    player.seek(Duration::from_secs(2), false).unwrap();
    player.pause().unwrap();
    assert!(
        !player.backend().calls.contains(&"pause".to_owned()),
        "`player.h`: pause during a seek is an invalid operation"
    );
    player.seek(Duration::from_secs(3), false).unwrap();
    assert_eq!(player.handle(Event::SeekCompleted), []);
    assert_eq!(
        player.handle(Event::SeekCompleted),
        [Notice::Seeked, Notice::Paused]
    );
    assert_eq!(player.state(), State::Paused);
    assert_eq!(
        player.backend().calls.last().map(String::as_str),
        Some("pause")
    );

    player.seek(Duration::from_secs(1), false).unwrap();
    player.play().unwrap();
    assert_eq!(
        player.handle(Event::SeekCompleted),
        [Notice::Seeked, Notice::Playing]
    );
    assert_eq!(player.state(), State::Playing);
}

#[test]
fn a_prepare_that_completes_after_an_error_still_counts() {
    let mut player = Player::new(Fake::default());
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    player.play().unwrap();
    // The player reports idle while it prepares.
    player.backend_mut().state = Some(State::Idle);
    assert_eq!(
        player.handle(Event::Error(-0x0194_0000 | 0x20 | 0x0e)),
        [Notice::Failed(PlayerError::NotSupportedAudioCodec)]
    );
    assert_eq!(player.state(), State::Idle);
    assert_eq!(
        player.handle(Event::Prepared),
        [Notice::Ready],
        "the late callback makes the player ready"
    );
    assert_eq!(player.state(), State::Ready);

    // Preparing again after a failed prepare cancels the old one first.
    let mut player = Player::new(Fake::default());
    player.set_uri("res/clip.mp4").unwrap();
    player.prepare().unwrap();
    player.backend_mut().state = Some(State::Idle);
    player.handle(Event::Error(-0x0194_0000 | 0x20 | 0x03));
    player.backend_mut().calls.clear();
    player.prepare().unwrap();
    assert_eq!(
        player.backend().calls,
        ["unprepare", "set_uri res/clip.mp4", "prepare_async"]
    );
}

#[test]
fn the_overlay_rectangle_snaps_like_the_hole() {
    // Edges rounded one by one: 10.3×2 → 21 and (10.3 + 100.4)×2 = 221.4 → 221.
    assert_eq!(
        DeviceRect::snapped(10.3, 20.6, 100.4, 50.2, 2.0),
        DeviceRect {
            x: 21,
            y: 41,
            width: 200,
            height: 101
        }
    );
    assert_eq!(
        DeviceRect::snapped(0., 0., 960., 540., 2.0),
        DeviceRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080
        }
    );
}
