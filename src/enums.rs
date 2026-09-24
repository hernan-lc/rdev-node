use napi_derive::napi;

/// Represents the type of button event
#[napi(string_enum)]
pub enum ButtonType {
  Left,
  Right,
  Middle,
  Unknown,
}

/// Represents keyboard keys
#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
  Alt,
  AltGr,
  Backspace,
  CapsLock,
  ControlLeft,
  ControlRight,
  Delete,
  DownArrow,
  End,
  Escape,
  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  Home,
  LeftArrow,
  MetaLeft,
  MetaRight,
  NumLock,
  PageDown,
  PageUp,
  Return,
  RightArrow,
  ShiftLeft,
  ShiftRight,
  Space,
  Tab,
  UpArrow,
  PrintScreen,
  ScrollLock,
  Pause,
  Insert,
  BackQuote,
  Num1,
  Num2,
  Num3,
  Num4,
  Num5,
  Num6,
  Num7,
  Num8,
  Num9,
  Num0,
  Minus,
  Equal,
  KeyQ,
  KeyW,
  KeyE,
  KeyR,
  KeyT,
  KeyY,
  KeyU,
  KeyI,
  KeyO,
  KeyP,
  LeftBracket,
  RightBracket,
  KeyA,
  KeyS,
  KeyD,
  KeyF,
  KeyG,
  KeyH,
  KeyJ,
  KeyK,
  KeyL,
  SemiColon,
  Quote,
  BackSlash,
  IntlBackslash,
  KeyZ,
  KeyX,
  KeyC,
  KeyV,
  KeyB,
  KeyN,
  KeyM,
  Comma,
  Dot,
  Slash,
  KpReturn,
  KpMinus,
  KpPlus,
  KpMultiply,
  KpDivide,
  Kp0,
  Kp1,
  Kp2,
  Kp3,
  Kp4,
  Kp5,
  Kp6,
  Kp7,
  Kp8,
  Kp9,
  KpDelete,
  Function,
  Unknown,
}

impl KeyCode {
  /// Returns true if this key is a modifier key (Ctrl, Shift, Alt, Meta)
  pub fn is_modifier(&self) -> bool {
    matches!(
      self,
      KeyCode::ControlLeft
        | KeyCode::ControlRight
        | KeyCode::ShiftLeft
        | KeyCode::ShiftRight
        | KeyCode::Alt
        | KeyCode::AltGr
        | KeyCode::MetaLeft
        | KeyCode::MetaRight
    )
  }

  /// Returns the normalized modifier name if this is a modifier key
  pub fn as_modifier_name(&self) -> Option<&'static str> {
    match self {
      KeyCode::ControlLeft | KeyCode::ControlRight => Some("ctrl"),
      KeyCode::ShiftLeft | KeyCode::ShiftRight => Some("shift"),
      KeyCode::Alt | KeyCode::AltGr => Some("alt"),
      KeyCode::MetaLeft | KeyCode::MetaRight => Some("meta"),
      _ => None,
    }
  }

  /// Returns the lowercase string representation for comparison
  pub fn as_lowercase(&self) -> String {
    format!("{:?}", self).to_lowercase()
  }
}

/// Represents normalized modifier keys for shortcut handling
#[napi(string_enum)]
pub enum NormalizedModifier {
  Ctrl,
  Shift,
  Alt,
  Meta,
}

/// Represents the type of event
#[napi(string_enum)]
pub enum EventTypeValue {
  KeyPress,
  KeyRelease,
  MouseMove,
  ButtonPress,
  ButtonRelease,
  Wheel,
}
