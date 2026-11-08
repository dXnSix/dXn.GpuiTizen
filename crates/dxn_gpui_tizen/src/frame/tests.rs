use super::*;

const FORCED: Option<FrameEffect> = Some(FrameEffect::Draw {
    force_render: true,
    epoch: 0,
});
const NORMAL: Option<FrameEffect> = Some(FrameEffect::Draw {
    force_render: false,
    epoch: 0,
});

/// A draw of the epoch after `pauses` pauses.
fn draw(force_render: bool, pauses: u32) -> Option<FrameEffect> {
    Some(FrameEffect::Draw {
        force_render,
        epoch: pauses,
    })
}

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
    assert_eq!(
        frames.resume(),
        draw(true, 1),
        "resume forces a full redraw"
    );
    assert_eq!(frames.request(), None, "its callback is pending");
    assert_eq!(
        frames.frame_callback(),
        draw(false, 1),
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
    assert_eq!(frames.resume(), draw(true, 1));
}

#[test]
fn nothing_is_drawn_without_a_request() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    frames.frame_callback();
    assert_eq!(frames.ready(), None);
    assert_eq!(frames.frames(), 1);
}

#[test]
fn a_frame_that_was_not_presented_does_not_block_the_next() {
    let mut frames = FrameScheduler::new();
    assert_eq!(frames.ready(), FORCED);
    assert_eq!(
        frames.not_presented(false),
        None,
        "GPUI had nothing to draw"
    );
    assert!(!frames.callback_pending());
    assert_eq!(
        frames.request(),
        NORMAL,
        "no callback will come for the skipped frame"
    );
}

#[test]
fn a_request_made_during_an_unpresented_frame_is_served() {
    let mut frames = FrameScheduler::new();
    assert_eq!(frames.ready(), FORCED);
    assert_eq!(frames.request(), None, "made while the frame is in flight");
    assert_eq!(
        frames.not_presented(false),
        NORMAL,
        "served once the frame turned out unpresented"
    );
}

#[test]
fn a_failed_present_is_retried_as_a_forced_frame() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    frames.frame_callback();
    assert_eq!(frames.request(), NORMAL);
    assert_eq!(
        frames.not_presented(true),
        FORCED,
        "GPUI cleared its dirty state: force the retry"
    );
    assert!(frames.callback_pending());
}

#[test]
fn needs_redraw_forces_a_frame() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    assert_eq!(frames.force(), None, "after the pending callback");
    assert_eq!(frames.frame_callback(), FORCED);
    frames.frame_callback();
    assert_eq!(frames.force(), FORCED, "no callback pending: at once");
}

#[test]
fn persistent_failures_back_off() {
    let mut frames = FrameScheduler::new();
    frames.ready();
    for _ in 0..3 {
        frames.not_presented(true);
        assert_eq!(frames.retry_delay(), std::time::Duration::ZERO);
    }
    frames.not_presented(true);
    assert_eq!(frames.retry_delay(), std::time::Duration::from_millis(100));
    frames.presented();
    assert_eq!(
        frames.retry_delay(),
        std::time::Duration::ZERO,
        "a presented frame resets it"
    );
}

#[test]
fn a_draw_posted_before_a_pause_is_dropped() {
    let mut frames = FrameScheduler::new();
    let posted = frames.ready();
    assert_eq!(frames.posted(posted), FORCED, "nothing happened meanwhile");
    let posted = frames
        .frame_callback()
        .or_else(|| frames.request())
        .or(NORMAL);
    frames.pause();
    assert_eq!(frames.posted(posted), None, "paused meanwhile");
    let forced = frames.resume();
    assert_eq!(
        frames.posted(posted),
        None,
        "resumed meanwhile: the old draw stays dropped"
    );
    assert_eq!(
        frames.posted(forced),
        draw(true, 1),
        "the resume's own draw runs"
    );
}
