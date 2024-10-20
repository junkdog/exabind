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
mod ui_state;

use std::env::args;
use app::ExabindApp;

use std::io;
use ::crossterm::event::KeyCode;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::{Buffer, Color, Frame, Line, Style, Stylize, Text, Widget};
use ratatui::widgets::{Block, Clear};
use tachyonfx::{fx, CenteredShrink, Duration, Effect, Interpolation, RefCount, Shader};
use tachyonfx::widget::EffectTimeline;
use crate::effect::starting_up;
use crate::event_handler::EventHandler;
use crate::styling::Catppuccin;
use crate::tui::Tui;
use crate::widget::{AnsiKeyboardTklLayout, ColorDemoWidget, KeyCap, KeyboardLayout, KeyboardWidget};

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    let mut app = ExabindApp::new(events.sender());
    let mut ui_state = ui_state::UiState::new();
    let mut tui = Tui::new(ratatui::init(), events);

    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout::default());
    ui_state.register_kbd_effect(starting_up());

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.apply_event(event);
        });

        tui.draw(|f| {
            ui_state.apply_kbd_effects(elapsed);
            ui(f, &mut ui_state);
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

fn ui(f: &mut Frame<'_>, ui_state: &mut ui_state::UiState) {
    if f.area().is_empty() {
        return;
    }

    ui_state.render_kbd(f.buffer_mut());

    let demo_area = Layout::horizontal([Percentage(50), Percentage(50)])
        .split(f.area())[1];
    ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}
