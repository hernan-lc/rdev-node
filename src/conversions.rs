use super::enums::*;
use super::events::*;
use rdev::{Button, Event, EventType, Key};

/// Normalizes a KeyCode to its modifier name if it's a modifier key
pub fn normalize_key_name(key_code: &KeyCode) -> String {
  key_code
    .as_modifier_name()
    .map(|s| s.to_string())
    .unwrap_or_else(|| key_code.as_lowercase())
}

/// Checks if a KeyCode is a modifier key
pub fn is_modifier_key(key_code: &KeyCode) -> bool {
  key_code.is_modifier()
}

/// Converts a string key representation to its KeyCode
/// Supports both single characters (e.g., "c", "1") and full key names (e.g., "KeyC", "Num1")
pub fn string_key_to_keycode(key: &str) -> Option<KeyCode> {
  match key.to_lowercase().as_str() {
    // Number keys
    "1" | "num1" => Some(KeyCode::Num1),
    "2" | "num2" => Some(KeyCode::Num2),
    "3" | "num3" => Some(KeyCode::Num3),
    "4" | "num4" => Some(KeyCode::Num4),
    "5" | "num5" => Some(KeyCode::Num5),
    "6" | "num6" => Some(KeyCode::Num6),
    "7" | "num7" => Some(KeyCode::Num7),
    "8" | "num8" => Some(KeyCode::Num8),
    "9" | "num9" => Some(KeyCode::Num9),
    "0" | "num0" => Some(KeyCode::Num0),
    // Letter keys
    "a" | "keya" => Some(KeyCode::KeyA),
    "b" | "keyb" => Some(KeyCode::KeyB),
    "c" | "keyc" => Some(KeyCode::KeyC),
    "d" | "keyd" => Some(KeyCode::KeyD),
    "e" | "keye" => Some(KeyCode::KeyE),
    "f" | "keyf" => Some(KeyCode::KeyF),
    "g" | "keyg" => Some(KeyCode::KeyG),
    "h" | "keyh" => Some(KeyCode::KeyH),
    "i" | "keyi" => Some(KeyCode::KeyI),
    "j" | "keyj" => Some(KeyCode::KeyJ),
    "k" | "keyk" => Some(KeyCode::KeyK),
    "l" | "keyl" => Some(KeyCode::KeyL),
    "m" | "keym" => Some(KeyCode::KeyM),
    "n" | "keyn" => Some(KeyCode::KeyN),
    "o" | "keyo" => Some(KeyCode::KeyO),
    "p" | "keyp" => Some(KeyCode::KeyP),
    "q" | "keyq" => Some(KeyCode::KeyQ),
    "r" | "keyr" => Some(KeyCode::KeyR),
    "s" | "keys" => Some(KeyCode::KeyS),
    "t" | "keyt" => Some(KeyCode::KeyT),
    "u" | "keyu" => Some(KeyCode::KeyU),
    "v" | "keyv" => Some(KeyCode::KeyV),
    "w" | "keyw" => Some(KeyCode::KeyW),
    "x" | "keyx" => Some(KeyCode::KeyX),
    "y" | "keyy" => Some(KeyCode::KeyY),
    "z" | "keyz" => Some(KeyCode::KeyZ),
    // Function keys
    "f1" => Some(KeyCode::F1),
    "f2" => Some(KeyCode::F2),
    "f3" => Some(KeyCode::F3),
    "f4" => Some(KeyCode::F4),
    "f5" => Some(KeyCode::F5),
    "f6" => Some(KeyCode::F6),
    "f7" => Some(KeyCode::F7),
    "f8" => Some(KeyCode::F8),
    "f9" => Some(KeyCode::F9),
    "f10" => Some(KeyCode::F10),
    "f11" => Some(KeyCode::F11),
    "f12" => Some(KeyCode::F12),
    // Special keys
    "escape" | "esc" => Some(KeyCode::Escape),
    "return" | "enter" => Some(KeyCode::Return),
    "space" => Some(KeyCode::Space),
    "tab" => Some(KeyCode::Tab),
    "backspace" => Some(KeyCode::Backspace),
    "delete" | "del" => Some(KeyCode::Delete),
    "insert" | "ins" => Some(KeyCode::Insert),
    "home" => Some(KeyCode::Home),
    "end" => Some(KeyCode::End),
    "pageup" => Some(KeyCode::PageUp),
    "pagedown" => Some(KeyCode::PageDown),
    "up" | "uparrow" => Some(KeyCode::UpArrow),
    "down" | "downarrow" => Some(KeyCode::DownArrow),
    "left" | "leftarrow" => Some(KeyCode::LeftArrow),
    "right" | "rightarrow" => Some(KeyCode::RightArrow),
    // Modifier keys
    "ctrl" | "control" | "controll" | "controlleft" => Some(KeyCode::ControlLeft),
    "ctrlr" | "controlr" | "controlright" => Some(KeyCode::ControlRight),
    "shift" | "shiftl" | "shiftleft" => Some(KeyCode::ShiftLeft),
    "shiftr" | "shiftright" => Some(KeyCode::ShiftRight),
    "alt" => Some(KeyCode::Alt),
    "altgr" => Some(KeyCode::AltGr),
    "meta" | "metal" | "metaleft" => Some(KeyCode::MetaLeft),
    "metar" | "metaright" => Some(KeyCode::MetaRight),
    // Numpad keys
    "kp0" | "numpad0" => Some(KeyCode::Kp0),
    "kp1" | "numpad1" => Some(KeyCode::Kp1),
    "kp2" | "numpad2" => Some(KeyCode::Kp2),
    "kp3" | "numpad3" => Some(KeyCode::Kp3),
    "kp4" | "numpad4" => Some(KeyCode::Kp4),
    "kp5" | "numpad5" => Some(KeyCode::Kp5),
    "kp6" | "numpad6" => Some(KeyCode::Kp6),
    "kp7" | "numpad7" => Some(KeyCode::Kp7),
    "kp8" | "numpad8" => Some(KeyCode::Kp8),
    "kp9" | "numpad9" => Some(KeyCode::Kp9),
    "kpreturn" | "kpenter" => Some(KeyCode::KpReturn),
    "kpminus" => Some(KeyCode::KpMinus),
    "kpplus" => Some(KeyCode::KpPlus),
    "kpmultiply" => Some(KeyCode::KpMultiply),
    "kpdivide" => Some(KeyCode::KpDivide),
    "kpdelete" | "kpdecimal" => Some(KeyCode::KpDelete),
    // Other keys
    "minus" | "dash" | "hyphen" => Some(KeyCode::Minus),
    "equal" | "equals" => Some(KeyCode::Equal),
    "comma" => Some(KeyCode::Comma),
    "dot" | "period" => Some(KeyCode::Dot),
    "slash" | "forwardslash" => Some(KeyCode::Slash),
    "backslash" => Some(KeyCode::BackSlash),
    "semicolon" | "semi" => Some(KeyCode::SemiColon),
    "quote" | "apostrophe" => Some(KeyCode::Quote),
    "backquote" | "grave" | "backtick" => Some(KeyCode::BackQuote),
    "leftbracket" | "lbracket" => Some(KeyCode::LeftBracket),
    "rightbracket" | "rbracket" => Some(KeyCode::RightBracket),
    "capslock" => Some(KeyCode::CapsLock),
    "numlock" => Some(KeyCode::NumLock),
    "scrolllock" => Some(KeyCode::ScrollLock),
    "printscreen" | "prtsc" => Some(KeyCode::PrintScreen),
    "pause" => Some(KeyCode::Pause),
    _ => None,
  }
}

// Helper functions to convert rdev types to our enums
pub fn button_to_type(btn: &Button) -> ButtonType {
  match btn {
    Button::Left => ButtonType::Left,
    Button::Right => ButtonType::Right,
    Button::Middle => ButtonType::Middle,
    Button::Unknown(_) => ButtonType::Unknown,
  }
}

pub fn type_to_button(btn: ButtonType) -> Button {
  match btn {
    ButtonType::Left => Button::Left,
    ButtonType::Right => Button::Right,
    ButtonType::Middle => Button::Middle,
    ButtonType::Unknown => Button::Unknown(0),
  }
}

pub fn key_to_type(key: &Key) -> KeyCode {
  match key {
    Key::Alt => KeyCode::Alt,
    Key::AltGr => KeyCode::AltGr,
    Key::Backspace => KeyCode::Backspace,
    Key::CapsLock => KeyCode::CapsLock,
    Key::ControlLeft => KeyCode::ControlLeft,
    Key::ControlRight => KeyCode::ControlRight,
    Key::Delete => KeyCode::Delete,
    Key::DownArrow => KeyCode::DownArrow,
    Key::End => KeyCode::End,
    Key::Escape => KeyCode::Escape,
    Key::F1 => KeyCode::F1,
    Key::F10 => KeyCode::F10,
    Key::F11 => KeyCode::F11,
    Key::F12 => KeyCode::F12,
    Key::F2 => KeyCode::F2,
    Key::F3 => KeyCode::F3,
    Key::F4 => KeyCode::F4,
    Key::F5 => KeyCode::F5,
    Key::F6 => KeyCode::F6,
    Key::F7 => KeyCode::F7,
    Key::F8 => KeyCode::F8,
    Key::F9 => KeyCode::F9,
    Key::Home => KeyCode::Home,
    Key::LeftArrow => KeyCode::LeftArrow,
    Key::MetaLeft => KeyCode::MetaLeft,
    Key::MetaRight => KeyCode::MetaRight,
    Key::NumLock => KeyCode::NumLock,
    Key::PageDown => KeyCode::PageDown,
    Key::PageUp => KeyCode::PageUp,
    Key::Return => KeyCode::Return,
    Key::RightArrow => KeyCode::RightArrow,
    Key::ShiftLeft => KeyCode::ShiftLeft,
    Key::ShiftRight => KeyCode::ShiftRight,
    Key::Space => KeyCode::Space,
    Key::Tab => KeyCode::Tab,
    Key::UpArrow => KeyCode::UpArrow,
    Key::PrintScreen => KeyCode::PrintScreen,
    Key::ScrollLock => KeyCode::ScrollLock,
    Key::Pause => KeyCode::Pause,
    Key::Insert => KeyCode::Insert,
    Key::BackQuote => KeyCode::BackQuote,
    Key::Num1 => KeyCode::Num1,
    Key::Num2 => KeyCode::Num2,
    Key::Num3 => KeyCode::Num3,
    Key::Num4 => KeyCode::Num4,
    Key::Num5 => KeyCode::Num5,
    Key::Num6 => KeyCode::Num6,
    Key::Num7 => KeyCode::Num7,
    Key::Num8 => KeyCode::Num8,
    Key::Num9 => KeyCode::Num9,
    Key::Num0 => KeyCode::Num0,
    Key::Minus => KeyCode::Minus,
    Key::Equal => KeyCode::Equal,
    Key::KeyQ => KeyCode::KeyQ,
    Key::KeyW => KeyCode::KeyW,
    Key::KeyE => KeyCode::KeyE,
    Key::KeyR => KeyCode::KeyR,
    Key::KeyT => KeyCode::KeyT,
    Key::KeyY => KeyCode::KeyY,
    Key::KeyU => KeyCode::KeyU,
    Key::KeyI => KeyCode::KeyI,
    Key::KeyO => KeyCode::KeyO,
    Key::KeyP => KeyCode::KeyP,
    Key::LeftBracket => KeyCode::LeftBracket,
    Key::RightBracket => KeyCode::RightBracket,
    Key::KeyA => KeyCode::KeyA,
    Key::KeyS => KeyCode::KeyS,
    Key::KeyD => KeyCode::KeyD,
    Key::KeyF => KeyCode::KeyF,
    Key::KeyG => KeyCode::KeyG,
    Key::KeyH => KeyCode::KeyH,
    Key::KeyJ => KeyCode::KeyJ,
    Key::KeyK => KeyCode::KeyK,
    Key::KeyL => KeyCode::KeyL,
    Key::SemiColon => KeyCode::SemiColon,
    Key::Quote => KeyCode::Quote,
    Key::BackSlash => KeyCode::BackSlash,
    Key::IntlBackslash => KeyCode::IntlBackslash,
    Key::KeyZ => KeyCode::KeyZ,
    Key::KeyX => KeyCode::KeyX,
    Key::KeyC => KeyCode::KeyC,
    Key::KeyV => KeyCode::KeyV,
    Key::KeyB => KeyCode::KeyB,
    Key::KeyN => KeyCode::KeyN,
    Key::KeyM => KeyCode::KeyM,
    Key::Comma => KeyCode::Comma,
    Key::Dot => KeyCode::Dot,
    Key::Slash => KeyCode::Slash,
    Key::KpReturn => KeyCode::KpReturn,
    Key::KpMinus => KeyCode::KpMinus,
    Key::KpPlus => KeyCode::KpPlus,
    Key::KpMultiply => KeyCode::KpMultiply,
    Key::KpDivide => KeyCode::KpDivide,
    Key::Kp0 => KeyCode::Kp0,
    Key::Kp1 => KeyCode::Kp1,
    Key::Kp2 => KeyCode::Kp2,
    Key::Kp3 => KeyCode::Kp3,
    Key::Kp4 => KeyCode::Kp4,
    Key::Kp5 => KeyCode::Kp5,
    Key::Kp6 => KeyCode::Kp6,
    Key::Kp7 => KeyCode::Kp7,
    Key::Kp8 => KeyCode::Kp8,
    Key::Kp9 => KeyCode::Kp9,
    Key::KpDelete => KeyCode::KpDelete,
    Key::Function => KeyCode::Function,
    Key::Unknown(_) => KeyCode::Unknown,
  }
}

pub fn type_to_key(key: KeyCode) -> Key {
  match key {
    KeyCode::Alt => Key::Alt,
    KeyCode::AltGr => Key::AltGr,
    KeyCode::Backspace => Key::Backspace,
    KeyCode::CapsLock => Key::CapsLock,
    KeyCode::ControlLeft => Key::ControlLeft,
    KeyCode::ControlRight => Key::ControlRight,
    KeyCode::Delete => Key::Delete,
    KeyCode::DownArrow => Key::DownArrow,
    KeyCode::End => Key::End,
    KeyCode::Escape => Key::Escape,
    KeyCode::F1 => Key::F1,
    KeyCode::F10 => Key::F10,
    KeyCode::F11 => Key::F11,
    KeyCode::F12 => Key::F12,
    KeyCode::F2 => Key::F2,
    KeyCode::F3 => Key::F3,
    KeyCode::F4 => Key::F4,
    KeyCode::F5 => Key::F5,
    KeyCode::F6 => Key::F6,
    KeyCode::F7 => Key::F7,
    KeyCode::F8 => Key::F8,
    KeyCode::F9 => Key::F9,
    KeyCode::Home => Key::Home,
    KeyCode::LeftArrow => Key::LeftArrow,
    KeyCode::MetaLeft => Key::MetaLeft,
    KeyCode::MetaRight => Key::MetaRight,
    KeyCode::NumLock => Key::NumLock,
    KeyCode::PageDown => Key::PageDown,
    KeyCode::PageUp => Key::PageUp,
    KeyCode::Return => Key::Return,
    KeyCode::RightArrow => Key::RightArrow,
    KeyCode::ShiftLeft => Key::ShiftLeft,
    KeyCode::ShiftRight => Key::ShiftRight,
    KeyCode::Space => Key::Space,
    KeyCode::Tab => Key::Tab,
    KeyCode::UpArrow => Key::UpArrow,
    KeyCode::PrintScreen => Key::PrintScreen,
    KeyCode::ScrollLock => Key::ScrollLock,
    KeyCode::Pause => Key::Pause,
    KeyCode::Insert => Key::Insert,
    KeyCode::BackQuote => Key::BackQuote,
    KeyCode::Num1 => Key::Num1,
    KeyCode::Num2 => Key::Num2,
    KeyCode::Num3 => Key::Num3,
    KeyCode::Num4 => Key::Num4,
    KeyCode::Num5 => Key::Num5,
    KeyCode::Num6 => Key::Num6,
    KeyCode::Num7 => Key::Num7,
    KeyCode::Num8 => Key::Num8,
    KeyCode::Num9 => Key::Num9,
    KeyCode::Num0 => Key::Num0,
    KeyCode::Minus => Key::Minus,
    KeyCode::Equal => Key::Equal,
    KeyCode::KeyQ => Key::KeyQ,
    KeyCode::KeyW => Key::KeyW,
    KeyCode::KeyE => Key::KeyE,
    KeyCode::KeyR => Key::KeyR,
    KeyCode::KeyT => Key::KeyT,
    KeyCode::KeyY => Key::KeyY,
    KeyCode::KeyU => Key::KeyU,
    KeyCode::KeyI => Key::KeyI,
    KeyCode::KeyO => Key::KeyO,
    KeyCode::KeyP => Key::KeyP,
    KeyCode::LeftBracket => Key::LeftBracket,
    KeyCode::RightBracket => Key::RightBracket,
    KeyCode::KeyA => Key::KeyA,
    KeyCode::KeyS => Key::KeyS,
    KeyCode::KeyD => Key::KeyD,
    KeyCode::KeyF => Key::KeyF,
    KeyCode::KeyG => Key::KeyG,
    KeyCode::KeyH => Key::KeyH,
    KeyCode::KeyJ => Key::KeyJ,
    KeyCode::KeyK => Key::KeyK,
    KeyCode::KeyL => Key::KeyL,
    KeyCode::SemiColon => Key::SemiColon,
    KeyCode::Quote => Key::Quote,
    KeyCode::BackSlash => Key::BackSlash,
    KeyCode::IntlBackslash => Key::IntlBackslash,
    KeyCode::KeyZ => Key::KeyZ,
    KeyCode::KeyX => Key::KeyX,
    KeyCode::KeyC => Key::KeyC,
    KeyCode::KeyV => Key::KeyV,
    KeyCode::KeyB => Key::KeyB,
    KeyCode::KeyN => Key::KeyN,
    KeyCode::KeyM => Key::KeyM,
    KeyCode::Comma => Key::Comma,
    KeyCode::Dot => Key::Dot,
    KeyCode::Slash => Key::Slash,
    KeyCode::KpReturn => Key::KpReturn,
    KeyCode::KpMinus => Key::KpMinus,
    KeyCode::KpPlus => Key::KpPlus,
    KeyCode::KpMultiply => Key::KpMultiply,
    KeyCode::KpDivide => Key::KpDivide,
    KeyCode::Kp0 => Key::Kp0,
    KeyCode::Kp1 => Key::Kp1,
    KeyCode::Kp2 => Key::Kp2,
    KeyCode::Kp3 => Key::Kp3,
    KeyCode::Kp4 => Key::Kp4,
    KeyCode::Kp5 => Key::Kp5,
    KeyCode::Kp6 => Key::Kp6,
    KeyCode::Kp7 => Key::Kp7,
    KeyCode::Kp8 => Key::Kp8,
    KeyCode::Kp9 => Key::Kp9,
    KeyCode::KpDelete => Key::KpDelete,
    KeyCode::Function => Key::Function,
    KeyCode::Unknown => Key::Unknown(0),
  }
}

impl From<Event> for InputEvent {
  fn from(event: Event) -> Self {
    let (event_type, key_press, key_release, mouse_move, button_press, button_release, wheel) =
      match &event.event_type {
        EventType::KeyPress(key) => (
          EventTypeValue::KeyPress,
          Some(KeyPressEvent {
            key: key_to_type(key),
          }),
          None,
          None,
          None,
          None,
          None,
        ),
        EventType::KeyRelease(key) => (
          EventTypeValue::KeyRelease,
          None,
          Some(KeyReleaseEvent {
            key: key_to_type(key),
          }),
          None,
          None,
          None,
          None,
        ),
        EventType::MouseMove { x, y } => (
          EventTypeValue::MouseMove,
          None,
          None,
          Some(MouseMoveEvent { x: *x, y: *y }),
          None,
          None,
          None,
        ),
        EventType::ButtonPress(btn) => (
          EventTypeValue::ButtonPress,
          None,
          None,
          None,
          Some(ButtonPressEvent {
            button: button_to_type(btn),
          }),
          None,
          None,
        ),
        EventType::ButtonRelease(btn) => (
          EventTypeValue::ButtonRelease,
          None,
          None,
          None,
          None,
          Some(ButtonReleaseEvent {
            button: button_to_type(btn),
          }),
          None,
        ),
        EventType::Wheel { delta_x, delta_y } => (
          EventTypeValue::Wheel,
          None,
          None,
          None,
          None,
          None,
          Some(WheelEvent {
            delta_x: *delta_x,
            delta_y: *delta_y,
          }),
        ),
      };

    InputEvent {
      event_type,
      key_press,
      key_release,
      mouse_move,
      button_press,
      button_release,
      wheel,
      name: event.name,
      time: event
        .time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64,
    }
  }
}

impl TryFrom<InputEvent> for Event {
  type Error = String;

  fn try_from(event: InputEvent) -> Result<Self, Self::Error> {
    let event_type = match event.event_type {
      EventTypeValue::KeyPress => {
        let key = event
          .key_press
          .ok_or("Missing key_press data for KeyPress event")?
          .key;
        EventType::KeyPress(type_to_key(key))
      }
      EventTypeValue::KeyRelease => {
        let key = event
          .key_release
          .ok_or("Missing key_release data for KeyRelease event")?
          .key;
        EventType::KeyRelease(type_to_key(key))
      }
      EventTypeValue::MouseMove => {
        let mv = event
          .mouse_move
          .ok_or("Missing mouse_move data for MouseMove event")?;
        EventType::MouseMove { x: mv.x, y: mv.y }
      }
      EventTypeValue::ButtonPress => {
        let btn = event
          .button_press
          .ok_or("Missing button_press data for ButtonPress event")?
          .button;
        EventType::ButtonPress(type_to_button(btn))
      }
      EventTypeValue::ButtonRelease => {
        let btn = event
          .button_release
          .ok_or("Missing button_release data for ButtonRelease event")?
          .button;
        EventType::ButtonRelease(type_to_button(btn))
      }
      EventTypeValue::Wheel => {
        let w = event.wheel.ok_or("Missing wheel data for Wheel event")?;
        EventType::Wheel {
          delta_x: w.delta_x,
          delta_y: w.delta_y,
        }
      }
    };

    Ok(Event {
      event_type,
      name: event.name,
      time: std::time::SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::from_millis(event.time as u64))
        .unwrap_or(std::time::SystemTime::now()),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn control_alias_maps_to_control_left() {
    assert_eq!(string_key_to_keycode("control"), Some(KeyCode::ControlLeft));
  }

  #[test]
  fn control_alias_is_case_insensitive() {
    assert_eq!(string_key_to_keycode("Control"), Some(KeyCode::ControlLeft));
    assert_eq!(string_key_to_keycode("CONTROL"), Some(KeyCode::ControlLeft));
  }

  #[test]
  fn legacy_controll_typo_kept_for_compatibility() {
    assert_eq!(
      string_key_to_keycode("controll"),
      Some(KeyCode::ControlLeft)
    );
  }

  #[test]
  fn other_control_aliases_still_map() {
    for alias in ["ctrl", "controlleft"] {
      assert_eq!(
        string_key_to_keycode(alias),
        Some(KeyCode::ControlLeft),
        "alias {alias} should map to ControlLeft"
      );
    }
    assert_eq!(
      string_key_to_keycode("controlright"),
      Some(KeyCode::ControlRight)
    );
  }

  #[test]
  fn unknown_keys_map_to_none() {
    assert_eq!(string_key_to_keycode("not-a-key"), None);
    assert_eq!(string_key_to_keycode(""), None);
  }
}
