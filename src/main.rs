mod exabind_event;
mod widget;
mod event_handler;
mod dispatcher;
mod app;
mod input;
mod tui;
mod effect;
mod parser;
mod crossterm;
mod styling;

use std::env::args;
use app::ExabindApp;

use std::io;
use ::crossterm::event::KeyCode;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Layout;
use ratatui::prelude::{Color, Frame, Line, Style, Stylize, Text, Widget};
use ratatui::widgets::{Block, Clear};
use tachyonfx::{fx, CenteredShrink, Duration, Effect, Shader};
use tachyonfx::widget::EffectTimeline;
use crate::event_handler::EventHandler;
use crate::styling::Catppuccin;
use crate::tui::Tui;
use crate::widget::{AnsiKeyboardTklLayout, ColorDemoWidget, KeyboardLayout, KeyboardWidget};

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    let mut app = ExabindApp::new(events.sender());
    let mut tui = Tui::new(ratatui::init(), events);

    "exabind".char_indices().enumerate().for_each(|(i, (_, c))| {
        let kbd = AnsiKeyboardTklLayout::default();
        let e = effect::key_press(Duration::from_millis(i as u32 * 250), kbd.key_cap(c), Catppuccin::new().sapphire);

        let timeline = EffectTimeline::builder().effect(&e).build();
        timeline.save_to_file("effect_timeline. txt", 120).unwrap();

        app.register_effect(e);
    });

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
    if f.area().is_empty() {
        return;
    }

    Clear.render(f.area(), f.buffer_mut());
    Block::default()
        .style(Style::default().bg(Catppuccin::new().crust))
        .render(f.area(), f.buffer_mut());

    let kbd = KeyboardWidget::new(AnsiKeyboardTklLayout::default().layout());
    kbd.render(f.area(), f.buffer_mut());

    let demo_area = Layout::horizontal([Percentage(50), Percentage(50)])
        .split(f.area())[1];
    ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}
