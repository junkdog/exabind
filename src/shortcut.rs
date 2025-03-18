use crate::crossterm::format_keycode;
use crossterm::event::{KeyCode, ModifierKeyCode};
use std::fmt::Display;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Action {
    id: String,
    category: String,
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
        let (modifiers, keystroke): (Vec<KeyCode>, Vec<KeyCode>) = keystroke
            .into_iter()
            .partition(|k| matches!(k, KeyCode::Modifier(_)));

        let as_modifier = |k: KeyCode| {
            if let KeyCode::Modifier(m) = k {
                Some(m)
            } else {
                None
            }
        };

        let modifiers: Vec<_> = modifiers.into_iter().filter_map(as_modifier).collect();

        let modifier_key_codes = [
            ModifierKeyCode::IsoLevel3Shift,
            ModifierKeyCode::IsoLevel5Shift,
            ModifierKeyCode::LeftHyper,
            ModifierKeyCode::RightHyper,
            ModifierKeyCode::LeftSuper,
            ModifierKeyCode::RightSuper,
            ModifierKeyCode::LeftMeta,
            ModifierKeyCode::RightMeta,
            ModifierKeyCode::LeftControl,
            ModifierKeyCode::RightControl,
            ModifierKeyCode::LeftAlt,
            ModifierKeyCode::RightAlt,
            ModifierKeyCode::LeftShift,
            ModifierKeyCode::RightShift,
        ]
        .into_iter()
        .filter(|m| modifiers.contains(m))
        .map(KeyCode::Modifier);

        let keystroke = modifier_key_codes.chain(keystroke).collect();

        Self { keystroke }
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
    pub fn new_filter_empty<S: ToString>(id: S, category: S, shortcuts: Vec<Shortcut>) -> Self {
        Self {
            id: id.to_string(),
            category: category.to_string(),
            shortcuts: shortcuts
                .into_iter()
                .filter(|s| !s.keystroke.is_empty())
                .collect(),
        }
    }

    pub fn name(&self) -> &str {
        &self.id
    }

    pub fn shortcuts(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    pub fn update_category<S: ToString>(&mut self, category: S) {
        self.category = category.to_string();
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shortcuts = self
            .shortcuts
            .iter()
            .map(Shortcut::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}: {}", self.id, shortcuts)
    }
}

impl Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keystroke = self
            .keystroke
            .iter()
            .map(|k| format_keycode(*k))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{}", keystroke)
    }
}
