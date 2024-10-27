use std::sync::mpsc::Sender;
use std::time::Instant;
use crossterm::event::{KeyCode, ModifierKeyCode};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tachyonfx::{Duration, Effect, Shader};
use crate::effect::starting_up;
use crate::exabind_event::ExabindEvent;
use crate::input::InputProcessor;
use crate::parser::jetbrains::KeyMap;
use crate::ui;
use crate::ui_state::UiState;

pub struct ExabindApp {
    running: bool,
    keymap_context: KeyMapContext,
    sender: Sender<ExabindEvent>,
    last_tick: Instant,
    input_processor: InputProcessor,
    effects: Vec<Effect>,
}


pub struct KeyMapContext {
    pub keymap: KeyMap,
    pub categories: Vec<(String, usize)>,
    pub current_category: usize,
    pub filter_key_control: bool,
    pub filter_key_alt: bool,
    pub filter_key_shift: bool,
}

impl KeyMapContext {
    pub fn next_category(&mut self) {
        if self.current_category == self.categories.len() - 1 {
            self.current_category = 0;
        } else {
            self.current_category += 1;
        }
    }

    pub fn previous_category(&mut self) {
        if self.current_category == 0 {
            self.current_category = self.categories.len() - 1;
        } else {
            self.current_category -= 1;
        }
    }

    pub fn category(&self) -> &str {
        self.categories[self.current_category].0.as_str()
    }

    pub fn toggle_filter_key(&mut self, key_code: ModifierKeyCode) {
        use ModifierKeyCode::*;
        match key_code {
            LeftShift | RightShift => self.filter_key_shift = !self.filter_key_shift,
            LeftControl | RightControl => self.filter_key_control = !self.filter_key_control,
            LeftAlt | RightAlt => self.filter_key_alt = !self.filter_key_alt,
            // LeftSuper | RightSuper => (),
            // LeftHyper | RightHyper => (),
            // LeftMeta | RightMeta =>(),
            _ => panic!("Invalid modifier key code: {:?}", key_code),
        }
    }
}

impl ExabindApp {
    pub fn new(
        sender: Sender<ExabindEvent>,
        keymap: KeyMap,
    ) -> Self {
        let keymap_context = KeyMapContext {
            categories: keymap.categories(),
            current_category: 0,
            filter_key_control: false,
            filter_key_alt: false,
            filter_key_shift: false,
            keymap,
        };
        Self {
            running: true,
            input_processor: InputProcessor::new(sender.clone()),
            sender,
            keymap_context,
            last_tick: Instant::now(),
            effects: Vec::new()
        }
    }

    pub fn keymap(&self) -> &KeyMap {
        &self.keymap_context.keymap
    }

    pub fn keymap_context(&self) -> &KeyMapContext {
        &self.keymap_context
    }

    pub fn register_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn sender(&self) -> Sender<ExabindEvent> {
        self.sender.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update_time(&mut self) -> Duration {
        let now = Instant::now();
        let last_frame_duration: Duration = now.duration_since(self.last_tick).into();
        self.last_tick = now;
        last_frame_duration.into()
    }

    pub fn update_effects(&mut self, last_frame_duration: Duration, buf: &mut Buffer, area: Rect) {
        for effect in self.effects.iter_mut() {
            effect.process(last_frame_duration, buf, area);
        }

        self.effects.retain(Effect::running);
    }

    pub fn apply_event(&mut self, event: ExabindEvent, ui_state: &mut UiState) {
        use ExabindEvent::*;
        match event {
            Tick                      => (),
            Shutdown                  => self.running = false,
            KeyPress(_)               => self.input_processor.apply(&event),
            ToggleHighlightShortcuts  => ui_state.toggle_highlight_shortcuts(),
            StartupAnimation          => ui_state.register_kbd_effect(starting_up()),
            NextCategory              => {
                self.keymap_context.next_category();
                let cat = self.keymap_context.category();
                ui_state.render_selected_actions(cat, self.keymap_context());
            },
            PreviousCategory          => {
                self.keymap_context.previous_category();
                let cat = self.keymap_context.category();
                ui_state.render_selected_actions(cat, self.keymap_context());
            },
            ToggleFilterKey(key_code) => {
                self.keymap_context.toggle_filter_key(key_code);
                let cat = self.keymap_context.category();
                ui_state.render_selected_actions(cat, self.keymap_context());
            },
        }
    }
}