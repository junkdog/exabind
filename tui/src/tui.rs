use std::io;


use crate::event_handler::EventHandler;
use crate::exabind_event::ExabindEvent;
use ratatui::layout::Size;
use ratatui::Frame;

pub type CrosstermTerminal =
    ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
pub struct Tui {
    /// Interface to the Terminal.
    terminal: CrosstermTerminal,
    /// Terminal event handler.
    events: EventHandler,
}

impl Tui {
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn draw(
        &mut self,
        render_ui: impl FnOnce(&mut Frame),
    ) -> io::Result<()> {
        self.terminal.draw(render_ui)?;
        Ok(())
    }

    pub fn size(&self) -> Size {
        self.terminal.size().unwrap()
    }

    /// iterates over all currently available events; waits
    /// until at least one event is available.
    pub fn receive_events<F>(&self, mut f: F)
        where F: FnMut(ExabindEvent)
    {
        f(self.events.next().unwrap());
        while let Some(event) = self.events.try_next() { f(event) }
    }
}