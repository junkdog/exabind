mod exabind_event;
mod widget;
mod event_handler;
mod dispatcher;
mod app;
mod input;
mod tui;
mod effect;

use app::ExabindApp;

use std::io;
use crossterm::event::KeyCode;
use ratatui::prelude::{Color, Frame, Line, Style, Stylize, Text, Widget};
use ratatui::widgets::{Block, Clear};
use tachyonfx::{fx, CenteredShrink, Duration, Effect, Shader};
use crate::event_handler::EventHandler;
use crate::tui::Tui;
use crate::widget::{AnsiKeyboardTklLayout, KeyboardLayout, KeyboardWidget};

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    let mut app = ExabindApp::new(events.sender());
    let mut tui = Tui::new(ratatui::init(), events);

    let e = effect::key_press(Duration::from_millis(100), Color::Cyan)
        .with_area(AnsiKeyboardTklLayout::default().key_area(KeyCode::Char('e')));
    app.register_effect(e);

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.update(event);
        });

        tui.draw(|f| {
            ui(f);
            effects(elapsed, &mut app, f);
        })?;
    }
    ratatui::restore();
    Ok(())
}

fn effects(
    elapsed: Duration,
    app: &mut ExabindApp,
    f: &mut Frame<'_>,
) {
    let area = f.area().clone();
    let buf = f.buffer_mut();
    app.update_effects(elapsed, buf, area);
}

fn ui(f: &mut Frame<'_>) {
    Clear.render(f.area(), f.buffer_mut());
    Block::default()
        .style(Style::default().bg(Color::DarkGray))
        .render(f.area(), f.buffer_mut());

    let kbd = KeyboardWidget::new();
    kbd.render(f.area(), f.buffer_mut());
}
