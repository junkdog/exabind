mod exabind_event;
mod widget;
mod event_handler;
mod dispatcher;
mod app;
mod input;
mod tui;

use app::ExabindApp;

use std::io;
use ratatui::prelude::{Color, Frame, Line, Style, Stylize, Text, Widget};
use ratatui::widgets::{Block, Clear};
use tachyonfx::{fx, CenteredShrink, Effect};
use crate::event_handler::EventHandler;
use crate::tui::Tui;
use crate::widget::KeyboardWidget;

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    let mut app = ExabindApp::new(events.sender());
    let mut tui = Tui::new(ratatui::init(), events);

    while app.is_running() {
        tui.receive_events(|event| {
            app.update(event);
        });

        tui.draw(|f| {
            ui(f);
        })?;
    }
    ratatui::restore();
    Ok(())
}

fn ui(f: &mut Frame<'_>) {
    Clear.render(f.area(), f.buffer_mut());
    Block::default()
        .style(Style::default().bg(Color::DarkGray))
        .render(f.area(), f.buffer_mut());

    let kbd = KeyboardWidget::new();
    kbd.render(f.area(), f.buffer_mut());
}
