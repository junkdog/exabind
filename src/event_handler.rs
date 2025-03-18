use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crossterm::event::Event as CrosstermEvent;
use ratatui::crossterm::event;
use ratatui::crossterm::event::KeyEventKind;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::Sender<ExabindEvent>,
    receiver: mpsc::Receiver<ExabindEvent>,
    _handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: std::time::Duration) -> Self {
        let (sender, receiver) = mpsc::channel();

        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = std::time::Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("successfully polled for events") {
                        Self::consume_event(&sender);
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.dispatch(ExabindEvent::Tick);
                        last_tick = std::time::Instant::now();
                    }
                }
            })
        };

        Self {
            sender,
            receiver,
            _handler: handler,
        }
    }

    pub fn sender(&self) -> mpsc::Sender<ExabindEvent> {
        self.sender.clone()
    }

    pub fn next(&self) -> Result<ExabindEvent, mpsc::RecvError> {
        self.receiver.recv()
    }

    pub fn try_next(&self) -> Option<ExabindEvent> {
        match self.receiver.try_recv() {
            Ok(e) => Some(e),
            Err(_) => None,
        }
    }

    fn consume_event(sender: &mpsc::Sender<ExabindEvent>) {
        match event::read().expect("event is read") {
            CrosstermEvent::Key(e) if e.kind == KeyEventKind::Press => {
                sender.send(ExabindEvent::KeyPress(e))
            }
            CrosstermEvent::Resize(w, h) => sender.send(ExabindEvent::Resize(w, h)),

            _ => Ok(()),
        }
        .expect("event should have been sent");
    }
}
