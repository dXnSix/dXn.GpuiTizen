use std::ops::Range;

use gpui::{TextInputStateChange, UTF16Selection};

use super::*;

/// A text field with GPUI's input-handler semantics (as GPUI's text input example implements
/// them), over UTF-16 offsets, recording every call.
#[derive(Default)]
struct Field {
    text: Vec<u16>,
    selection: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    calls: Vec<String>,
}

impl Field {
    fn new(text: &str, cursor: usize) -> Self {
        Self {
            text: text.encode_utf16().collect(),
            selection: cursor..cursor,
            ..Self::default()
        }
    }

    fn text(&self) -> String {
        String::from_utf16(&self.text).unwrap()
    }

    fn clamp(&self, range: Range<usize>) -> Range<usize> {
        range.start.min(self.text.len())..range.end.min(self.text.len())
    }

    fn splice(&mut self, range: Option<Range<usize>>, text: &str) -> Range<usize> {
        let range = range
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        let inserted: Vec<u16> = text.encode_utf16().collect();
        let len = inserted.len();
        self.text.splice(range.clone(), inserted);
        range.start..range.start + len
    }
}

impl TextInput for Field {
    fn selection(&mut self) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.selection.clone(),
            reversed: self.reversed,
        })
    }

    fn marked_range(&mut self) -> Option<Range<usize>> {
        self.marked.clone()
    }

    fn text_for_range(&mut self, range: Range<usize>) -> Option<String> {
        let range = self.clamp(range);
        Some(String::from_utf16_lossy(&self.text[range]))
    }

    fn replace_text(&mut self, range: Option<Range<usize>>, text: &str) {
        self.calls.push(format!("replace {range:?} {text:?}"));
        let inserted = self.splice(range, text);
        self.selection = inserted.end..inserted.end;
        self.reversed = false;
        self.marked = None;
    }

    fn replace_and_mark_text(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        self.calls
            .push(format!("mark {range:?} {text:?} {selected:?}"));
        let inserted = self.splice(range, text);
        self.marked = (!text.is_empty()).then_some(inserted.clone());
        self.selection = selected
            .map(|s| inserted.start + s.start..inserted.start + s.end)
            .unwrap_or(inserted.end..inserted.end);
    }
}

fn preedit(text: &str, cursor: usize) -> ImeEvent {
    ImeEvent::PreeditChanged {
        text: text.to_owned(),
        cursor,
    }
}

#[test]
fn a_preedit_becomes_marked_text_with_its_cursor() {
    let mut field = Field::new("ab", 1);
    apply(&ImeEvent::PreeditStart, &mut field);
    apply(&preedit("한", 1), &mut field);
    assert_eq!(field.text(), "a한b");
    assert_eq!(field.marked, Some(1..2));
    assert_eq!(
        field.selection,
        2..2,
        "the IME's cursor, after the syllable"
    );
    apply(&preedit("한글", 1), &mut field);
    assert_eq!(field.text(), "a한글b", "the new preedit replaces the old");
    assert_eq!(field.marked, Some(1..3));
    assert_eq!(field.selection, 2..2, "the cursor inside the preedit");
    assert_eq!(
        field.calls,
        [
            "mark None \"한\" Some(1..1)",
            "mark Some(1..2) \"한글\" Some(1..1)"
        ]
    );
}

#[test]
fn the_preedit_cursor_counts_characters_not_utf16_units() {
    let mut field = Field::new("", 0);
    apply(&preedit("😀x", 1), &mut field);
    assert_eq!(field.marked, Some(0..3));
    assert_eq!(field.selection, 2..2, "after the surrogate pair");
}

#[test]
fn a_commit_replaces_the_marked_text() {
    let mut field = Field::new("ab", 1);
    apply(&preedit("ㅎ", 1), &mut field);
    apply(&ImeEvent::Commit("한".into()), &mut field);
    assert_eq!(field.text(), "a한b");
    assert_eq!(field.marked, None);
    assert_eq!(field.selection, 2..2);
    apply(&ImeEvent::PreeditEnd, &mut field);
    assert_eq!(field.text(), "a한b", "nothing left to remove");
}

#[test]
fn a_commit_without_preedit_inserts_at_the_selection() {
    let mut field = Field::new("hello", 0);
    field.selection = 1..4;
    apply(&ImeEvent::Commit("i".into()), &mut field);
    assert_eq!(field.text(), "hio");
    assert_eq!(field.calls, ["replace None \"i\""]);
}

#[test]
fn a_cleared_preedit_removes_the_marked_text_before_the_commit() {
    // Ecore_IMF's Wayland module clears the preedit, ends it, then commits.
    let mut field = Field::new("", 0);
    apply(&preedit("ka", 2), &mut field);
    apply(&preedit("", 0), &mut field);
    assert_eq!(field.text(), "");
    apply(&ImeEvent::PreeditEnd, &mut field);
    apply(&ImeEvent::Commit("か".into()), &mut field);
    assert_eq!(field.text(), "か");
    assert_eq!(field.selection, 1..1);
}

#[test]
fn an_empty_preedit_without_marked_text_does_nothing() {
    let mut field = Field::new("abc", 3);
    apply(&preedit("", 0), &mut field);
    apply(&ImeEvent::PreeditEnd, &mut field);
    assert!(field.calls.is_empty());
}

#[test]
fn a_cancelled_preedit_leaves_nothing_behind() {
    let mut field = Field::new("x", 1);
    apply(&preedit("abc", 3), &mut field);
    apply(&ImeEvent::PreeditEnd, &mut field);
    assert_eq!(field.text(), "x");
    assert_eq!(field.marked, None);
}

#[test]
fn delete_surrounding_deletes_characters_around_the_cursor() {
    let mut field = Field::new("a😀bc", 3);
    apply(
        &ImeEvent::DeleteSurrounding {
            offset: -1,
            n_chars: 1,
        },
        &mut field,
    );
    assert_eq!(field.text(), "abc", "one character, two UTF-16 units");
    assert_eq!(field.selection, 1..1);
    apply(
        &ImeEvent::DeleteSurrounding {
            offset: 0,
            n_chars: 2,
        },
        &mut field,
    );
    assert_eq!(field.text(), "a");
}

#[test]
fn delete_surrounding_spans_the_cursor_and_clamps() {
    let mut field = Field::new("abcdef", 3);
    apply(
        &ImeEvent::DeleteSurrounding {
            offset: -2,
            n_chars: 3,
        },
        &mut field,
    );
    assert_eq!(field.text(), "aef");
    let mut field = Field::new("abc", 1);
    apply(
        &ImeEvent::DeleteSurrounding {
            offset: -5,
            n_chars: 20,
        },
        &mut field,
    );
    assert_eq!(field.text(), "", "clamped at both ends");
    let mut field = Field::new("abc", 1);
    apply(
        &ImeEvent::DeleteSurrounding {
            offset: 0,
            n_chars: 0,
        },
        &mut field,
    );
    assert!(field.calls.is_empty(), "nothing to delete");
}

#[test]
fn the_cursor_is_the_selection_head() {
    let mut field = Field::new("héllo", 0);
    field.selection = 1..4;
    assert_eq!(cursor_chars(&mut field), Some(4));
    field.reversed = true;
    assert_eq!(cursor_chars(&mut field), Some(1));
    assert_eq!(surrounding(&mut field, 64), Some(("héllo".into(), 1)));
    let mut field = Field::new("😀😀", 2);
    assert_eq!(cursor_chars(&mut field), Some(1), "characters, not units");
}

#[test]
fn editable_focus_shows_and_hides_the_keyboard() {
    let mut focus = ImeFocus::<u32>::default();
    assert!(!focus.editable());
    assert!(focus.show_keyboard().is_empty(), "no field: no keyboard");
    assert_eq!(
        focus.state_changed(TextInputStateChange::FocusGained),
        [
            ImfCall::FocusIn,
            ImfCall::CursorPosition,
            ImfCall::ShowPanel
        ]
    );
    assert!(focus.editable());
    assert!(
        focus
            .state_changed(TextInputStateChange::FocusGained)
            .is_empty()
    );
    assert_eq!(
        focus.state_changed(TextInputStateChange::SelectionChanged),
        [ImfCall::CursorPosition]
    );
    assert_eq!(
        focus.state_changed(TextInputStateChange::ContentChanged),
        [ImfCall::CursorPosition],
        "edits keep the composition"
    );
    assert_eq!(focus.show_keyboard(), [ImfCall::ShowPanel]);
    assert_eq!(focus.hide_keyboard(), [ImfCall::HidePanel]);
    assert_eq!(
        focus.state_changed(TextInputStateChange::FocusLost),
        [
            ImfCall::BeginDiscard,
            ImfCall::Reset,
            ImfCall::FocusOut,
            ImfCall::EndDiscard,
            ImfCall::HidePanel
        ]
    );
    assert!(!focus.editable());
    assert!(
        focus
            .state_changed(TextInputStateChange::FocusLost)
            .is_empty()
    );
    assert!(
        focus
            .state_changed(TextInputStateChange::SelectionChanged)
            .is_empty()
    );
    assert!(focus.hide_keyboard().is_empty());
}

#[test]
fn the_window_focus_follows_the_system() {
    let mut focus = ImeFocus::<u32>::default();
    assert!(focus.window_focus(false).is_empty());
    focus.state_changed(TextInputStateChange::FocusGained);
    assert_eq!(
        focus.window_focus(false),
        [ImfCall::Reset, ImfCall::FocusOut]
    );
    assert_eq!(
        focus.window_focus(true),
        [ImfCall::FocusIn, ImfCall::CursorPosition]
    );
    assert!(focus.editable(), "the field keeps its focus");
}

#[test]
fn ecore_modifiers_and_locks_map_to_imf() {
    assert_eq!(imf_modifiers(0), 0);
    assert_eq!(imf_modifiers(0x0001), IMF_MODIFIER_SHIFT);
    assert_eq!(imf_modifiers(0x0002), IMF_MODIFIER_CTRL);
    assert_eq!(imf_modifiers(0x0004), IMF_MODIFIER_ALT);
    assert_eq!(imf_modifiers(0x0008), IMF_MODIFIER_WIN);
    assert_eq!(imf_modifiers(0x0400), IMF_MODIFIER_ALTGR);
    assert_eq!(
        imf_modifiers(0x0003 | 0x0200),
        IMF_MODIFIER_SHIFT | IMF_MODIFIER_CTRL,
        "locks are not modifiers"
    );
    assert_eq!(imf_locks(0x0100), IMF_LOCK_NUM);
    assert_eq!(imf_locks(0x0200), IMF_LOCK_CAPS);
    assert_eq!(imf_locks(0x0080), IMF_LOCK_SCROLL);
    assert_eq!(imf_locks(0x0300), IMF_LOCK_NUM | IMF_LOCK_CAPS);
    assert_eq!(imf_locks(0x0001), 0);
}

#[test]
fn the_caret_location_is_in_physical_pixels() {
    let caret = gpui::Bounds {
        origin: gpui::point(gpui::px(100.25), gpui::px(40.)),
        size: gpui::size(gpui::px(1.), gpui::px(18.)),
    };
    assert_eq!(cursor_location(caret, 2.0), (201, 80, 2, 36));
    assert_eq!(cursor_location(caret, 1.0), (100, 40, 1, 18));
}

#[test]
fn moving_straight_to_another_field_restarts_the_context_after_the_frame() {
    let mut focus = ImeFocus::<u32>::default();
    assert!(!focus.input_handler(Some(1)), "no field focused yet");
    focus.state_changed(TextInputStateChange::FocusGained);
    assert!(
        !focus.input_handler(Some(1)),
        "the same field, frame after frame"
    );
    assert!(focus.settle().is_empty());
    assert!(
        focus.input_handler(Some(2)),
        "field 1 to field 2: post a settle"
    );
    assert!(!focus.input_handler(Some(3)), "one settle per frame");
    assert_eq!(
        focus.settle(),
        [
            ImfCall::BeginDiscard,
            ImfCall::Reset,
            ImfCall::FocusOut,
            ImfCall::EndDiscard,
            ImfCall::FocusIn,
            ImfCall::CursorPosition,
            ImfCall::ShowPanel,
        ]
    );
    assert!(focus.settle().is_empty(), "settled once");
    assert!(focus.editable());
}

#[test]
fn a_switch_that_ends_without_a_field_settles_to_nothing() {
    let mut focus = ImeFocus::<u32>::default();
    focus.input_handler(Some(1));
    focus.state_changed(TextInputStateChange::FocusGained);
    // GPUI installs a handler for a non-text element, then reports the loss, in one frame.
    assert!(focus.input_handler(Some(9)));
    assert_eq!(
        focus.state_changed(TextInputStateChange::FocusLost),
        [
            ImfCall::BeginDiscard,
            ImfCall::Reset,
            ImfCall::FocusOut,
            ImfCall::EndDiscard,
            ImfCall::HidePanel
        ]
    );
    assert!(focus.settle().is_empty(), "no keyboard flash");
    assert!(!focus.input_handler(None), "a handler naming no field");
    assert!(!focus.input_handler(Some(4)), "not editable: no switch");
}

/// An input method that, like Ecore_IMF's Korean engine, commits its preedit on reset.
struct Engine {
    preedit: String,
}

impl Engine {
    /// Performs `calls` as the shell does, delivering what the engine emits to `field` unless
    /// the focus state discards it.
    fn perform(&mut self, calls: &[ImfCall], focus: &mut ImeFocus<u32>, field: &mut Field) {
        for call in calls {
            focus.marker(*call);
            if *call == ImfCall::Reset && !self.preedit.is_empty() {
                for event in [
                    preedit("", 0),
                    ImeEvent::PreeditEnd,
                    ImeEvent::Commit(std::mem::take(&mut self.preedit)),
                ] {
                    if focus.accepts_events() {
                        apply(&event, field);
                    }
                }
            }
        }
    }
}

#[test]
fn a_switch_drops_the_old_field_s_composition_instead_of_committing_it_into_the_new_one() {
    let mut focus = ImeFocus::<u32>::default();
    let mut engine = Engine {
        preedit: String::new(),
    };
    let mut a = Field::new("x", 1);
    focus.input_handler(Some(1));
    focus.state_changed(TextInputStateChange::FocusGained);
    engine.preedit = "한".into();
    apply(&preedit("한", 1), &mut a);
    assert_eq!(a.text(), "x한");
    // Down to field B: GPUI installs B's handler; after the frame the context restarts on B.
    let mut b = Field::new("yz", 2);
    assert!(focus.input_handler(Some(2)));
    let calls = focus.settle();
    engine.perform(&calls, &mut focus, &mut b);
    assert_eq!(
        b.text(),
        "yz",
        "nothing of field A's composition reaches field B"
    );
    assert!(b.calls.is_empty());
    assert!(focus.accepts_events(), "events reach field B again");
    apply(&ImeEvent::Commit("!".into()), &mut b);
    assert_eq!(b.text(), "yz!");
}

#[test]
fn focus_loss_drops_the_composition_and_pause_keeps_it() {
    let mut focus = ImeFocus::<u32>::default();
    focus.state_changed(TextInputStateChange::FocusGained);
    let mut engine = Engine {
        preedit: "ㅎ".into(),
    };
    let mut other = Field::new("", 0);
    let lost = focus.state_changed(TextInputStateChange::FocusLost);
    engine.perform(&lost, &mut focus, &mut other);
    assert!(other.calls.is_empty(), "a non-text element gets no text");
    assert!(focus.accepts_events());
    // A pause keeps the field: its composition is committed into it.
    focus.state_changed(TextInputStateChange::FocusGained);
    engine.preedit = "한".into();
    let mut field = Field::new("", 0);
    apply(&preedit("한", 1), &mut field);
    let paused = focus.window_focus(false);
    engine.perform(&paused, &mut focus, &mut field);
    assert_eq!(field.text(), "한");
    assert_eq!(field.marked, None, "committed, not left marked");
}
