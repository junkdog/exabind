mod exabind_event;
mod widget;
mod event_handler;
mod dispatcher;
mod app;
mod input;
mod tui;
mod parser;
mod crossterm;
mod styling;
mod ui_state;
mod shortcut;
mod keymap;
mod stateful_widgets;
mod fx;
mod color_cycle;

use app::ExabindApp;

use crate::app::KeyMapContext;
use crate::fx::effect::{open_all_categories, starting_up};
use crate::event_handler::EventHandler;
use crate::keymap::IntoKeyMap;
use crate::parser::jetbrains::JetbrainsKeymapSource;
use crate::parser::kde::parse_kglobalshortcuts;
use crate::stateful_widgets::StatefulWidgets;
use crate::styling::{ExabindTheme, Theme, CATPPUCCIN};
use crate::tui::Tui;
use crate::widget::{AnsiKeyboardTklLayout, KeyboardLayout, ShortcutsWidget};
use ::crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use ::crossterm::execute;
use ::crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Layout;
use ratatui::prelude::{Frame, StatefulWidget, Stylize};
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidgetRef};
use ratatui::Terminal;
use std::io;
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use tachyonfx::{CenteredShrink, Duration, Shader};

fn shortcut_widget(context: &KeyMapContext, category: &str) -> ShortcutsWidget {
    let (category_idx, actions) = context.filtered_actions_by_category(category);
    let base_color = Theme.shortcuts_base_color(category_idx);

    ShortcutsWidget::new(
        category.to_string(),
        Theme.shortcuts_widget_keystroke(),
        Theme.shortcuts_widget_label(),
        base_color,
        actions
    )
}

fn shortcut_widgets(context: &KeyMapContext) -> Vec<ShortcutsWidget> {
    context.unordered_categories().iter()
        .map(|category| shortcut_widget(context, category))
        .collect()
}


fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        hook(info);
    }));
}


fn init_crossterm() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    set_panic_hook();
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(
        stdout,
        EnterAlternateScreen,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn main() -> io::Result<()> {
    let mut events = EventHandler::new(std::time::Duration::from_millis(33));
    // let keymap = PathBuf::from("test/Eclipse copy.xml").parse_jetbrains_keymap();
    // let keymap = PathBuf::from("test/default.xml").parse_jetbrains_keymap();
    let keymap = PathBuf::from("test/kglobalshortcutsrc")
        .into_keymap(parse_kglobalshortcuts);

    let mut ui_state = ui_state::UiState::new();
    let sender = events.sender();
    let mut tui = Tui::new(ratatui::init(), events);
    ui_state.screen = tui.size();
    let mut app = ExabindApp::new(&mut ui_state, sender, keymap);

    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    )?;
    // let mut tui = Tui::new(init_crossterm()?, events);

    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout::default());
    ui_state.register_kbd_effect(starting_up());
    // ui_state.render_selection_outline(app.keymap_context());

    let widgets = app.stateful_widgets().category_widgets();
    let animate_category_widgets = open_all_categories(app.sender(), widgets);
    app.stage_mut().add_effect(animate_category_widgets);

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.apply_event(event, &mut ui_state);
        });

        tui.draw(|f| {
            ui_state.apply_kbd_effects(elapsed);
            ui(f, app.stateful_widgets(), app.keymap_context(), &mut ui_state);
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
    app.process_effects(elapsed, buf, area);
}

fn ui(
    f: &mut Frame<'_>,
    stateful_widgets: &StatefulWidgets,
    keymap_context: &KeyMapContext,
    ui_state: &mut ui_state::UiState
) {
    ui_state.screen = f.area().as_size();
    if f.area().is_empty() || f.area().width == 2500 || f.area().height < 3 {
        return;
    }

    Block::new()
        .style(Style::new().bg(CATPPUCCIN.crust))
        .render(f.area(), f.buffer_mut());

    ui_state.render_kbd(f.buffer_mut());

    let area = f.area();

    // shortcuts window
    stateful_widgets.shortcuts
        .iter()
        .for_each(|w| w.render_ref(area, f.buffer_mut(), &mut ui_state.shortcuts));

    let demo_area = Layout::horizontal([Percentage(50), Percentage(50)])
        .split(f.area())[1];
    use ratatui::prelude::Widget;
    // widget::ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}
