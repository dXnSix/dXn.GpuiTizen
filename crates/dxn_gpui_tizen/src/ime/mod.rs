//! Text input through Ecore_IMF (design `d16-ime`): the IME's events become GPUI input-handler
//! calls, GPUI's text-input state becomes IMF context calls, and Ecore key events become IMF key
//! events for the IME to filter first.
//!
//! - A preedit string becomes marked text (replacing the previous marked text, or the selection)
//!   with the IME's cursor as the selection inside it; an empty preedit removes the marked text.
//! - A commit replaces the marked text, or inserts at the selection.
//! - Delete-surrounding deletes characters around the cursor, by the IME's character offsets.
//! - Editable focus focuses the context and shows the on-screen keyboard; losing it resets the
//!   context, unfocuses it and hides the keyboard. Moving straight to another field restarts the
//!   context on it. Leaving a field drops what the reset commits (GPUI has installed the next
//!   handler by then), so a composition never moves into another field; the left field keeps its
//!   marked text, as GPUI no longer offers its handler. A pause keeps the field and commits. Only while a field is focused do keys go to the
//!   IME, and a key the IME consumes never reaches the key mapping.
//!
//! GPUI's ranges count UTF-16 code units; Ecore_IMF's offsets count characters. The shell
//! (`src/tizen/imf.rs`) owns the context and performs [`ImfCall`]s.

use std::ops::Range;

use gpui::{Bounds, Pixels, PlatformInputHandler, TextInputStateChange, UTF16Selection};

/// The text field the IME edits: GPUI's input handler, or a fake in tests.
pub trait TextInput {
    fn selection(&mut self) -> Option<UTF16Selection>;
    fn marked_range(&mut self) -> Option<Range<usize>>;
    fn text_for_range(&mut self, range: Range<usize>) -> Option<String>;
    /// Replaces `range` (`None`: the marked text, else the selection) and unmarks.
    fn replace_text(&mut self, range: Option<Range<usize>>, text: &str);
    /// Replaces `range` (`None` as above) with `text`, marks it, and selects `selected` inside it.
    fn replace_and_mark_text(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    );
}

impl TextInput for PlatformInputHandler {
    fn selection(&mut self) -> Option<UTF16Selection> {
        self.selected_text_range(false)
    }

    fn marked_range(&mut self) -> Option<Range<usize>> {
        self.marked_text_range()
    }

    fn text_for_range(&mut self, range: Range<usize>) -> Option<String> {
        PlatformInputHandler::text_for_range(self, range, &mut None)
    }

    fn replace_text(&mut self, range: Option<Range<usize>>, text: &str) {
        self.replace_text_in_range(range, text);
    }

    fn replace_and_mark_text(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        self.replace_and_mark_text_in_range(range, text, selected);
    }
}

/// An Ecore_IMF context event (`ECORE_IMF_CALLBACK_*`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImeEvent {
    PreeditStart,
    /// The whole preedit string, with the IME's cursor in characters.
    PreeditChanged {
        text: String,
        cursor: usize,
    },
    PreeditEnd,
    Commit(String),
    /// Delete `n_chars` characters starting `offset` characters from the cursor (negative:
    /// before it).
    DeleteSurrounding {
        offset: i32,
        n_chars: i32,
    },
}

/// UTF-16 length of the first `chars` characters of `text` (all of it if shorter).
fn utf16_len_of_chars(text: &str, chars: usize) -> usize {
    text.chars().take(chars).map(char::len_utf16).sum()
}

/// The cursor: the selection's head, in UTF-16.
fn cursor(selection: &UTF16Selection) -> usize {
    if selection.reversed {
        selection.range.start
    } else {
        selection.range.end
    }
}

/// Applies an IME event to the field.
pub fn apply(event: &ImeEvent, input: &mut impl TextInput) {
    match event {
        ImeEvent::PreeditStart => {}
        ImeEvent::PreeditChanged { text, cursor } => {
            let marked = input.marked_range();
            if text.is_empty() {
                if marked.is_some() {
                    input.replace_text(marked, "");
                }
            } else {
                let at = utf16_len_of_chars(text, *cursor);
                input.replace_and_mark_text(marked, text, Some(at..at));
            }
        }
        ImeEvent::PreeditEnd => {
            // A preedit that ended without a commit (cancelled) leaves nothing behind.
            let marked = input.marked_range();
            if marked.is_some() {
                input.replace_text(marked, "");
            }
        }
        ImeEvent::Commit(text) => {
            let marked = input.marked_range();
            input.replace_text(marked, text);
        }
        ImeEvent::DeleteSurrounding { offset, n_chars } => {
            if let Some(range) = surrounding_range(input, *offset, *n_chars) {
                input.replace_text(Some(range), "");
            }
        }
    }
}

/// The UTF-16 range `n_chars` characters long starting `offset` characters from the cursor,
/// clamped to the text.
fn surrounding_range(
    input: &mut impl TextInput,
    offset: i32,
    n_chars: i32,
) -> Option<Range<usize>> {
    if n_chars <= 0 {
        return None;
    }
    let cursor = cursor(&input.selection()?);
    let start = i64::from(offset);
    let end = start + i64::from(n_chars);
    let before = if start < 0 {
        input.text_for_range(0..cursor).unwrap_or_default()
    } else {
        String::new()
    };
    // At most two UTF-16 units per character.
    let after_chars = end.max(0) as usize;
    let after = if after_chars > 0 {
        input
            .text_for_range(cursor..cursor + 2 * after_chars)
            .unwrap_or_default()
    } else {
        String::new()
    };
    let at = |chars: i64| -> usize {
        if chars < 0 {
            let back: usize = before
                .chars()
                .rev()
                .take(chars.unsigned_abs() as usize)
                .map(char::len_utf16)
                .sum();
            cursor - back.min(cursor)
        } else {
            cursor + utf16_len_of_chars(&after, chars as usize)
        }
    };
    let (start, end) = (at(start), at(end));
    (start < end).then_some(start..end)
}

/// The text around the cursor for the IME (`retrieve_surrounding`): the text before the cursor
/// and up to `after` UTF-16 units after it, with the cursor in characters.
pub fn surrounding(input: &mut impl TextInput, after: usize) -> Option<(String, usize)> {
    let cursor = cursor(&input.selection()?);
    let mut text = input.text_for_range(0..cursor).unwrap_or_default();
    let chars = text.chars().count();
    if after > 0 {
        text.push_str(
            &input
                .text_for_range(cursor..cursor + after)
                .unwrap_or_default(),
        );
    }
    Some((text, chars))
}

/// The cursor in characters, for `ecore_imf_context_cursor_position_set`.
pub fn cursor_chars(input: &mut impl TextInput) -> Option<usize> {
    let cursor = cursor(&input.selection()?);
    Some(input.text_for_range(0..cursor)?.chars().count())
}

/// The caret's bounds (GPUI's logical pixels) in physical pixels, as
/// `ecore_imf_context_cursor_location_set` takes them: `(x, y, w, h)`.
pub fn cursor_location(bounds: Bounds<Pixels>, scale: f32) -> (i32, i32, i32, i32) {
    let physical = |value: Pixels| (f32::from(value) * scale).round() as i32;
    (
        physical(bounds.origin.x),
        physical(bounds.origin.y),
        physical(bounds.size.width),
        physical(bounds.size.height),
    )
}

/// A call on the window's IMF context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImfCall {
    /// From here until [`ImfCall::EndDiscard`], what the IME emits is dropped: leaving a field
    /// resets the context, and a reset commits the pending preedit (`Ecore_IMF.h`), which would
    /// otherwise land in whichever field GPUI has installed by now.
    BeginDiscard,
    EndDiscard,
    FocusIn,
    FocusOut,
    Reset,
    /// `ecore_imf_context_cursor_position_set` with [`cursor_chars`].
    CursorPosition,
    ShowPanel,
    HidePanel,
}

/// Whether a text field has GPUI's focus, which field it is, and what that asks of the IMF
/// context. `K` identifies a field: GPUI's `FocusId` (from `PlatformInputHandler::focus_id`).
///
/// GPUI reports `FocusGained` and `FocusLost` only between "no text field" and "a text field";
/// moving straight from one field to another installs another field's input handler and says
/// nothing else. A new field while one is editable is therefore a switch: once GPUI's frame is
/// done ([`ImeFocus::settle`]), and only if a field is still focused, the context drops the old
/// field's composition and starts over on the new one.
#[derive(Clone, Copy, Debug)]
pub struct ImeFocus<K = gpui::FocusId> {
    editable: bool,
    discarding: bool,
    field: Option<K>,
    switch_pending: bool,
}

impl<K> Default for ImeFocus<K> {
    fn default() -> Self {
        Self {
            editable: false,
            discarding: false,
            field: None,
            switch_pending: false,
        }
    }
}

impl<K: Copy + PartialEq> ImeFocus<K> {
    /// Whether keys go to the IME first.
    pub fn editable(&self) -> bool {
        self.editable
    }

    /// Whether IME events reach the field now (`false` between [`ImfCall::BeginDiscard`] and
    /// [`ImfCall::EndDiscard`]).
    pub fn accepts_events(&self) -> bool {
        !self.discarding
    }

    /// Performs the markers [`ImfCall::BeginDiscard`] and [`ImfCall::EndDiscard`]; other calls are
    /// the context's.
    pub fn marker(&mut self, call: ImfCall) {
        match call {
            ImfCall::BeginDiscard => self.discarding = true,
            ImfCall::EndDiscard => self.discarding = false,
            _ => {}
        }
    }

    /// GPUI's `text_input_state_changed`.
    pub fn state_changed(&mut self, change: TextInputStateChange) -> Vec<ImfCall> {
        match change {
            TextInputStateChange::FocusGained if !self.editable => {
                self.editable = true;
                self.switch_pending = false;
                vec![
                    ImfCall::FocusIn,
                    ImfCall::CursorPosition,
                    ImfCall::ShowPanel,
                ]
            }
            TextInputStateChange::FocusLost if self.editable => {
                self.editable = false;
                self.switch_pending = false;
                vec![
                    ImfCall::BeginDiscard,
                    ImfCall::Reset,
                    ImfCall::FocusOut,
                    ImfCall::EndDiscard,
                    ImfCall::HidePanel,
                ]
            }
            // Edits, including the IME's own, keep the composition: only the cursor moves.
            TextInputStateChange::SelectionChanged | TextInputStateChange::ContentChanged
                if self.editable =>
            {
                vec![ImfCall::CursorPosition]
            }
            _ => Vec::new(),
        }
    }

    /// GPUI installed an input handler for `field` (every frame; `None` when it names no field).
    /// Returns `true` when the caller posts [`ImeFocus::settle`] for after the frame.
    pub fn input_handler(&mut self, field: Option<K>) -> bool {
        let previous = std::mem::replace(&mut self.field, field);
        let switched = self.editable && previous.is_some() && field.is_some() && previous != field;
        if switched && !self.switch_pending {
            self.switch_pending = true;
            return true;
        }
        false
    }

    /// After the frame that switched fields: the calls that move the context to the new field,
    /// unless no field is focused any more (`FocusLost` came in the same frame).
    pub fn settle(&mut self) -> Vec<ImfCall> {
        if std::mem::take(&mut self.switch_pending) && self.editable {
            vec![
                ImfCall::BeginDiscard,
                ImfCall::Reset,
                ImfCall::FocusOut,
                ImfCall::EndDiscard,
                ImfCall::FocusIn,
                ImfCall::CursorPosition,
                ImfCall::ShowPanel,
            ]
        } else {
            Vec::new()
        }
    }

    /// GPUI asked for the on-screen keyboard (`show_soft_keyboard`).
    pub fn show_keyboard(&self) -> Vec<ImfCall> {
        if self.editable {
            vec![ImfCall::ShowPanel]
        } else {
            Vec::new()
        }
    }

    /// GPUI asked to dismiss the on-screen keyboard (`hide_soft_keyboard`).
    pub fn hide_keyboard(&self) -> Vec<ImfCall> {
        if self.editable {
            vec![ImfCall::HidePanel]
        } else {
            Vec::new()
        }
    }

    /// The window lost the system's focus: the context goes with it, the field stays editable.
    pub fn window_focus(&self, active: bool) -> Vec<ImfCall> {
        match (self.editable, active) {
            (true, true) => vec![ImfCall::FocusIn, ImfCall::CursorPosition],
            (true, false) => vec![ImfCall::Reset, ImfCall::FocusOut],
            _ => Vec::new(),
        }
    }
}

/// `ECORE_EVENT_MODIFIER_*` and `ECORE_EVENT_LOCK_*` (`Ecore_Input.h`).
const ECORE_SHIFT: u32 = 0x0001;
const ECORE_CTRL: u32 = 0x0002;
const ECORE_ALT: u32 = 0x0004;
const ECORE_WIN: u32 = 0x0008;
const ECORE_LOCK_SCROLL: u32 = 0x0080;
const ECORE_LOCK_NUM: u32 = 0x0100;
const ECORE_LOCK_CAPS: u32 = 0x0200;
const ECORE_ALTGR: u32 = 0x0400;

/// `Ecore_IMF_Keyboard_Modifiers` (`Ecore_IMF.h`).
pub const IMF_MODIFIER_CTRL: u32 = 1 << 0;
pub const IMF_MODIFIER_ALT: u32 = 1 << 1;
pub const IMF_MODIFIER_SHIFT: u32 = 1 << 2;
pub const IMF_MODIFIER_WIN: u32 = 1 << 3;
pub const IMF_MODIFIER_ALTGR: u32 = 1 << 4;
/// `Ecore_IMF_Keyboard_Locks` (`Ecore_IMF.h`).
pub const IMF_LOCK_NUM: u32 = 1 << 0;
pub const IMF_LOCK_CAPS: u32 = 1 << 1;
pub const IMF_LOCK_SCROLL: u32 = 1 << 2;

fn map_bits(bits: u32, table: &[(u32, u32)]) -> u32 {
    table
        .iter()
        .filter(|(from, _)| bits & from == *from)
        .fold(0, |acc, (_, to)| acc | to)
}

/// An Ecore key event's modifiers as IMF modifiers.
pub fn imf_modifiers(ecore: u32) -> u32 {
    map_bits(
        ecore,
        &[
            (ECORE_CTRL, IMF_MODIFIER_CTRL),
            (ECORE_ALT, IMF_MODIFIER_ALT),
            (ECORE_SHIFT, IMF_MODIFIER_SHIFT),
            (ECORE_WIN, IMF_MODIFIER_WIN),
            (ECORE_ALTGR, IMF_MODIFIER_ALTGR),
        ],
    )
}

/// An Ecore key event's locks as IMF locks.
pub fn imf_locks(ecore: u32) -> u32 {
    map_bits(
        ecore,
        &[
            (ECORE_LOCK_NUM, IMF_LOCK_NUM),
            (ECORE_LOCK_CAPS, IMF_LOCK_CAPS),
            (ECORE_LOCK_SCROLL, IMF_LOCK_SCROLL),
        ],
    )
}

#[cfg(test)]
mod tests;
