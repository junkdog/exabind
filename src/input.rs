use std::sync::mpsc::Sender;
use crossterm::event::{KeyCode, KeyEvent, ModifierKeyCode};
use crate::dispatcher::Dispatcher;
use crate::exabind_event::{ExabindEvent, UiElement};

#[derive(Debug)]
pub struct InputProcessor {
    sender: Sender<ExabindEvent>,
    input_receiver_view: UiElement,
}

impl InputProcessor {
    pub fn new(sender: Sender<ExabindEvent>) -> Self {
        Self {
            sender,
            input_receiver_view: UiElement::Category,
        }
    }

    pub fn change_input(&mut self, receiver: UiElement) {
        self.input_receiver_view = receiver;
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
        use crossterm::event::{KeyCode::*, ModifierKeyCode::*};
        match event.code {
            Char('q')     => Some(ExabindEvent::Shutdown),
            Char('h')     => Some(ExabindEvent::ToggleHighlightShortcuts),
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