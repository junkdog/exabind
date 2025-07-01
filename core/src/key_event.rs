#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    Media(MediaKeyCode),
    Modifier(ModifierKeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierKeyCode {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
    }
}

impl KeyEvent {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

// Conversion from crossterm KeyEvent to our custom KeyEvent
#[cfg(feature = "crossterm")]
impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        Self {
            code: event.code.into(),
            modifiers: event.modifiers.into(),
        }
    }
}

#[cfg(feature = "crossterm")]
impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(code: crossterm::event::KeyCode) -> Self {
        use crossterm::event::KeyCode as CrosstermKeyCode;
        match code {
            CrosstermKeyCode::Backspace => KeyCode::Backspace,
            CrosstermKeyCode::Enter => KeyCode::Enter,
            CrosstermKeyCode::Left => KeyCode::Left,
            CrosstermKeyCode::Right => KeyCode::Right,
            CrosstermKeyCode::Up => KeyCode::Up,
            CrosstermKeyCode::Down => KeyCode::Down,
            CrosstermKeyCode::Home => KeyCode::Home,
            CrosstermKeyCode::End => KeyCode::End,
            CrosstermKeyCode::PageUp => KeyCode::PageUp,
            CrosstermKeyCode::PageDown => KeyCode::PageDown,
            CrosstermKeyCode::Tab => KeyCode::Tab,
            CrosstermKeyCode::BackTab => KeyCode::BackTab,
            CrosstermKeyCode::Delete => KeyCode::Delete,
            CrosstermKeyCode::Insert => KeyCode::Insert,
            CrosstermKeyCode::F(n) => KeyCode::F(n),
            CrosstermKeyCode::Char(c) => KeyCode::Char(c),
            CrosstermKeyCode::Null => KeyCode::Null,
            CrosstermKeyCode::Esc => KeyCode::Esc,
            CrosstermKeyCode::CapsLock => KeyCode::CapsLock,
            CrosstermKeyCode::ScrollLock => KeyCode::ScrollLock,
            CrosstermKeyCode::NumLock => KeyCode::NumLock,
            CrosstermKeyCode::PrintScreen => KeyCode::PrintScreen,
            CrosstermKeyCode::Pause => KeyCode::Pause,
            CrosstermKeyCode::Menu => KeyCode::Menu,
            CrosstermKeyCode::KeypadBegin => KeyCode::KeypadBegin,
            CrosstermKeyCode::Media(media) => KeyCode::Media(media.into()),
            CrosstermKeyCode::Modifier(modifier) => KeyCode::Modifier(modifier.into()),
        }
    }
}

#[cfg(feature = "crossterm")]
impl From<crossterm::event::MediaKeyCode> for MediaKeyCode {
    fn from(media: crossterm::event::MediaKeyCode) -> Self {
        use crossterm::event::MediaKeyCode as CrosstermMediaKeyCode;
        match media {
            CrosstermMediaKeyCode::Play => MediaKeyCode::Play,
            CrosstermMediaKeyCode::Pause => MediaKeyCode::Pause,
            CrosstermMediaKeyCode::PlayPause => MediaKeyCode::PlayPause,
            CrosstermMediaKeyCode::Reverse => MediaKeyCode::Reverse,
            CrosstermMediaKeyCode::Stop => MediaKeyCode::Stop,
            CrosstermMediaKeyCode::FastForward => MediaKeyCode::FastForward,
            CrosstermMediaKeyCode::Rewind => MediaKeyCode::Rewind,
            CrosstermMediaKeyCode::TrackNext => MediaKeyCode::TrackNext,
            CrosstermMediaKeyCode::TrackPrevious => MediaKeyCode::TrackPrevious,
            CrosstermMediaKeyCode::Record => MediaKeyCode::Record,
            CrosstermMediaKeyCode::LowerVolume => MediaKeyCode::LowerVolume,
            CrosstermMediaKeyCode::RaiseVolume => MediaKeyCode::RaiseVolume,
            CrosstermMediaKeyCode::MuteVolume => MediaKeyCode::MuteVolume,
        }
    }
}

#[cfg(feature = "crossterm")]
impl From<crossterm::event::ModifierKeyCode> for ModifierKeyCode {
    fn from(modifier: crossterm::event::ModifierKeyCode) -> Self {
        use crossterm::event::ModifierKeyCode as CrosstermModifierKeyCode;
        match modifier {
            CrosstermModifierKeyCode::LeftShift => ModifierKeyCode::LeftShift,
            CrosstermModifierKeyCode::LeftControl => ModifierKeyCode::LeftControl,
            CrosstermModifierKeyCode::LeftAlt => ModifierKeyCode::LeftAlt,
            CrosstermModifierKeyCode::LeftSuper => ModifierKeyCode::LeftSuper,
            CrosstermModifierKeyCode::LeftHyper => ModifierKeyCode::LeftHyper,
            CrosstermModifierKeyCode::LeftMeta => ModifierKeyCode::LeftMeta,
            CrosstermModifierKeyCode::RightShift => ModifierKeyCode::RightShift,
            CrosstermModifierKeyCode::RightControl => ModifierKeyCode::RightControl,
            CrosstermModifierKeyCode::RightAlt => ModifierKeyCode::RightAlt,
            CrosstermModifierKeyCode::RightSuper => ModifierKeyCode::RightSuper,
            CrosstermModifierKeyCode::RightHyper => ModifierKeyCode::RightHyper,
            CrosstermModifierKeyCode::RightMeta => ModifierKeyCode::RightMeta,
            CrosstermModifierKeyCode::IsoLevel3Shift => ModifierKeyCode::IsoLevel3Shift,
            CrosstermModifierKeyCode::IsoLevel5Shift => ModifierKeyCode::IsoLevel5Shift,
        }
    }
}

#[cfg(feature = "crossterm")]
impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(modifiers: crossterm::event::KeyModifiers) -> Self {
        let mut result = KeyModifiers::empty();
        
        if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            result |= KeyModifiers::SHIFT;
        }
        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            result |= KeyModifiers::CONTROL;
        }
        if modifiers.contains(crossterm::event::KeyModifiers::ALT) {
            result |= KeyModifiers::ALT;
        }
        if modifiers.contains(crossterm::event::KeyModifiers::SUPER) {
            result |= KeyModifiers::SUPER;
        }
        if modifiers.contains(crossterm::event::KeyModifiers::HYPER) {
            result |= KeyModifiers::HYPER;
        }
        if modifiers.contains(crossterm::event::KeyModifiers::META) {
            result |= KeyModifiers::META;
        }
        
        result
    }
}