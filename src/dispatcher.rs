use std::sync::mpsc;

pub trait Dispatcher<T> {
    fn dispatch(&self, event: T);
}

impl<T> Dispatcher<T> for mpsc::Sender<T> {
    fn dispatch(&self, event: T) {
        self.send(event).expect("event should have been sent");
    }
}