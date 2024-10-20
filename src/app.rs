use std::sync::mpsc::Sender;
use std::time::Instant;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tachyonfx::{Duration, Effect, Shader};
use crate::exabind_event::ExabindEvent;
use crate::input::InputProcessor;

pub struct ExabindApp {
    running: bool,
    sender: Sender<ExabindEvent>,
    last_tick: Instant,
    input_processor: InputProcessor,
    effects: Vec<Effect>
}

impl ExabindApp {
    pub fn new(sender: Sender<ExabindEvent>) -> Self {
        Self {
            running: true,
            input_processor: InputProcessor::new(sender.clone()),
            sender,
            last_tick: Instant::now(),
            effects: Vec::new()
        }
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

    pub fn apply_event(&mut self, event: ExabindEvent) {
        match event {
            ExabindEvent::Tick        => (),
            ExabindEvent::Shutdown    => self.running = false,
            ExabindEvent::KeyPress(_) => self.input_processor.apply(&event),
        }
    }
}