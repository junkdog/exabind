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
use crate::event_handler::EventHandler;
use crate::fx::effect::{open_all_categories, starting_up};
use crate::keymap::IntoKeyMap;
use crate::parser::kde::parse_kglobalshortcuts;
use crate::stateful_widgets::StatefulWidgets;
use crate::styling::{ExabindTheme, Theme, CATPPUCCIN};
use crate::tui::Tui;
use crate::widget::{AnsiKeyboardTklLayout, ShortcutsWidget};
use clap::Parser;
use ::crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use ::crossterm::execute;
use ratatui::prelude::Frame;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidgetRef};
use std::io;
use std::io::stdout;
use std::path::PathBuf;
use tachyonfx::Duration;

/// Exabind - A keyboard shortcut visualization tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to KDE global shortcuts file (typically ~/.config/kglobalshortcutsrc)
    #[arg(short, long)]
    pub shortcuts_file: Option<PathBuf>,
}

pub fn parse_args() -> Result<PathBuf, String> {
    let args = Args::parse();

    // Use provided path or fall back to default
    let shortcuts_path = args.shortcuts_file
        .unwrap_or(PathBuf::from("~/.config/kglobalshortcutsrc"));

    // Expand tilde if present
    let expanded_path = if shortcuts_path.to_string_lossy().starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let path_str = shortcuts_path.to_string_lossy().replace('~', &home.to_string_lossy());
            PathBuf::from(path_str)
        } else {
            return Err("Could not determine home directory".to_string());
        }
    } else {
        shortcuts_path
    };

    // Verify file exists
    if !expanded_path.exists() {
        return Err(format!(
            "Shortcuts file not found at: {}\nProvide path with --shortcuts-file or place file at default location",
            expanded_path.display()
        ));
    }

    Ok(expanded_path)
}

fn main() -> io::Result<()> {
    let shortcuts_path = match parse_args() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let events = EventHandler::new(std::time::Duration::from_millis(33));
    // let keymap = PathBuf::from("test/Eclipse copy.xml").parse_jetbrains_keymap();
    // let keymap = PathBuf::from("test/default.xml").parse_jetbrains_keymap();
    let keymap = shortcuts_path
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

    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout);
    ui_state.register_kbd_effect(starting_up());

    let widgets = app.stateful_widgets().category_widgets();
    let open_categories_fx = open_all_categories(app.sender(), widgets);
    app.stage_mut().add_effect(open_categories_fx);

    while app.is_running() {
        let elapsed = app.update_time();
        tui.receive_events(|event| {
            app.apply_event(event, &mut ui_state);
        });

        tui.draw(|f| {
            ui_state.apply_kbd_effects(elapsed);
            ui(f, app.stateful_widgets(), &mut ui_state);
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
    let area = f.area();
    app.process_effects(elapsed, f.buffer_mut(), area);
}

fn ui(
    f: &mut Frame<'_>,
    stateful_widgets: &StatefulWidgets,
    ui_state: &mut ui_state::UiState
) {
    use ratatui::prelude::Widget;

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

    // widget::ColorDemoWidget::new().render(demo_area, f.buffer_mut());
}


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
