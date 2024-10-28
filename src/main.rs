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

use crate::app::KeyMapContext;
use crate::effect::starting_up;
use crate::event_handler::EventHandler;
use crate::parser::jetbrains::JetbrainsKeymapSource;
use crate::tui::Tui;
use crate::widget::{AnsiKeyboardTklLayout, KeyboardLayout, ShortcutCategoriesWidget, ShortcutsWindow};
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::{Constraint, Layout, Offset};
use ratatui::prelude::{Frame, StatefulWidget, Stylize};
use ratatui::style::{Color, Style};
use ratatui::widgets::{ListState, StatefulWidgetRef, TableState};
use std::io;
use std::path::PathBuf;
use tachyonfx::{CenteredShrink, Duration, Shader};

struct StatefulWidgets {
    shortcuts_window: ShortcutsWindow
}

impl StatefulWidgets {
    fn new() -> Self {
        Self {
            shortcuts_window: ShortcutsWindow::new(
                "Shortcuts".to_string(),
                Style::default(),
                Style::default(),
                Color::Green,
                vec![]
            )
        }
    }

    fn update_shortcut_category(
        &mut self,
        keymap_context: &KeyMapContext,
    ) {
        let selected_category = keymap_context.categories[keymap_context.current_category].clone();
        let actions = keymap_context.keymap.valid_actions()
            .filter(|(cat, _a)| *cat == &(selected_category.0))
            .map(|(_cat, a)| a.clone())
            .collect();

        self.shortcuts_window = ShortcutsWindow::new(selected_category.0,
            Style::default(),
            Style::default(),
            Color::Green,
            actions
        )
    }
}

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    // let keymap = PathBuf::from("test/Eclipse copy.xml").parse_jetbrains_keymap();
    let keymap = PathBuf::from("test/default.xml").parse_jetbrains_keymap();
    let mut app = ExabindApp::new(events.sender(), keymap);
    let mut ui_state = ui_state::UiState::new();
    let mut tui = Tui::new(ratatui::init(), events);

    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout::default());
    ui_state.register_kbd_effect(starting_up());
    ui_state.render_selection_outline(app.keymap_context().category(), app.keymap_context());

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.apply_event(event, &mut ui_state);
        });

        tui.draw(|f| {
            ui_state.apply_kbd_effects(elapsed);
            ui(f, app.widgets(), app.keymap_context(), &mut ui_state);
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
    stateful_widgets: &StatefulWidgets,
    keymap_context: &KeyMapContext,
    ui_state: &mut ui_state::UiState
) {
    if f.area().is_empty() || f.area().width == 2500 || f.area().height < 3 {
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
    let mut list_state = ListState::default().with_selected(Some(keymap_context.current_category));

    ShortcutCategoriesWidget::new(keymap_context.categories.clone())
        .render(category_area, f.buffer_mut(), &mut list_state);

    stateful_widgets
        .shortcuts_window
        .render_ref(shortcut_area, f.buffer_mut(), &mut TableState::new().with_selected(stateful_widgets.shortcuts_window.selected_shortcut));

    let demo_area = Layout::horizontal([Percentage(50), Percentage(50)])
        .split(f.area())[1];
    // widget::ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}
