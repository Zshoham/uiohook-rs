use bitflags::bitflags;
use ffi::*;
use uiohook_sys as ffi;

pub const EVENT_HOOK_ENABLED: ffi::event_type = ffi::_event_type_EVENT_HOOK_ENABLED;
pub const EVENT_HOOK_DISABLED: ffi::event_type = ffi::_event_type_EVENT_HOOK_DISABLED;
pub const EVENT_KEY_TYPED: ffi::event_type = ffi::_event_type_EVENT_KEY_TYPED;
pub const EVENT_KEY_PRESSED: ffi::event_type = ffi::_event_type_EVENT_KEY_PRESSED;
pub const EVENT_KEY_RELEASED: ffi::event_type = ffi::_event_type_EVENT_KEY_RELEASED;
pub const EVENT_MOUSE_CLICKED: ffi::event_type = ffi::_event_type_EVENT_MOUSE_CLICKED;
pub const EVENT_MOUSE_PRESSED: ffi::event_type = ffi::_event_type_EVENT_MOUSE_PRESSED;
pub const EVENT_MOUSE_RELEASED: ffi::event_type = ffi::_event_type_EVENT_MOUSE_RELEASED;
pub const EVENT_MOUSE_MOVED: ffi::event_type = ffi::_event_type_EVENT_MOUSE_MOVED;
pub const EVENT_MOUSE_DRAGGED: ffi::event_type = ffi::_event_type_EVENT_MOUSE_DRAGGED;
pub const EVENT_MOUSE_WHEEL: ffi::event_type = ffi::_event_type_EVENT_MOUSE_WHEEL;

bitflags! {
    #[derive(Default)]
    /// Specifies weather the event is reserved or synthetic.
    ///
    /// `Reserved` events are events that will or did not propagate to the UI of the system, this
    /// flag can be set using the [`reserve_events`] method.
    ///
    /// `Synthetic` events are events that were created by this library.
    ///
    /// Note that the synthetic flag is not guaranteed to be accurate, it could be set
    /// when it the event is not synthetic, and could not be set for a synthetic event.
    /// For more information read the [`crate::hook::global`] documentation.
    ///
    /// [`reserve_events`]: crate::hook::global::reserve_events
    pub struct EventMode: u16 {
        const DEFAULT = 0b00000000;
        const RESERVED = 0b00000001;
        const SYNTHETIC = 0b00000010;
    }
}

const MASK_NONE: u32 = 0;

crate::constant_to_enum! {
    /// Mask applied to events to indicate that they are a part of a sequence or combination
    /// such as `Ctrl-C`
    ///
    /// Each variant represents the mask that should be applied when creating a sequence with
    /// the equivalent key, when we want a `Ctrl-A` combination we should apply the `Control` mask
    /// or one of `LeftControl`, `RightControl` masks if we want to be more specific.
    (u32 => u16) => EventMask {
        MASK_NONE => None,
        MASK_SHIFT_L => LeftShift,
        MASK_CTRL_L => LeftControl,
        MASK_META_L => LeftMeta,
        MASK_ALT_L =>  LeftAlt,
        MASK_SHIFT_R => RightShift,
        MASK_CTRL_R => RightControl,
        MASK_META_R => RightMeta,
        MASK_ALT_R =>  RightAlt,
        MASK_SHIFT => Shift,
        MASK_CTRL => Control,
        MASK_META => Meta,
        MASK_ALT => Alt,
        MASK_BUTTON1 => LeftMouseButton,
        MASK_BUTTON2 => RightMouseButton,
        MASK_BUTTON3 => MiddleMouseButton,
        MASK_BUTTON4 => ExtraMouseButton1,
        MASK_BUTTON5 => ExtraMouseButton2,
        MASK_NUM_LOCK => NumLock,
        MASK_CAPS_LOCK => CapsLock,
        MASK_SCROLL_LOCK => ScrollLock
    }
}

crate::constant_to_enum! {
    (u32 => u16) => Key {
        VC_ESCAPE => Escape,
        VC_F1 => F1,
        VC_F2 => F2,
        VC_F3 => F3,
        VC_F4 => F4,
        VC_F5 => F5,
        VC_F6 => F6,
        VC_F7 => F7,
        VC_F8 => F8,
        VC_F9 => F9,
        VC_F10 => F10,
        VC_F11 => F11,
        VC_F12 => F12,
        VC_F13 => F13,
        VC_F14 => F14,
        VC_F15 => F15,
        VC_F16 => F16,
        VC_F17 => F17,
        VC_F18 => F18,
        VC_F19 => F19,
        VC_F20 => F20,
        VC_F21 => F21,
        VC_F22 => F22,
        VC_F23 => F23,
        VC_F24 => F24,
        VC_BACKQUOTE => Backquote,
        VC_1 => Key1,
        VC_2 => Key2,
        VC_3 => Key3,
        VC_4 => Key4,
        VC_5 => Key5,
        VC_6 => Key6,
        VC_7 => Key7,
        VC_8 => Key8,
        VC_9 => Key9,
        VC_0 => Key0,
        VC_MINUS => Minus,
        VC_EQUALS => Equals,
        VC_BACKSPACE => Backspace,
        VC_TAB => Tab,
        VC_CAPS_LOCK => CapsLock,
        VC_A => A,
        VC_B => B,
        VC_C => C,
        VC_D => D,
        VC_E => E,
        VC_F => F,
        VC_G => G,
        VC_H => H,
        VC_I => I,
        VC_J => J,
        VC_K => K,
        VC_L => L,
        VC_M => M,
        VC_N => N,
        VC_O => O,
        VC_P => P,
        VC_Q => Q,
        VC_R => R,
        VC_S => S,
        VC_T => T,
        VC_U => U,
        VC_V => V,
        VC_W => W,
        VC_X => X,
        VC_Y => Y,
        VC_Z => Z,
        VC_OPEN_BRACKET => OpenBracket,
        VC_CLOSE_BRACKET => CloseBracket,
        VC_BACK_SLASH => BackSlash,
        VC_SEMICOLON => SemiColon,
        VC_QUOTE => Quote,
        VC_ENTER => Enter,
        VC_COMMA => Comma,
        VC_PERIOD => Period,
        VC_SLASH => Slash,
        VC_SPACE => Space,
        VC_PRINTSCREEN => PrintScreen,
        VC_SCROLL_LOCK => ScrollLock,
        VC_PAUSE => Pause,
        VC_LESSER_GREATER => LesserGreater,
        VC_INSERT => Insert,
        VC_DELETE => Delete,
        VC_HOME => Home,
        VC_END => End,
        VC_PAGE_UP => PageUp,
        VC_PAGE_DOWN => PageDown,
        VC_UP => Up,
        VC_LEFT => Left,
        VC_CLEAR => Clear,
        VC_RIGHT => Right,
        VC_DOWN => Down,
        VC_NUM_LOCK => NumLock,
        VC_KP_DIVIDE => NumPadDivide,
        VC_KP_MULTIPLY => NumPadMultiply,
        VC_KP_SUBTRACT => NumPadSubtract,
        VC_KP_EQUALS => NumPadEquals,
        VC_KP_ADD => NumPadAdd,
        VC_KP_ENTER => NumPadEnter,
        VC_KP_SEPARATOR => NumPadSeparator,
        VC_KP_1 => NumPad1,
        VC_KP_2 => NumPad2,
        VC_KP_3 => NumPad3,
        VC_KP_4 => NumPad4,
        VC_KP_5 => NumPad5,
        VC_KP_6 => NumPad6,
        VC_KP_7 => NumPad7,
        VC_KP_8 => NumPad8,
        VC_KP_9 => NumPad9,
        VC_KP_0 => NumPad0,
        VC_KP_END => NumPadEnd,
        VC_KP_DOWN => NumPadDown,
        VC_KP_PAGE_DOWN => NumPadPageDown,
        VC_KP_LEFT => NumPadLeft,
        VC_KP_CLEAR => NumPadClear,
        VC_KP_RIGHT => NumPadRight,
        VC_KP_HOME => NumPadHome,
        VC_KP_UP => NumPadUp,
        VC_KP_PAGE_UP => NumPadPageUp,
        VC_KP_INSERT => NumPadInsert,
        VC_KP_DELETE => NumPadDelete,
        VC_SHIFT_L => LeftShift,
        VC_SHIFT_R => RightShift,
        VC_CONTROL_L => LeftControl,
        VC_CONTROL_R => RightControl,
        VC_ALT_L => LeftAlt,
        VC_ALT_R => RightAlt,
        VC_META_L => LeftMeta,
        VC_META_R => RightMeta,
        VC_CONTEXT_MENU => ContextMenu,
        VC_POWER => Power,
        VC_SLEEP => Sleep,
        VC_WAKE => Wake,
        VC_MEDIA_PLAY => MediaPlay,
        VC_MEDIA_STOP => MediaStop,
        VC_MEDIA_PREVIOUS => MediaPrevious,
        VC_MEDIA_NEXT => MediaNext,
        VC_MEDIA_SELECT => MediaSelect,
        VC_MEDIA_EJECT => MediaEject,
        VC_VOLUME_MUTE => MediaMute,
        VC_VOLUME_UP => VolumeUp,
        VC_VOLUME_DOWN => VolumeDown,
        VC_APP_MAIL => AppMail,
        VC_APP_CALCULATOR => AppCalculator,
        VC_APP_MUSIC => AppMusic,
        VC_APP_PICTURES => Apppictures,
        VC_BROWSER_SEARCH => BrowserSearch,
        VC_BROWSER_HOME => BrowserHome,
        VC_BROWSER_BACK => BrowserBack,
        VC_BROWSER_FORWARD => BrowserForward,
        VC_BROWSER_STOP => BrowserStop,
        VC_BROWSER_REFRESH => BrowserRefresh,
        VC_BROWSER_FAVORITES => BrowserFavorites,
        VC_KATAKANA => Katakana,
        VC_UNDERSCORE => Underscore,
        VC_FURIGANA => Furigana,
        VC_KANJI => Kanji,
        VC_HIRAGANA => Hiragana,
        VC_YEN => Yen,
        VC_KP_COMMA => NumPadComma,
        VC_SUN_HELP => SunHelp,
        VC_SUN_STOP => SunStop,
        VC_SUN_PROPS => SunProps,
        VC_SUN_FRONT => SunFront,
        VC_SUN_OPEN => SunOpen,
        VC_SUN_FIND => SinFind,
        VC_SUN_AGAIN => SunAgain,
        VC_SUN_UNDO => SunUndo,
        VC_SUN_COPY => SunCopy,
        VC_SUN_INSERT => SunInsert,
        VC_SUN_CUT => SunCut,
        VC_UNDEFINED => Undefined
    }
}

crate::constant_to_enum! {
    (u32 => u16) => MouseButton {
        MOUSE_NOBUTTON => NoButton,
        MOUSE_BUTTON1 => Left,
        MOUSE_BUTTON2 => Right,
        MOUSE_BUTTON3 => Middle,
        MOUSE_BUTTON4 => Extra1,
        MOUSE_BUTTON5 => Extra2,
    }
}

crate::constant_to_enum! {
    (u32 => u8) => MouseScrollKind {
        WHEEL_UNIT_SCROLL => Unit,
        WHEEL_BLOCK_SCROLL => Block
    }
}

crate::constant_to_enum! { (u32 => u8) => MouseScrollDirection {
    WHEEL_VERTICAL_DIRECTION => Vertical,
    WHEEL_HORIZONTAL_DIRECTION => Horizontal
} }
