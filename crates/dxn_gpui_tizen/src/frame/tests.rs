use super::*;

const FORCED: Option<FrameEffect> = Some(FrameEffect::Draw { force_render: true });
const NORMAL: Option<FrameEffect> = Some(FrameEffect::Draw {
    force_render: false,
});

#[test]
fn the_first_frame_is_forced() {
    let mut frames = FrameScheduler::new();
    assert_eq!(frames.ready(), FORCED);
    assert!(frames.callback_pending());
    assert_eq!(frames.ready(), None, "only once");
}

#[test]
fn a_request_draws_only_once_the_previous_callback_arrived() {
    let mut frames = FrameScheduler::new();
    assert_eq!(frames.ready(), FORCED);
    assert_eq!(
        frames.request(),
        None,
        "the forced frame's callback is pending"
    );
    assert_eq!(frames.frame_callback(), NORMAL);
    assert_eq!(
        frames.frame_callback(),
        None,
        "a callback alone does not draw"
    );
    assert_eq!(frames.request(), NORMAL, "no callback pending: draw now");
    assert_eq!(frames.frames(), 3);
}

#[test]
fn requests_coalesce_into_one_frame() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    for _ in 0..10 {
        assert_eq!(frames.request(), None);
    }
    assert_eq!(frames.frame_callback(), NORMAL);
    assert_eq!(frames.frame_callback(), None);
    assert_eq!(frames.frames(), 2);
}

#[test]
fn no_frames_while_paused_and_a_forced_frame_after_resume() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    frames.pause();
    assert!(frames.is_paused());
    assert_eq!(frames.request(), None);
    assert_eq!(frames.frame_callback(), None);
    assert_eq!(frames.ready(), None);
    assert_eq!(frames.resume(), FORCED, "resume forces a full redraw");
    assert_eq!(frames.request(), None, "its callback is pending");
    assert_eq!(
        frames.frame_callback(),
        NORMAL,
        "the request made meanwhile is kept"
    );
}

#[test]
fn pause_drops_a_callback_that_may_never_come() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    assert!(frames.callback_pending());
    frames.pause();
    assert!(!frames.callback_pending());
    assert_eq!(frames.resume(), FORCED);
}

#[test]
fn nothing_is_drawn_without_a_request() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    frames.frame_callback();
    assert_eq!(frames.ready(), None);
    assert_eq!(frames.frames(), 1);
}
