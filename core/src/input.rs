use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crate::key_event::{KeyEvent, KeyCode, ModifierKeyCode};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct InputProcessor {
    sender: Sender<ExabindEvent>,
}

impl InputProcessor {
    pub fn new(sender: Sender<ExabindEvent>) -> Self {
        Self {
            sender,
        }
    }

    pub fn apply(&self, event: &ExabindEvent) {
        if let ExabindEvent::KeyPress(event) = event {
            if let Some(e) = Self::resolve_key_pressed(event) {
                self.sender.dispatch(e);
            }
        }
    }

    fn resolve_key_pressed(event: &KeyEvent) -> Option<ExabindEvent> {
        use KeyCode::*;
        use ModifierKeyCode::*;
        match event.code {
            Char('q')     => Some(ExabindEvent::Shutdown),
            Char('a')     => Some(ExabindEvent::SelectedCategoryFxSandbox),
            Char('s')     => Some(ExabindEvent::StartupAnimation),
            Up            => Some(ExabindEvent::PreviousCategory),
            Down          => Some(ExabindEvent::NextCategory),
            Esc           => Some(ExabindEvent::DeselectCategory),
            Modifier(mfc) => Some(ExabindEvent::ToggleFilterKey(mfc)),
            Char('1')     => Some(ExabindEvent::ToggleFilterKey(LeftShift)),
            Char('2')     => Some(ExabindEvent::ToggleFilterKey(LeftControl)),
            Char('3')     => Some(ExabindEvent::ToggleFilterKey(LeftMeta)),
            Char('4')     => Some(ExabindEvent::ToggleFilterKey(LeftAlt)),
            _             => None,
        }
    }
}