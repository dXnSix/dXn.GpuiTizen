//! Ecore key events to GPUI keystrokes, by the table of design `d14-keys`: TV remote keys get
//! their documented names, printable keys their character (with `key_char`), anything else its
//! key name lowercased. A key pressed again before it was released is a repeat: GPUI sees it as
//! held. Back is only a keystroke; the platform never exits on it.

use std::collections::HashSet;

use gpui::{Keystroke, Modifiers};

/// `ECORE_EVENT_MODIFIER_*` (`Ecore_Input.h`).
pub const MODIFIER_SHIFT: u32 = 0x0001;
pub const MODIFIER_CTRL: u32 = 0x0002;
pub const MODIFIER_ALT: u32 = 0x0004;
pub const MODIFIER_WIN: u32 = 0x0008;

/// Tizen key names with a documented GPUI name.
pub const NAMED_KEYS: &[(&str, &str)] = &[
    ("Up", "up"),
    ("Down", "down"),
    ("Left", "left"),
    ("Right", "right"),
    ("Return", "enter"),
    ("KP_Enter", "enter"),
    ("XF86Back", "back"),
    ("XF86Exit", "exit"),
    ("BackSpace", "backspace"),
    ("Escape", "escape"),
    ("XF86Red", "red"),
    ("XF86Green", "green"),
    ("XF86Yellow", "yellow"),
    ("XF86Blue", "blue"),
    ("XF86AudioPlay", "mediaplay"),
    ("XF86AudioPause", "mediapause"),
    ("XF86PlayBack", "mediaplaypause"),
    ("XF86AudioStop", "mediastop"),
    ("XF86AudioRewind", "mediarewind"),
    ("XF86AudioForward", "mediafastforward"),
    ("XF86AudioNext", "medianext"),
    ("XF86AudioPrev", "mediaprevious"),
    ("XF86RaiseChannel", "channelup"),
    ("XF86LowerChannel", "channeldown"),
];

/// A key event as Ecore delivers it (`Ecore_Event_Key`).
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent<'a> {
    /// `keyname`: `Up`, `XF86Back`, `a`, `space`, ...
    pub keyname: &'a str,
    /// `string`: the text the key produces, if any.
    pub string: Option<&'a str>,
    /// `modifiers`: `ECORE_EVENT_MODIFIER_*` bits.
    pub modifiers: u32,
}

/// GPUI's modifiers for Ecore's bits.
pub fn modifiers(bits: u32) -> Modifiers {
    Modifiers {
        control: bits & MODIFIER_CTRL != 0,
        alt: bits & MODIFIER_ALT != 0,
        shift: bits & MODIFIER_SHIFT != 0,
        platform: bits & MODIFIER_WIN != 0,
        ..Modifiers::default()
    }
}

fn named(keyname: &str) -> Option<&'static str> {
    NAMED_KEYS
        .iter()
        .find(|(tizen, _)| *tizen == keyname)
        .map(|(_, gpui)| *gpui)
}

/// The printable character a key produces, if it produces exactly one that is not a control.
fn printable(string: Option<&str>) -> Option<&str> {
    let string = string?;
    let mut chars = string.chars();
    let c = chars.next()?;
    (chars.next().is_none() && !c.is_control()).then_some(string)
}

/// The keystroke for a key event.
pub fn keystroke(event: &KeyEvent<'_>) -> Keystroke {
    let modifiers = modifiers(event.modifiers);
    if let Some(key) = named(event.keyname) {
        return Keystroke {
            modifiers,
            key: key.to_owned(),
            key_char: None,
        };
    }
    if let Some(text) = printable(event.string) {
        let key = if text == " " {
            "space".to_owned()
        } else {
            text.to_lowercase()
        };
        return Keystroke {
            modifiers,
            key,
            key_char: Some(text.to_owned()),
        };
    }
    Keystroke {
        modifiers,
        key: event.keyname.to_lowercase(),
        key_char: None,
    }
}

/// Which keys are down, to tell a repeat from a press.
#[derive(Debug, Default)]
pub struct HeldKeys {
    down: HashSet<String>,
}

impl HeldKeys {
    /// A key went down: its keystroke, and whether it was already held (a repeat).
    pub fn key_down(&mut self, event: &KeyEvent<'_>) -> (Keystroke, bool) {
        let held = !self.down.insert(event.keyname.to_owned());
        (keystroke(event), held)
    }

    /// A key went up: its keystroke.
    pub fn key_up(&mut self, event: &KeyEvent<'_>) -> Keystroke {
        self.down.remove(event.keyname);
        keystroke(event)
    }

    /// The window lost focus: no key is held any more.
    pub fn clear(&mut self) {
        self.down.clear();
    }
}

#[cfg(test)]
mod tests;
