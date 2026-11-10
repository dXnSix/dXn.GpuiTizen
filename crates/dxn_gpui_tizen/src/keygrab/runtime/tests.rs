use super::*;
use crate::keygrab::{self, DEFAULT_GRABS, NEVER_GRABBED, ON_REQUEST};

fn shown(keys: &[&'static str]) -> GrabSet {
    let mut set = GrabSet::new(keys.to_vec());
    set.shown();
    set
}

fn calls(release: &[&'static str], grab: &[&'static str]) -> GrabCalls {
    GrabCalls {
        release: release.to_vec(),
        grab: grab.to_vec(),
    }
}

#[test]
fn info_is_grabbable_on_request_but_not_by_default() {
    assert_eq!(ON_REQUEST, ["XF86Info"]);
    assert!(!DEFAULT_GRABS.contains(&"XF86Info"));
    assert!(!keygrab::plan(None).keys.contains(&"XF86Info"));
    let plan = keygrab::plan(Some(&["info".to_owned(), "red".to_owned()]));
    assert_eq!(
        plan.keys,
        ["XF86Red", "XF86Info"],
        "the options may ask for it"
    );
    assert!(plan.refused.is_empty());
    for key in NEVER_GRABBED {
        assert!(!ON_REQUEST.contains(key), "{key}");
    }
}

#[test]
fn grabbing_while_shown_grabs_only_the_new_keys() {
    let mut set = shown(&["XF86Red"]);
    let change = set.grab(["XF86Info", "red", "blue"]);
    assert_eq!(change.calls, calls(&[], &["XF86Blue", "XF86Info"]));
    assert!(change.refused.is_empty());
    assert_eq!(set.keys(), ["XF86Red", "XF86Blue", "XF86Info"]);
    assert_eq!(
        set.grab(["XF86Info"]).calls,
        GrabCalls::default(),
        "grabbed once"
    );
}

#[test]
fn releasing_while_shown_releases_only_grabbed_keys() {
    let mut set = shown(&["XF86Red", "XF86Info"]);
    let change = set.release(["info", "XF86Green"]);
    assert_eq!(change.calls, calls(&["XF86Info"], &[]));
    assert!(
        change.refused.is_empty(),
        "a grabbable key not grabbed is no refusal"
    );
    assert_eq!(set.keys(), ["XF86Red"]);
}

#[test]
fn set_replaces_the_keys() {
    let mut set = shown(DEFAULT_GRABS);
    let change = set.set(["XF86Info", "mediaplaypause", "XF86Red"]);
    let released: Vec<_> = DEFAULT_GRABS
        .iter()
        .copied()
        .filter(|key| !["XF86Red", "XF86PlayBack"].contains(key))
        .collect();
    assert_eq!(change.calls, calls(&released, &["XF86Info"]));
    assert_eq!(set.keys(), ["XF86Red", "XF86PlayBack", "XF86Info"]);
    assert!(set.set(Vec::<String>::new()).calls.grab.is_empty());
    assert!(set.keys().is_empty());
}

#[test]
fn changes_while_hidden_wait_for_the_window() {
    let mut set = GrabSet::new(vec!["XF86Red"]);
    assert!(set.grab(["XF86Info"]).calls.is_empty(), "no window yet");
    assert_eq!(set.shown(), calls(&[], &["XF86Red", "XF86Info"]));
    assert_eq!(set.hidden(), calls(&["XF86Red", "XF86Info"], &[]));
    assert!(set.release(["XF86Red"]).calls.is_empty(), "paused");
    assert!(set.grab(["XF86Blue"]).calls.is_empty(), "paused");
    assert_eq!(
        set.shown(),
        calls(&[], &["XF86Blue", "XF86Info"]),
        "resume grabs what is wanted now"
    );
}

#[test]
fn nothing_is_grabbed_after_terminate() {
    let mut set = shown(&["XF86Red"]);
    set.grab(["XF86Info"]);
    assert_eq!(set.terminated(), calls(&["XF86Red", "XF86Info"], &[]));
    assert!(set.grab(["XF86Blue"]).calls.is_empty());
    assert!(set.shown().is_empty());
    assert!(!set.grabbed());
}

#[test]
fn runtime_changes_refuse_like_the_options() {
    let mut set = shown(&["XF86Red"]);
    let change = set.grab([
        "Return", "enter", "exit", "XF86Home", "up", "XF86Back", "green",
    ]);
    assert_eq!(change.calls, calls(&[], &["XF86Green"]));
    assert_eq!(
        change.refused,
        [
            ("Return".to_owned(), Refusal::Forbidden),
            ("enter".to_owned(), Refusal::Forbidden),
            ("exit".to_owned(), Refusal::Forbidden),
            ("XF86Home".to_owned(), Refusal::Forbidden),
            ("up".to_owned(), Refusal::NotGrabbable),
            ("XF86Back".to_owned(), Refusal::NotGrabbable),
        ]
    );
    let change = set.release(["XF86Exit", "XF86Menu"]);
    assert!(change.calls.is_empty());
    assert_eq!(
        change.refused,
        [
            ("XF86Exit".to_owned(), Refusal::Forbidden),
            ("XF86Menu".to_owned(), Refusal::NotGrabbable),
        ]
    );
}

#[test]
fn the_app_api_changes_the_main_threads_set() {
    // Its own thread: the set is per thread, and other tests may run on this one.
    std::thread::spawn(|| {
        assert_eq!(
            keygrab::grabbed_keys(),
            DEFAULT_GRABS,
            "the defaults before start"
        );
        assert!(keygrab::set(["red"]).is_empty());
        assert_eq!(
            keygrab::grab(["info", "enter"]),
            [("enter".to_owned(), Refusal::Forbidden)]
        );
        assert_eq!(keygrab::grabbed_keys(), ["XF86Red", "XF86Info"]);
        assert!(keygrab::release(["XF86Red"]).is_empty());
        assert_eq!(keygrab::grabbed_keys(), ["XF86Info"]);
        keygrab::with_grabs(|set| assert!(!set.grabbed(), "off Tizen nothing is shown"));
    })
    .join()
    .unwrap();
}
