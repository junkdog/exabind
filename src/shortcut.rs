use std::fmt::Display;
use crossterm::event::{KeyCode, ModifierKeyCode};
use crate::crossterm::format_keycode;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Action {
    id: String,
    shortcuts: Vec<Shortcut>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Shortcut {
    keystroke: Vec<KeyCode>,
}

impl Shortcut {
    pub fn keystroke(&self) -> &[KeyCode] {
        &self.keystroke
    }

    pub fn new(keystroke: Vec<KeyCode>) -> Self {
        let (modifiers, keystroke): (Vec<KeyCode>, Vec<KeyCode>) = keystroke.into_iter()
            .partition(|k| matches!(k, KeyCode::Modifier(_)));

        let as_modifier = |k: KeyCode| {
            match k {
                KeyCode::Modifier(m) => Some(m),
                _                    => None
            }
        };

        let modifiers: Vec<_> = modifiers.into_iter()
            .filter_map(as_modifier)
            .collect();

        let modifier_key_codes = [
            ModifierKeyCode::IsoLevel3Shift,
            ModifierKeyCode::IsoLevel5Shift,
            ModifierKeyCode::LeftHyper,
            ModifierKeyCode::RightHyper,
            ModifierKeyCode::LeftSuper,
            ModifierKeyCode::RightSuper,
            ModifierKeyCode::LeftControl,
            ModifierKeyCode::RightControl,
            ModifierKeyCode::LeftAlt,
            ModifierKeyCode::RightAlt,
            ModifierKeyCode::LeftShift,
            ModifierKeyCode::RightShift,
        ].into_iter()
            .filter(|m| modifiers.contains(m))
            .map(KeyCode::Modifier);

        let keystroke = modifier_key_codes.chain(keystroke.into_iter()).collect();

        Self {
            keystroke
        }
    }

    pub fn contains(&self, key: KeyCode) -> bool {
        self.keystroke.contains(&key)
    }

    pub fn uses_modifier(&self, key: ModifierKeyCode) -> bool {
        self.keystroke.iter().any(|kc| {
            if let KeyCode::Modifier(m) = kc {
                m == &key
            } else {
                false
            }
        })
    }
}

impl Action {
    pub fn new(id: String, shortcuts: Vec<Shortcut>) -> Self {
        Self {
            id,
            shortcuts
        }
    }

    pub fn name(&self) -> &str {
        &self.id
    }

    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    pub fn is_bound(&self) -> bool {
        !self.shortcuts.is_empty()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shortcuts = self.shortcuts.iter().map(Shortcut::to_string).collect::<Vec<_>>().join(", ");
        write!(f, "{}: {}", self.id, shortcuts)
    }
}

impl Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keystroke = self.keystroke.iter()
            .map(|k| format_keycode(*k))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{}", keystroke)
    }
}