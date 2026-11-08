use super::*;

fn key(keyname: &str) -> Keystroke {
    keystroke(&KeyEvent {
        keyname,
        string: None,
        modifiers: 0,
    })
}

#[test]
fn the_whole_named_table() {
    let expected = [
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
    assert_eq!(expected.len(), NAMED_KEYS.len());
    for (tizen, gpui) in expected {
        let stroke = key(tizen);
        assert_eq!(stroke.key, gpui, "{tizen}");
        assert_eq!(stroke.key_char, None, "{tizen}");
        assert_eq!(stroke.modifiers, Modifiers::default());
    }
    // Return produces "\r" as its string; it is still `enter`, without a key_char.
    let enter = keystroke(&KeyEvent {
        keyname: "Return",
        string: Some("\r"),
        modifiers: 0,
    });
    assert_eq!((enter.key.as_str(), enter.key_char), ("enter", None));
}

#[test]
fn digits_and_printable_keys_carry_their_character() {
    for digit in '0'..='9' {
        let s = digit.to_string();
        let stroke = keystroke(&KeyEvent {
            keyname: &s,
            string: Some(&s),
            modifiers: 0,
        });
        assert_eq!(stroke.key, s);
        assert_eq!(stroke.key_char.as_deref(), Some(s.as_str()));
    }
    let a = keystroke(&KeyEvent {
        keyname: "a",
        string: Some("a"),
        modifiers: 0,
    });
    assert_eq!((a.key.as_str(), a.key_char.as_deref()), ("a", Some("a")));
    let shifted = keystroke(&KeyEvent {
        keyname: "A",
        string: Some("A"),
        modifiers: MODIFIER_SHIFT,
    });
    assert_eq!(
        (shifted.key.as_str(), shifted.key_char.as_deref()),
        ("a", Some("A"))
    );
    assert!(shifted.modifiers.shift);
    let space = keystroke(&KeyEvent {
        keyname: "space",
        string: Some(" "),
        modifiers: 0,
    });
    assert_eq!(
        (space.key.as_str(), space.key_char.as_deref()),
        ("space", Some(" "))
    );
    let umlaut = keystroke(&KeyEvent {
        keyname: "udiaeresis",
        string: Some("ü"),
        modifiers: 0,
    });
    assert_eq!(
        (umlaut.key.as_str(), umlaut.key_char.as_deref()),
        ("ü", Some("ü"))
    );
}

#[test]
fn unknown_key_names_pass_through_lowercased() {
    assert_eq!(key("XF86Info").key, "xf86info");
    assert_eq!(key("XF86Menu").key, "xf86menu");
    assert_eq!(key("Tab").key, "tab");
    // A control string is not a key_char.
    let tab = keystroke(&KeyEvent {
        keyname: "Tab",
        string: Some("\t"),
        modifiers: 0,
    });
    assert_eq!((tab.key.as_str(), tab.key_char), ("tab", None));
}

#[test]
fn modifiers_map_from_ecore_bits() {
    let all = modifiers(MODIFIER_SHIFT | MODIFIER_CTRL | MODIFIER_ALT | MODIFIER_WIN);
    assert!(all.shift && all.control && all.alt && all.platform);
    assert_eq!(modifiers(0), Modifiers::default());
    let ctrl_up = keystroke(&KeyEvent {
        keyname: "Up",
        string: None,
        modifiers: MODIFIER_CTRL,
    });
    assert!(ctrl_up.modifiers.control && !ctrl_up.modifiers.shift);
}

#[test]
fn a_repeat_is_held_until_released() {
    let mut held = HeldKeys::default();
    let down = KeyEvent {
        keyname: "Down",
        string: None,
        modifiers: 0,
    };
    assert!(!held.key_down(&down).1, "first press");
    assert!(held.key_down(&down).1, "repeat");
    assert!(held.key_down(&down).1, "repeat");
    assert_eq!(held.key_up(&down).key, "down");
    assert!(!held.key_down(&down).1, "pressed again after release");
    let up = KeyEvent {
        keyname: "Up",
        string: None,
        modifiers: 0,
    };
    assert!(!held.key_down(&up).1, "another key is not held");
    held.clear();
    assert!(!held.key_down(&down).1, "focus loss releases everything");
}
