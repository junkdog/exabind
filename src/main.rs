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
use crate::parser::jetbrains::{JetbrainsKeymapSource, KeyMap};
use crate::tui::Tui;
use crate::ui_state::UiState;
use crate::widget::{resolve_key_code, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout, ShortcutCategoriesWidget, ShortcutsWindow};
use ::crossterm::event::KeyCode;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::prelude::{Frame, Stylize, StatefulWidget};
use std::io;
use std::path::PathBuf;
use ratatui::style::{Color, Style};
use ratatui::widgets::{StatefulWidgetRef, TableState};
use tachyonfx::{CenteredShrink, Duration, Shader};

fn render_goto_actions(keymap: &KeyMap, ui_state: &mut UiState) {
    let key_caps: HashMap<KeyCode, KeyCap> = AnsiKeyboardTklLayout::default()
        .key_cap_lookup();

    // let keymap = PathBuf::from("test/Eclipse copy.xml").parse_jetbrains_keymap();
    // let keymap = PathBuf::from("test/default.xml").parse_jetbrains_keymap();

    // let goto_actions: Vec<&Action> = keymap.valid_actions()
    let goto_keyset: HashSet<&KeyCode> = keymap.valid_actions()
        .filter(|(cat, _)| *cat == "navigate")
        .filter(|(_, a)| !a.shortcuts().is_empty())
        .flat_map(|(_, a)| a.shortcuts())
        .flat_map(|s| s.keystroke())
        .collect();

    let goto_caps: Vec<KeyCap> = goto_keyset.into_iter()
        .map(|key_code| {
            if let Some(key_cap) = key_caps.get(&resolve_key_code(key_code.clone())) {
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
    let keymap = PathBuf::from("test/Eclipse copy.xml").parse_jetbrains_keymap();
    let mut app = ExabindApp::new(events.sender(), keymap);
    let mut ui_state = ui_state::UiState::new();
    let mut tui = Tui::new(ratatui::init(), events);

    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout::default());
    ui_state.register_kbd_effect(starting_up());
    render_goto_actions(app.keymap(), &mut ui_state);

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.apply_event(event, &mut ui_state);
        });

        tui.draw(|f| {
            ui_state.apply_kbd_effects(elapsed);
            ui(f, app.keymap(), &mut ui_state);
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

fn ui(
    f: &mut Frame<'_>,
    keymap: &KeyMap,
    ui_state: &mut ui_state::UiState
) {
    if f.area().is_empty() {
        return;
    }

    ui_state.render_kbd(f.buffer_mut());

    let kbd_size = ui_state.kbd_size();
    let mut shortcut_area = f.area().clone()
        .offset(Offset{ x: 0, y: kbd_size.height as i32 + 1 });
    shortcut_area.height -= kbd_size.height;

    // shortcut category selection
    let category_area = Layout::horizontal([
        Constraint::Min(kbd_size.width),
        Constraint::Length(1),
        Constraint::Percentage(100),
    ]).split(f.area())[2];
    let categories = keymap.categories();
    ShortcutCategoriesWidget::new(categories)
        .render(category_area, f.buffer_mut(), &mut ui_state.shortcut_categories);

    // render shortcuts
    let actions = keymap.valid_actions()
        .filter(|(cat, _a)| cat.eq(&"navigate"))
        .map(|(_cat, a)| a.clone())
        .collect();

    ShortcutsWindow::new("navigate",
        Style::default(),
        Style::default(),
        Color::Green,
        actions
    ).render_ref(shortcut_area, f.buffer_mut(), &mut TableState::new());
    // ).render_ref(shortcut_area, f.buffer_mut(), &mut ui_state.shortcuts_state);

    let demo_area = Layout::horizontal([Percentage(50), Percentage(50)])
        .split(f.area())[1];
    // widget::ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}
