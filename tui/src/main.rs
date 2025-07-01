use exabind_core::{
    parse_args, IntoKeyMap, parser::kde::parse_kglobalshortcuts,
    app::ExabindApp,
    event_handler::EventHandler,
    fx::effect::{open_all_categories, starting_up},
    stateful_widgets::StatefulWidgets,
    styling::CATPPUCCIN,
    tui::Tui,
    widget::AnsiKeyboardTklLayout,
    ui_state,
};
use ::crossterm::event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use ::crossterm::execute;
use ratatui::prelude::Frame;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidgetRef};
use std::io;
use std::io::stdout;
use tachyonfx::Duration;

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
}

