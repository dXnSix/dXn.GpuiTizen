use gpui::TestAppContext;

use super::*;

fn list(cx: &mut TestAppContext) -> (gpui::Entity<LongList>, &mut gpui::VisualTestContext) {
    cx.update(|cx| cx.bind_keys(bindings()));
    let (list, cx) = cx.add_window_view(|_window, cx| LongList::new(cx));
    cx.update(|window, cx| window.focus(&list.focus_handle(cx), cx));
    (list, cx)
}

#[gpui::test]
fn down_and_up_move_the_selection(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("down down down");
    cx.read_entity(&list, |l, _| assert_eq!(l.selected, 3));
    cx.simulate_keystrokes("up");
    cx.read_entity(&list, |l, _| assert_eq!(l.selected, 2));
}

#[gpui::test]
fn the_selection_stops_at_both_ends(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("up up");
    cx.read_entity(&list, |l, _| assert_eq!(l.selected, 0));
    cx.update(|_, cx| list.update(cx, |l, cx| l.select(ROWS + 50, cx)));
    cx.read_entity(&list, |l, _| assert_eq!(l.selected, ROWS - 1));
    cx.simulate_keystrokes("down");
    cx.read_entity(&list, |l, _| assert_eq!(l.selected, ROWS - 1));
}

/// Lays the view out in a 1080p-sized space (logical 960x540). The first pass measures the rows;
/// the deferred scroll applies once the item size is known.
fn lay_out(list: &gpui::Entity<LongList>, cx: &mut gpui::VisualTestContext) {
    for _ in 0..2 {
        let view = list.clone();
        cx.draw(
            gpui::point(px(0.), px(0.)),
            gpui::size(px(960.), px(540.)),
            move |_, _| gpui::div().size_full().child(view),
        );
    }
}

/// The rows the viewport fully shows, from the list's scroll offset and sizes.
fn visible_rows(
    list: &gpui::Entity<LongList>,
    cx: &mut gpui::VisualTestContext,
) -> std::ops::Range<usize> {
    cx.read_entity(list, |l, _| {
        let state = l.scroll.0.borrow();
        assert!(
            state.deferred_scroll_to_item.is_none(),
            "the scroll was applied"
        );
        let size = state.last_item_size.expect("laid out");
        // `contents` is the whole list, `item` the viewport.
        let row = f32::from(size.contents.height) / ROWS as f32;
        let top = -f32::from(state.base_handle.offset().y);
        let first = (top / row).ceil() as usize;
        let end = ((top + f32::from(size.item.height)) / row).floor() as usize;
        first..end
    })
}

#[gpui::test]
fn the_selection_stays_visible(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    lay_out(&list, cx);
    assert!(visible_rows(&list, cx).contains(&0));
    cx.update(|_, cx| list.update(cx, |l, cx| l.select(500, cx)));
    lay_out(&list, cx);
    let visible = visible_rows(&list, cx);
    assert!(visible.contains(&500), "row 500 is visible ({visible:?})");
    assert!(!visible.contains(&0), "the list scrolled");
    cx.simulate_keystrokes("down");
    lay_out(&list, cx);
    let visible = visible_rows(&list, cx);
    assert!(visible.contains(&501), "row 501 is visible ({visible:?})");
    cx.update(|_, cx| list.update(cx, |l, cx| l.select(ROWS - 1, cx)));
    lay_out(&list, cx);
    assert!(
        visible_rows(&list, cx).contains(&(ROWS - 1)),
        "the last row"
    );
    cx.update(|_, cx| list.update(cx, |l, cx| l.select(0, cx)));
    lay_out(&list, cx);
    assert!(visible_rows(&list, cx).contains(&0), "back at the top");
}

#[gpui::test]
fn back_quits(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("back");
    cx.read_entity(&list, |l, _| assert!(l.quit_requested));
}

#[test]
fn rows_and_labels() {
    assert_eq!(ROWS, 1_000);
    assert_eq!(label(0), "Row 1");
    assert_eq!(label(999), "Row 1000");
    let keys: Vec<String> = bindings()
        .iter()
        .map(|b| b.keystrokes()[0].key().to_owned())
        .collect();
    for name in ["up", "down", "back"] {
        assert!(
            keys.iter().any(|k| k == name),
            "{name} missing from {keys:?}"
        );
    }
}
