use std::sync::mpsc::Sender;
use crossterm::event::{KeyCode, KeyEvent};
use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;

#[derive(Debug)]
pub struct InputProcessor {
    sender: Sender<ExabindEvent>,
}

impl InputProcessor {
    pub fn new(sender: Sender<ExabindEvent>) -> Self {
        Self {
            sender
        }
    }

    pub fn apply(&self, event: &ExabindEvent) {
        match event {
            ExabindEvent::KeyPress(event) => {
                if let Some(e) = Self::resolve_key_pressed(event) {
                    self.sender.dispatch(e);
                }
            }
            _ => {}
        }
    }

    fn resolve_key_pressed(event: &KeyEvent) -> Option<ExabindEvent> {
        match event.code {
            KeyCode::Char('q') => Some(ExabindEvent::Shutdown),
            KeyCode::Char('s') => Some(ExabindEvent::ToggleHighlightShortcuts),
            _ => None,
        }
    }
}