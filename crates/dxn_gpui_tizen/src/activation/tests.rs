use super::*;
use crate::lifecycle::Lifecycle;

use Effect::*;
use Report::*;

fn lifecycle(effects: &[Effect]) -> Vec<Step> {
    effects.iter().copied().map(Step::Lifecycle).collect()
}

#[test]
fn a_launched_app_is_active_and_visible() {
    let status = WindowStatus::new();
    assert!(status.is_active());
    assert!(status.is_visible());
    assert_eq!(status, WindowStatus::default());
    let mut app = Lifecycle::new();
    let mut status = WindowStatus::new();
    assert_eq!(status.steps(app.create()), lifecycle(&[Start]));
    assert_eq!(
        status.steps(app.resume()),
        [],
        "the launch resume reports nothing"
    );
    assert!(status.is_active() && status.is_visible());
}

#[test]
fn pause_reports_inactive_then_hidden_after_frames_stop() {
    let mut app = Lifecycle::new();
    let mut status = WindowStatus::new();
    status.steps(app.create());
    status.steps(app.resume());
    assert_eq!(
        status.steps(app.pause()),
        [
            Step::Lifecycle(StopFrames),
            Step::Report(Active(false)),
            Step::Report(Visible(false)),
            Step::Lifecycle(ReleaseGpu),
        ]
    );
    assert!(!status.is_active());
    assert!(!status.is_visible());
}

#[test]
fn resume_reports_visible_then_active_before_the_redraw() {
    let mut app = Lifecycle::new();
    let mut status = WindowStatus::new();
    status.steps(app.create());
    status.steps(app.resume());
    status.steps(app.pause());
    assert_eq!(
        status.steps(app.resume()),
        [
            Step::Lifecycle(RebuildGpu),
            Step::Report(Visible(true)),
            Step::Report(Active(true)),
            Step::Lifecycle(ForceRedraw),
        ]
    );
    assert!(status.is_active() && status.is_visible());
}

#[test]
fn pause_straight_after_create_hides_the_window() {
    let mut app = Lifecycle::new();
    let mut status = WindowStatus::new();
    status.steps(app.create());
    let steps = status.steps(app.pause());
    assert!(steps.contains(&Step::Report(Active(false))));
    assert!(steps.contains(&Step::Report(Visible(false))));
    let steps = status.steps(app.resume());
    assert!(steps.contains(&Step::Report(Active(true))));
}

#[test]
fn terminate_reports_inactive_before_quit_only_when_shown() {
    let mut app = Lifecycle::new();
    let mut status = WindowStatus::new();
    status.steps(app.create());
    status.steps(app.resume());
    assert_eq!(
        status.steps(app.terminate()),
        [
            Step::Lifecycle(StopFrames),
            Step::Report(Active(false)),
            Step::Report(Visible(false)),
            Step::Lifecycle(Quit),
        ]
    );

    let mut app = Lifecycle::new();
    let mut status = WindowStatus::new();
    status.steps(app.create());
    status.steps(app.pause());
    assert_eq!(
        status.steps(app.terminate()),
        lifecycle(&[StopFrames, Quit]),
        "a paused app was already reported hidden"
    );
}

#[test]
fn repeated_callbacks_report_each_change_once() {
    let mut status = WindowStatus::new();
    status.steps(vec![StopFrames]);
    assert_eq!(status.steps(vec![StopFrames]), lifecycle(&[StopFrames]));
    status.steps(vec![ForceRedraw]);
    assert_eq!(status.steps(vec![ForceRedraw]), lifecycle(&[ForceRedraw]));
    assert_eq!(
        status.steps(vec![AppControl, Start]),
        lifecycle(&[AppControl, Start]),
        "other effects pass through"
    );
}
