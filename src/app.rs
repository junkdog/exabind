use std::sync::mpsc::Sender;
use tachyonfx::Duration;
use crate::exabind_event::ExabindEvent;
use crate::input::InputProcessor;

pub struct ExabindApp {
    running: bool,
    sender: Sender<ExabindEvent>,
    last_tick: std::time::Instant,
    last_frame_duration: Duration,
    input_processor: InputProcessor,
}

impl ExabindApp {
    pub fn new(sender: Sender<ExabindEvent>) -> Self {
        Self {
            running: true,
            input_processor: InputProcessor::new(sender.clone()),
            sender,
            last_tick: std::time::Instant::now(),
            last_frame_duration: Duration::ZERO,
        }
    }

    pub fn sender(&self) -> Sender<ExabindEvent> {
        self.sender.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update(&mut self, event: ExabindEvent) {
        let now = std::time::Instant::now();
        self.last_frame_duration = now.duration_since(self.last_tick).into();
        self.last_tick = now;

        self.apply_event(event);
    }

    fn apply_event(&mut self, event: ExabindEvent) {
        match event {
            ExabindEvent::Tick        => (),
            ExabindEvent::Shutdown    => self.running = false,
            ExabindEvent::KeyPress(_) => self.input_processor.apply(&event),
        }
    }
}