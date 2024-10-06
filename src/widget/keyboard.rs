use crossterm::event::{KeyCode, ModifierKeyCode};
use ratatui::prelude::Position;


pub struct KeyboardWidget {
}


impl KeyboardWidget {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn position_of_key(&self, key: KeyCode) -> Position {
        let (x, y) = match key {
            KeyCode::Backspace => (0, 3),
            KeyCode::Enter => (0, 7),
            KeyCode::Left => (0, 11),
            KeyCode::Right => (0, 11),
            KeyCode::Up => (0, 9),
            KeyCode::Down => (0, 11),
            KeyCode::Home => (0, 3),
            KeyCode::End => (0, 5),
            KeyCode::PageUp => (0, 3),
            KeyCode::PageDown => (0, 5),
            KeyCode::Tab => (0, 5),
            KeyCode::BackTab => (0, 3),
            KeyCode::Delete => (0, 5),
            KeyCode::Insert => (0, 3),
            KeyCode::F(_) => (0, 0),
            KeyCode::Char(_) => (0, 0),
            KeyCode::Null => (0, 0),
            KeyCode::Esc => (0, 0),
            KeyCode::CapsLock => (0, 7),
            KeyCode::ScrollLock => (0, 0),
            KeyCode::NumLock => (0, 0),
            KeyCode::PrintScreen => (0, 0),
            KeyCode::Pause => (0, 0),
            KeyCode::Menu => (0, 11),
            KeyCode::KeypadBegin => (0, 0),
            KeyCode::Media(_) => (0, 0),
            KeyCode::Modifier(c) => match c {
                ModifierKeyCode::LeftShift => (9, 0),
                ModifierKeyCode::LeftControl => (11, 0),
                ModifierKeyCode::LeftAlt => (11, 0),
                ModifierKeyCode::LeftSuper => (11, 0),
                ModifierKeyCode::LeftHyper => (11, 0),
                ModifierKeyCode::LeftMeta => (11, 0),
                ModifierKeyCode::RightShift => (9, 0),
                ModifierKeyCode::RightControl => (11, 0),
                ModifierKeyCode::RightAlt => (11, 0),
                ModifierKeyCode::RightSuper => (11, 0),
                ModifierKeyCode::RightHyper => (11, 0),
                ModifierKeyCode::RightMeta => (11, 0),
                ModifierKeyCode::IsoLevel3Shift => (9, 0),
                ModifierKeyCode::IsoLevel5Shift => (9, 0),
            },
        };

        Position::new(x, y)
    }
}