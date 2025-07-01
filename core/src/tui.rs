#[cfg(feature = "crossterm")]
use std::io;

#[cfg(feature = "crossterm")]
use crate::event_handler::EventHandler;
#[cfg(feature = "crossterm")]
use crate::exabind_event::ExabindEvent;
#[cfg(feature = "crossterm")]
use ratatui::layout::Size;
#[cfg(feature = "crossterm")]
use ratatui::Frame;

#[cfg(feature = "crossterm")]
pub type CrosstermTerminal =
    ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[cfg(feature = "crossterm")]
pub struct Tui {
    /// Interface to the Terminal.
    terminal: CrosstermTerminal,
    /// Terminal event handler.
    events: EventHandler,
}

#[cfg(feature = "crossterm")]
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