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
mod shortcut;
mod buffer;

use app::ExabindApp;
use std::collections::{HashMap, HashSet};

use crate::effect::starting_up;
use crate::event_handler::EventHandler;
use crate::parser::jetbrains::JetbrainsKeymapSource;
use crate::tui::Tui;
use crate::ui_state::UiState;
use crate::widget::{AnsiKeyboardTklLayout, KeyCap, KeyboardLayout};
use ::crossterm::event::KeyCode;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Layout;
use ratatui::prelude::{Frame, Stylize, Widget};
use std::io;
use std::path::PathBuf;
use tachyonfx::{CenteredShrink, Duration, Shader};

fn render_goto_actions(ui_state: &mut UiState) {
    fn resolve_key_code(key_code: &KeyCode) -> KeyCode {
        // fixme: this is a mess - do something about it
        // translate shifted key_codes to their unshifted counterparts
        use KeyCode::*;

        match key_code {
            Char('"') => Char('\''),
            Char('<') => Char(','),
            Char('>') => Char('.'),
            Char('?') => Char('/'),
            Char(':') => Char(';'),
            Char('_') => Char('-'),
            Char('+') => Char('='),
            Char('{') => Char('['),
            Char('}') => Char(']'),
            Char('|') => Char('\\'),
            Char('!') => Char('1'),
            Char('@') => Char('2'),
            Char('#') => Char('3'),
            Char('$') => Char('4'),
            Char('%') => Char('5'),
            Char('^') => Char('6'),
            Char('&') => Char('7'),
            Char('*') => Char('8'),
            Char('(') => Char('9'),
            Char(')') => Char('0'),
            key_code => key_code.clone(),
        }
    };

    let key_caps: HashMap<KeyCode, KeyCap> = AnsiKeyboardTklLayout::default()
        .key_cap_lookup();

    let keymap = PathBuf::from("test/Eclipse copy.xml").parse_jetbrains_keymap();

    // let goto_actions: Vec<&Action> = keymap.valid_actions()
    let goto_keyset: HashSet<&KeyCode> = keymap.valid_actions()
        // .filter(|a| a.name().starts_with("Goto"))
        .filter(|a| !a.shortcuts().is_empty())
        .flat_map(|a| a.shortcuts())
        .flat_map(|s| s.keystroke())
        .collect();

    let goto_caps: Vec<KeyCap> = goto_keyset.into_iter()
        .map(|key_code| {
            if let Some(key_cap) = key_caps.get(&resolve_key_code(key_code)) {
                key_cap.clone()
            } else {
                panic!("Key code not found in layout: {:?}", key_code);
            }
        })
        .collect();

    ui_state.update_selected_shortcuts(&goto_caps);
}

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    let mut app = ExabindApp::new(events.sender());
    let mut ui_state = ui_state::UiState::new();
    let mut tui = Tui::new(ratatui::init(), events);

    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout::default());
    ui_state.register_kbd_effect(starting_up());
    render_goto_actions(&mut ui_state);

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.apply_event(event, &mut ui_state);
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
    // widget::ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}
