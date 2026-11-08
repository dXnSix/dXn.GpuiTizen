use gpui::TestAppContext;

use super::*;

fn counter(cx: &mut TestAppContext) -> (gpui::Entity<Counter>, &mut gpui::VisualTestContext) {
    cx.update(|cx| cx.bind_keys(bindings()));
    let (counter, cx) = cx.add_window_view(|_window, cx| Counter::new(cx));
    cx.update(|window, cx| window.focus(&counter.focus_handle(cx), cx));
    (counter, cx)
}

#[gpui::test]
fn enter_presses_the_focused_button(cx: &mut TestAppContext) {
    let (counter, cx) = counter(cx);
    cx.read_entity(&counter, |c, _| {
        assert_eq!((c.count, c.focused), (0, Button::Plus))
    });
    cx.simulate_keystrokes("enter enter");
    cx.read_entity(&counter, |c, _| assert_eq!(c.count, 2));
}

#[gpui::test]
fn left_and_right_move_the_focus(cx: &mut TestAppContext) {
    let (counter, cx) = counter(cx);
    cx.simulate_keystrokes("left enter enter enter");
    cx.read_entity(&counter, |c, _| {
        assert_eq!((c.count, c.focused), (-3, Button::Minus))
    });
    cx.simulate_keystrokes("left");
    cx.read_entity(&counter, |c, _| {
        assert_eq!(c.focused, Button::Minus, "left stays on the first button")
    });
    cx.simulate_keystrokes("right enter");
    cx.read_entity(&counter, |c, _| {
        assert_eq!((c.count, c.focused), (-2, Button::Plus))
    });
}

#[gpui::test]
fn back_quits(cx: &mut TestAppContext) {
    let (counter, cx) = counter(cx);
    cx.simulate_keystrokes("back");
    cx.read_entity(&counter, |c, _| assert!(c.quit_requested));
}

#[gpui::test]
fn escape_quits_on_the_desktop(cx: &mut TestAppContext) {
    let (counter, cx) = counter(cx);
    cx.simulate_keystrokes("escape");
    cx.read_entity(&counter, |c, _| assert!(c.quit_requested));
}

#[test]
fn the_bindings_use_the_tv_key_names() {
    let keys: Vec<String> = bindings()
        .iter()
        .map(|b| b.keystrokes()[0].key().to_owned())
        .collect();
    for name in ["left", "right", "enter", "back"] {
        assert!(
            keys.iter().any(|k| k == name),
            "{name} missing from {keys:?}"
        );
    }
}
