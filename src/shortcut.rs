use std::fmt::Display;
use crossterm::event::KeyCode;
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
        Self {
            keystroke
        }
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