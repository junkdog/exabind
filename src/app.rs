use std::sync::mpsc::Sender;
use std::time::Instant;
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
    sender: Sender<ExabindEvent>,
    keymap: KeyMap,
    last_tick: Instant,
    input_processor: InputProcessor,
    effects: Vec<Effect>
}

impl ExabindApp {
    pub fn new(
        sender: Sender<ExabindEvent>,
        keymap: KeyMap,
    ) -> Self {
        Self {
            running: true,
            input_processor: InputProcessor::new(sender.clone()),
            sender,
            keymap,
            last_tick: Instant::now(),
            effects: Vec::new()
        }
    }

    pub fn keymap(&self) -> &KeyMap {
        &self.keymap
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
        }
    }
}