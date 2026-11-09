use gpui::{Entity, Focusable, TestAppContext, VisualTestContext};

use super::*;
use crate::view::{LongList, ROWS, bindings};

fn list(cx: &mut TestAppContext) -> (Entity<LongList>, &mut VisualTestContext) {
    cx.update(|cx| cx.bind_keys(bindings()));
    let (list, cx) = cx.add_window_view(|_window, cx| LongList::new(cx));
    cx.update(|window, cx| window.focus(&list.focus_handle(cx), cx));
    (list, cx)
}

fn field_focused(list: &Entity<LongList>, cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx| {
        let field = list.read(cx).filter.focus_handle(cx);
        field.is_focused(window)
    })
}

fn labels(list: &Entity<LongList>, cx: &mut VisualTestContext) -> Vec<String> {
    cx.read_entity(list, |l, _| {
        l.rows.iter().map(|&row| label(row).to_string()).collect()
    })
}

#[test]
fn a_query_keeps_the_rows_whose_label_contains_it() {
    assert_eq!(rows("").len(), ROWS, "no query keeps every row");
    assert_eq!(rows("   ").len(), ROWS);
    assert_eq!(rows("row 1000"), [999]);
    assert_eq!(rows("ROW 7")[0], 6, "case does not matter");
    assert_eq!(rows("  999 "), [998], "the query is trimmed");
    assert!(rows("xyz").is_empty());
    let twelve = rows("12");
    assert_eq!(&twelve[..3], [11, 111, 119]);
    assert!(twelve.iter().all(|&row| label(row).contains("12")));
    assert_eq!(
        twelve.len(),
        (0..ROWS)
            .filter(|row| (row + 1).to_string().contains("12"))
            .count()
    );
}

#[gpui::test]
fn up_from_the_first_row_enters_the_field_and_down_leaves_it(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("down up");
    assert!(
        !field_focused(&list, cx),
        "row 2 to row 1 stays in the list"
    );
    cx.simulate_keystrokes("up");
    assert!(field_focused(&list, cx));
    cx.simulate_keystrokes("up");
    assert!(field_focused(&list, cx), "nothing above the field");
    cx.simulate_keystrokes("down");
    assert!(!field_focused(&list, cx));
    for leave in ["enter", "back", "escape"] {
        cx.simulate_keystrokes("up");
        assert!(field_focused(&list, cx));
        cx.simulate_keystrokes(leave);
        assert!(!field_focused(&list, cx), "{leave} leaves the field");
        cx.read_entity(&list, |l, _| {
            assert!(!l.quit_requested, "{leave} in the field does not quit")
        });
    }
}

#[gpui::test]
fn typing_narrows_the_list_and_backspace_widens_it(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("down down");
    cx.update(|_, cx| list.update(cx, |l, cx| l.select(0, cx)));
    cx.simulate_keystrokes("up");
    cx.simulate_input("99");
    let shown = labels(&list, cx);
    assert_eq!(shown.len(), rows("99").len());
    assert!(shown.iter().all(|label| label.contains("99")));
    cx.read_entity(&list, |l, _| {
        assert_eq!(l.selected, 0);
        assert_eq!(l.selected_row(), Some(98), "Row 99 first");
    });
    cx.simulate_keystrokes("backspace");
    cx.read_entity(&list, |l, cx| {
        assert_eq!(l.filter.read(cx).content, "9");
        assert_eq!(l.rows, rows("9"));
    });
    cx.simulate_input("x");
    cx.read_entity(&list, |l, _| {
        assert!(l.rows.is_empty());
        assert_eq!(l.selected_row(), None);
    });
    cx.simulate_keystrokes("backspace backspace");
    cx.read_entity(&list, |l, _| assert_eq!(l.rows.len(), ROWS));
}

#[gpui::test]
fn the_ime_marks_its_preedit_and_the_commit_replaces_it(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("up");
    cx.simulate_input("Row 5");
    let field = cx.read_entity(&list, |l, _| l.filter.clone());
    // What the Tizen platform does for an Ecore_IMF preedit and commit.
    cx.update(|window, cx| {
        field.update(cx, |f, cx| {
            f.replace_and_mark_text_in_range(None, "0", Some(1..1), window, cx)
        })
    });
    cx.read_entity(&field, |f, _| {
        assert_eq!(f.content, "Row 50");
        assert_eq!(f.marked_range, Some(5..6));
        assert_eq!(f.selected_range, 6..6);
    });
    cx.read_entity(&list, |l, _| assert_eq!(l.rows, rows("row 50")));
    cx.update(|window, cx| {
        field.update(cx, |f, cx| f.replace_text_in_range(None, "00", window, cx))
    });
    cx.read_entity(&field, |f, _| {
        assert_eq!(f.content, "Row 500");
        assert_eq!(f.marked_range, None);
    });
    cx.read_entity(&list, |l, _| assert_eq!(l.rows, [499]));
}

#[gpui::test]
fn left_right_home_end_move_by_graphemes(cx: &mut TestAppContext) {
    let (list, cx) = list(cx);
    cx.simulate_keystrokes("up");
    cx.simulate_input("aé");
    let field = cx.read_entity(&list, |l, _| l.filter.clone());
    cx.read_entity(&field, |f, _| assert_eq!(f.selected_range, 3..3));
    cx.simulate_keystrokes("left");
    cx.read_entity(&field, |f, _| assert_eq!(f.selected_range, 1..1));
    cx.simulate_keystrokes("home");
    cx.read_entity(&field, |f, _| assert_eq!(f.selected_range, 0..0));
    cx.simulate_keystrokes("right");
    cx.read_entity(&field, |f, _| assert_eq!(f.selected_range, 1..1));
    cx.simulate_keystrokes("delete");
    cx.read_entity(&field, |f, _| assert_eq!(f.content, "a"));
    cx.simulate_keystrokes("end");
    cx.read_entity(&field, |f, _| assert_eq!(f.selected_range, 1..1));
}
