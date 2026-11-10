use super::*;

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn by_default_colors_media_and_channels_are_grabbed() {
    let plan = plan(None);
    assert_eq!(plan.keys, DEFAULT_GRABS);
    assert!(plan.refused.is_empty());
    for key in [
        "XF86Red",
        "XF86Blue",
        "XF86AudioPlay",
        "XF86PlayBack",
        "XF86RaiseChannel",
    ] {
        assert!(plan.keys.contains(&key), "{key}");
    }
}

#[test]
fn the_defaults_never_contain_the_forbidden_keys() {
    for key in NEVER_GRABBED {
        assert!(!DEFAULT_GRABS.contains(key), "{key}");
    }
    for key in ["XF86Back", "Up", "Down", "Left", "Right", "KP_Enter"] {
        assert!(
            !DEFAULT_GRABS.contains(&key),
            "{key} reaches the app ungrabbed"
        );
    }
}

#[test]
fn options_narrow_the_set_by_tizen_or_gpui_name() {
    let plan = plan(Some(&names(&["red", "XF86Blue", "mediaplaypause", "red"])));
    assert_eq!(plan.keys, ["XF86Red", "XF86Blue", "XF86PlayBack"]);
    assert!(plan.refused.is_empty());
    assert!(
        super::plan(Some(&[])).keys.is_empty(),
        "an empty list grabs nothing"
    );
}

#[test]
fn return_exit_and_home_are_refused() {
    let plan = plan(Some(&names(&[
        "Return", "enter", "KP_Enter", "XF86Exit", "exit", "XF86Home", "red",
    ])));
    assert_eq!(plan.keys, ["XF86Red"]);
    assert_eq!(
        plan.refused,
        [
            ("Return".to_owned(), Refusal::Forbidden),
            ("enter".to_owned(), Refusal::Forbidden),
            ("KP_Enter".to_owned(), Refusal::Forbidden),
            ("XF86Exit".to_owned(), Refusal::Forbidden),
            ("exit".to_owned(), Refusal::Forbidden),
            ("XF86Home".to_owned(), Refusal::Forbidden),
        ]
    );
}

#[test]
fn keys_outside_the_set_are_refused() {
    let plan = plan(Some(&names(&["XF86Menu", "up", "XF86Back", "green"])));
    assert_eq!(plan.keys, ["XF86Green"]);
    assert_eq!(
        plan.refused,
        [
            ("XF86Menu".to_owned(), Refusal::NotGrabbable),
            ("up".to_owned(), Refusal::NotGrabbable),
            ("XF86Back".to_owned(), Refusal::NotGrabbable),
        ]
    );
}

#[test]
fn grabs_follow_show_hide_and_terminate() {
    let mut grabs = KeyGrabs::default();
    assert!(!grabs.grabbed());
    assert_eq!(grabs.hidden(), None, "nothing to release");
    assert_eq!(grabs.shown(), Some(GrabCall::Grab));
    assert_eq!(grabs.shown(), None, "grabbed once");
    assert!(grabs.grabbed());
    assert_eq!(grabs.hidden(), Some(GrabCall::Release));
    assert_eq!(grabs.hidden(), None);
    assert_eq!(grabs.shown(), Some(GrabCall::Grab), "resumed");
    assert_eq!(grabs.terminated(), Some(GrabCall::Release));
    assert_eq!(grabs.shown(), None, "never after terminate");
    assert_eq!(grabs.terminated(), None);
}
