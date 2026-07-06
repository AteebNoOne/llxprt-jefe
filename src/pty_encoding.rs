//! PTY input encoding: converts key events and mouse events to raw bytes for
//! terminal passthrough.

use iocraft::prelude::{KeyCode, KeyEvent, KeyModifiers};

use jefe::input::InputMode;

pub fn ctrl_char_to_byte(c: char) -> Option<u8> {
    let c = c.to_ascii_lowercase();
    match c {
        '@' | ' ' | '2' => Some(0x00),
        '[' | '3' => Some(0x1b),
        '\\' | '4' => Some(0x1c),
        ']' | '5' => Some(0x1d),
        '^' | '6' => Some(0x1e),
        '_' | '7' | '/' => Some(0x1f),
        '?' | '8' => Some(0x7f),
        _ if c.is_ascii_alphabetic() => {
            let byte = (c as u8).wrapping_sub(b'a').wrapping_add(1);
            Some(byte)
        }
        _ if c.is_ascii() => Some((c as u8) & 0x1f),
        _ => None,
    }
}

fn modifiers_to_param(modifiers: KeyModifiers) -> Option<u8> {
    let shift = if modifiers.contains(KeyModifiers::SHIFT) { 1 } else { 0 };
    let alt = if modifiers.contains(KeyModifiers::ALT) { 2 } else { 0 };
    let ctrl = if modifiers.contains(KeyModifiers::CONTROL) { 4 } else { 0 };
    let meta = if modifiers.contains(KeyModifiers::META) || modifiers.contains(KeyModifiers::SUPER) { 8 } else { 0 };
    let val = 1 + shift + alt + ctrl + meta;
    if val > 1 {
        Some(val)
    } else {
        None
    }
}

fn function_key_to_bytes(n: u8, modifier: Option<u8>) -> Option<Vec<u8>> {
    if let Some(param) = modifier {
        Some(match n {
            1 => format!("\x1b[1;{param}P").into_bytes(),
            2 => format!("\x1b[1;{param}Q").into_bytes(),
            3 => format!("\x1b[1;{param}R").into_bytes(),
            4 => format!("\x1b[1;{param}S").into_bytes(),
            5 => format!("\x1b[15;{param}~").into_bytes(),
            6 => format!("\x1b[17;{param}~").into_bytes(),
            7 => format!("\x1b[18;{param}~").into_bytes(),
            8 => format!("\x1b[19;{param}~").into_bytes(),
            9 => format!("\x1b[20;{param}~").into_bytes(),
            10 => format!("\x1b[21;{param}~").into_bytes(),
            11 => format!("\x1b[23;{param}~").into_bytes(),
            12 => format!("\x1b[24;{param}~").into_bytes(),
            _ => return None,
        })
    } else {
        Some(match n {
            1 => b"\x1bOP".to_vec(),
            2 => b"\x1bOQ".to_vec(),
            3 => b"\x1bOR".to_vec(),
            4 => b"\x1bOS".to_vec(),
            5 => b"\x1b[15~".to_vec(),
            6 => b"\x1b[17~".to_vec(),
            7 => b"\x1b[18~".to_vec(),
            8 => b"\x1b[19~".to_vec(),
            9 => b"\x1b[20~".to_vec(),
            10 => b"\x1b[21~".to_vec(),
            11 => b"\x1b[23~".to_vec(),
            12 => b"\x1b[24~".to_vec(),
            _ => return None,
        })
    }
}

/// Convert a key event to raw bytes for PTY input.
///
/// When `passthrough_enter` is true, Enter maps directly to CR regardless of
/// modifiers, so terminal-focus mode stays close to raw passthrough.
pub fn key_to_bytes(key: &KeyEvent, passthrough_enter: bool) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    let mut alt_encoded = false;

    let mut out = match key.code {
        KeyCode::Char(c) if ctrl => {
            let byte = ctrl_char_to_byte(c)?;
            vec![byte]
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            s.as_bytes().to_vec()
        }
        KeyCode::Enter => {
            if passthrough_enter {
                vec![b'\r']
            } else if shift {
                // llxprt handles multiline Enter via Shift+Return key state and
                // also via VSCode fallback sequence backslash+carriage-return.
                // The fallback survives tmux attach paths more reliably.
                alt_encoded = alt;
                if alt {
                    b"\\\x1b\r".to_vec()
                } else {
                    b"\\\r".to_vec()
                }
            } else if ctrl {
                // llxprt accepts Ctrl+J as newline.
                vec![b'\n']
            } else {
                vec![b'\r']
            }
        }
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[1;{param}A").into_bytes()
            } else {
                b"\x1b[A".to_vec()
            }
        }
        KeyCode::Down => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[1;{param}B").into_bytes()
            } else {
                b"\x1b[B".to_vec()
            }
        }
        KeyCode::Right => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[1;{param}C").into_bytes()
            } else {
                b"\x1b[C".to_vec()
            }
        }
        KeyCode::Left => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[1;{param}D").into_bytes()
            } else {
                b"\x1b[D".to_vec()
            }
        }
        KeyCode::Home => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[1;{param}H").into_bytes()
            } else {
                b"\x1b[H".to_vec()
            }
        }
        KeyCode::End => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[1;{param}F").into_bytes()
            } else {
                b"\x1b[F".to_vec()
            }
        }
        KeyCode::PageUp => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[5;{param}~").into_bytes()
            } else {
                b"\x1b[5~".to_vec()
            }
        }
        KeyCode::PageDown => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[6;{param}~").into_bytes()
            } else {
                b"\x1b[6~".to_vec()
            }
        }
        KeyCode::Delete => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[3;{param}~").into_bytes()
            } else {
                b"\x1b[3~".to_vec()
            }
        }
        KeyCode::Insert => {
            if let Some(param) = modifiers_to_param(key.modifiers) {
                alt_encoded = true;
                format!("\x1b[2;{param}~").into_bytes()
            } else {
                b"\x1b[2~".to_vec()
            }
        }
        KeyCode::F(n) => {
            let param = modifiers_to_param(key.modifiers);
            if param.is_some() {
                alt_encoded = true;
            }
            function_key_to_bytes(n, param)?
        }
        _ => return None,
    };

    if alt && !alt_encoded {
        let mut prefixed = Vec::with_capacity(out.len() + 1);
        prefixed.push(0x1b);
        prefixed.extend_from_slice(&out);
        out = prefixed;
    }

    Some(out)
}

pub fn should_suppress_synthetic_enter(armed: bool, key_event: &KeyEvent) -> bool {
    armed && key_event.code == KeyCode::Enter
}

pub fn should_disarm_paste_enter_suppression(armed: bool, key_event: &KeyEvent) -> bool {
    armed && key_event.code != KeyCode::Enter
}

pub fn should_arm_paste_enter_suppression(key_event: &KeyEvent, input_mode: InputMode) -> bool {
    input_mode == InputMode::TerminalCapture
        && key_event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER | KeyModifiers::META)
        && matches!(key_event.code, KeyCode::Char('v' | 'V'))
}

/// Convert a fullscreen mouse event into xterm SGR mouse reporting bytes.
pub fn mouse_event_to_bytes(event: &iocraft::FullscreenMouseEvent) -> Option<Vec<u8>> {
    use iocraft::MouseEventKind;

    // Hold Shift for host-side selection/copy gestures.
    // This mirrors typical terminal behavior where Shift bypasses app mouse reporting.
    if event.modifiers.contains(iocraft::KeyModifiers::SHIFT) {
        return None;
    }

    let (cb, release) = match event.kind {
        MouseEventKind::Down(button) => {
            let code = match button {
                crossterm::event::MouseButton::Left => 0,
                crossterm::event::MouseButton::Middle => 1,
                crossterm::event::MouseButton::Right => 2,
            };
            (code, false)
        }
        MouseEventKind::Up(button) => {
            let code = match button {
                crossterm::event::MouseButton::Left => 0,
                crossterm::event::MouseButton::Middle => 1,
                crossterm::event::MouseButton::Right => 2,
            };
            (code, true)
        }
        MouseEventKind::Drag(button) => {
            let base = match button {
                crossterm::event::MouseButton::Left => 0,
                crossterm::event::MouseButton::Middle => 1,
                crossterm::event::MouseButton::Right => 2,
            };
            (base + 32, false)
        }
        MouseEventKind::Moved => return None,
        MouseEventKind::ScrollDown => (65, false),
        MouseEventKind::ScrollUp => (64, false),
        MouseEventKind::ScrollLeft => (66, false),
        MouseEventKind::ScrollRight => (67, false),
    };

    let mut cb_with_mods = cb;
    if event.modifiers.contains(iocraft::KeyModifiers::ALT) {
        cb_with_mods += 8;
    }
    if event.modifiers.contains(iocraft::KeyModifiers::CONTROL) {
        cb_with_mods += 16;
    }

    let cx = event.column.saturating_add(1);
    let cy = event.row.saturating_add(1);
    let suffix = if release { 'm' } else { 'M' };
    let seq = format!("\x1b[<{cb_with_mods};{cx};{cy}{suffix}");
    Some(seq.into_bytes())
}

#[cfg(test)]
mod key_tests {
    use super::{
        ctrl_char_to_byte, key_to_bytes, should_arm_paste_enter_suppression,
        should_disarm_paste_enter_suppression, should_suppress_synthetic_enter,
    };
    use iocraft::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use jefe::input::InputMode;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        let mut event = KeyEvent::new(KeyEventKind::Press, code);
        event.modifiers = modifiers;
        event
    }

    #[test]
    fn plain_enter_maps_to_cr() {
        let key = key_event(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(key_to_bytes(&key, false), Some(vec![b'\r']));
    }

    #[test]
    fn shift_enter_maps_to_backslash_cr() {
        let key = key_event(KeyCode::Enter, KeyModifiers::SHIFT);
        assert_eq!(key_to_bytes(&key, false), Some(b"\\\r".to_vec()));
    }

    #[test]
    fn synthetic_enter_is_only_suppressed_when_armed() {
        let enter = key_event(KeyCode::Enter, KeyModifiers::NONE);
        assert!(should_suppress_synthetic_enter(true, &enter));
        assert!(!should_suppress_synthetic_enter(false, &enter));
    }

    #[test]
    fn non_enter_key_disarms_paste_suppression_when_armed() {
        let key = key_event(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(should_disarm_paste_enter_suppression(true, &key));
        assert!(!should_disarm_paste_enter_suppression(false, &key));

        let enter = key_event(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!should_disarm_paste_enter_suppression(true, &enter));
    }

    #[test]
    fn paste_shortcut_arming_only_applies_in_terminal_capture() {
        let ctrl_v = key_event(KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert!(should_arm_paste_enter_suppression(
            &ctrl_v,
            InputMode::TerminalCapture
        ));
        assert!(!should_arm_paste_enter_suppression(
            &ctrl_v,
            InputMode::Normal
        ));

        let cmd_v = key_event(KeyCode::Char('v'), KeyModifiers::SUPER);
        assert!(should_arm_paste_enter_suppression(
            &cmd_v,
            InputMode::TerminalCapture
        ));

        let meta_v = key_event(KeyCode::Char('v'), KeyModifiers::META);
        assert!(should_arm_paste_enter_suppression(
            &meta_v,
            InputMode::TerminalCapture
        ));

        let alt_v = key_event(KeyCode::Char('v'), KeyModifiers::ALT);
        assert!(!should_arm_paste_enter_suppression(
            &alt_v,
            InputMode::TerminalCapture
        ));

        let plain_v = key_event(KeyCode::Char('v'), KeyModifiers::NONE);
        assert!(!should_arm_paste_enter_suppression(
            &plain_v,
            InputMode::TerminalCapture
        ));
    }

    #[test]
    fn passthrough_enter_keeps_cr_for_common_newline_modifiers() {
        let plain_enter = key_event(KeyCode::Enter, KeyModifiers::NONE);
        let shift_enter = key_event(KeyCode::Enter, KeyModifiers::SHIFT);
        let ctrl_enter = key_event(KeyCode::Enter, KeyModifiers::CONTROL);

        assert_eq!(key_to_bytes(&plain_enter, true), Some(vec![b'\r']));
        assert_eq!(key_to_bytes(&shift_enter, true), Some(vec![b'\r']));
        assert_eq!(key_to_bytes(&ctrl_enter, true), Some(vec![b'\r']));
    }

    #[test]
    fn passthrough_enter_with_alt_preserves_escape_prefix() {
        let alt_enter = key_event(KeyCode::Enter, KeyModifiers::ALT);
        assert_eq!(key_to_bytes(&alt_enter, true), Some(vec![0x1b, b'\r']));
    }

    #[test]
    fn alt_char_prefixes_escape() {
        let alt_x = key_event(KeyCode::Char('x'), KeyModifiers::ALT);
        assert_eq!(key_to_bytes(&alt_x, false), Some(b"\x1bx".to_vec()));
    }

    #[test]
    fn alt_shift_enter_does_not_double_prefix_escape() {
        let key = key_event(KeyCode::Enter, KeyModifiers::ALT | KeyModifiers::SHIFT);
        assert_eq!(key_to_bytes(&key, false), Some(b"\\\x1b\r".to_vec()));
    }

    #[test]
    fn shift_alt_enter_maps_to_backslash_esc_cr() {
        let key = key_event(KeyCode::Enter, KeyModifiers::SHIFT | KeyModifiers::ALT);
        assert_eq!(key_to_bytes(&key, false), Some(b"\\\x1b\r".to_vec()));
    }

    #[test]
    fn ctrl_backslash_maps_to_fs() {
        let key = key_event(KeyCode::Char('\\'), KeyModifiers::CONTROL);
        assert_eq!(ctrl_char_to_byte('\\'), Some(0x1c));
        assert_eq!(key_to_bytes(&key, false), Some(vec![0x1c]));
    }

    #[test]
    fn ctrl_underscore_maps_to_us() {
        let key = key_event(KeyCode::Char('_'), KeyModifiers::CONTROL);
        assert_eq!(ctrl_char_to_byte('_'), Some(0x1f));
        assert_eq!(key_to_bytes(&key, false), Some(vec![0x1f]));
    }

    #[test]
    fn ctrl_enter_maps_to_lf() {
        let key = key_event(KeyCode::Enter, KeyModifiers::CONTROL);
        assert_eq!(key_to_bytes(&key, false), Some(vec![b'\n']));
    }

    #[test]
    fn function_keys_use_expected_xterm_sequences() {
        let f1 = key_event(KeyCode::F(1), KeyModifiers::NONE);
        let f2 = key_event(KeyCode::F(2), KeyModifiers::NONE);
        let f12 = key_event(KeyCode::F(12), KeyModifiers::NONE);
        let insert = key_event(KeyCode::Insert, KeyModifiers::NONE);

        assert_eq!(key_to_bytes(&f1, false), Some(b"\x1bOP".to_vec()));
        assert_eq!(key_to_bytes(&f2, false), Some(b"\x1bOQ".to_vec()));
        assert_eq!(key_to_bytes(&f12, false), Some(b"\x1b[24~".to_vec()));
        assert_ne!(key_to_bytes(&f2, false), key_to_bytes(&insert, false));
    }

    #[test]
    fn modified_arrow_keys_use_xterm_sequences() {
        let ctrl_up = key_event(KeyCode::Up, KeyModifiers::CONTROL);
        let alt_down = key_event(KeyCode::Down, KeyModifiers::ALT);
        let shift_right = key_event(KeyCode::Right, KeyModifiers::SHIFT);
        let ctrl_alt_left = key_event(KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::ALT);

        // ctrl parameter = 5
        assert_eq!(key_to_bytes(&ctrl_up, false), Some(b"\x1b[1;5A".to_vec()));
        // alt parameter = 3
        assert_eq!(key_to_bytes(&alt_down, false), Some(b"\x1b[1;3B".to_vec()));
        // shift parameter = 2
        assert_eq!(key_to_bytes(&shift_right, false), Some(b"\x1b[1;2C".to_vec()));
        // ctrl + alt parameter = 7
        assert_eq!(key_to_bytes(&ctrl_alt_left, false), Some(b"\x1b[1;7D".to_vec()));
    }

    #[test]
    fn modified_edit_keys_use_xterm_sequences() {
        let ctrl_pageup = key_event(KeyCode::PageUp, KeyModifiers::CONTROL);
        let alt_pagedown = key_event(KeyCode::PageDown, KeyModifiers::ALT);
        let shift_delete = key_event(KeyCode::Delete, KeyModifiers::SHIFT);
        let ctrl_alt_insert = key_event(KeyCode::Insert, KeyModifiers::CONTROL | KeyModifiers::ALT);
        let shift_home = key_event(KeyCode::Home, KeyModifiers::SHIFT);
        let ctrl_end = key_event(KeyCode::End, KeyModifiers::CONTROL);

        assert_eq!(key_to_bytes(&ctrl_pageup, false), Some(b"\x1b[5;5~".to_vec()));
        assert_eq!(key_to_bytes(&alt_pagedown, false), Some(b"\x1b[6;3~".to_vec()));
        assert_eq!(key_to_bytes(&shift_delete, false), Some(b"\x1b[3;2~".to_vec()));
        assert_eq!(key_to_bytes(&ctrl_alt_insert, false), Some(b"\x1b[2;7~".to_vec()));
        assert_eq!(key_to_bytes(&shift_home, false), Some(b"\x1b[1;2H".to_vec()));
        assert_eq!(key_to_bytes(&ctrl_end, false), Some(b"\x1b[1;5F".to_vec()));
    }

    #[test]
    fn modified_function_keys_use_xterm_sequences() {
        let ctrl_f1 = key_event(KeyCode::F(1), KeyModifiers::CONTROL);
        let alt_f5 = key_event(KeyCode::F(5), KeyModifiers::ALT);
        let ctrl_alt_f12 = key_event(KeyCode::F(12), KeyModifiers::CONTROL | KeyModifiers::ALT);

        assert_eq!(key_to_bytes(&ctrl_f1, false), Some(b"\x1b[1;5P".to_vec()));
        assert_eq!(key_to_bytes(&alt_f5, false), Some(b"\x1b[15;3~".to_vec()));
        assert_eq!(key_to_bytes(&ctrl_alt_f12, false), Some(b"\x1b[24;7~".to_vec()));
    }

    #[test]
    fn alt_encoding_is_consistent_and_not_double_encoded() {
        // Alt-up modified should be \x1b[1;3A, not double ESC-prefixed (e.g. not \x1b\x1b[1;3A)
        let alt_up = key_event(KeyCode::Up, KeyModifiers::ALT);
        assert_eq!(key_to_bytes(&alt_up, false), Some(b"\x1b[1;3A".to_vec()));

        // Alt-F1 modified should be \x1b[1;3P, not \x1b\x1b[1;3P
        let alt_f1 = key_event(KeyCode::F(1), KeyModifiers::ALT);
        assert_eq!(key_to_bytes(&alt_f1, false), Some(b"\x1b[1;3P".to_vec()));
    }
}

#[cfg(test)]
mod mouse_tests {
    use super::mouse_event_to_bytes;
    use crossterm::event::MouseButton;
    use iocraft::{FullscreenMouseEvent, KeyModifiers, MouseEventKind};

    #[test]
    fn shift_mouse_events_are_not_forwarded_to_pty() {
        let mut event = FullscreenMouseEvent::new(MouseEventKind::Down(MouseButton::Left), 9, 4);
        event.modifiers = KeyModifiers::SHIFT;
        assert_eq!(mouse_event_to_bytes(&event), None);
    }

    #[test]
    fn left_click_uses_sgr_press_encoding() {
        let event = FullscreenMouseEvent::new(MouseEventKind::Down(MouseButton::Left), 9, 4);
        assert_eq!(
            mouse_event_to_bytes(&event),
            Some(b"\x1b[<0;10;5M".to_vec())
        );
    }

    #[test]
    fn right_release_uses_sgr_release_suffix() {
        let event = FullscreenMouseEvent::new(MouseEventKind::Up(MouseButton::Right), 3, 7);
        assert_eq!(mouse_event_to_bytes(&event), Some(b"\x1b[<2;4;8m".to_vec()));
    }

    #[test]
    fn drag_with_alt_and_ctrl_sets_modifier_bits() {
        let mut event = FullscreenMouseEvent::new(MouseEventKind::Drag(MouseButton::Left), 0, 0);
        event.modifiers = KeyModifiers::ALT | KeyModifiers::CONTROL;
        assert_eq!(
            mouse_event_to_bytes(&event),
            Some(b"\x1b[<56;1;1M".to_vec())
        );
    }
}
