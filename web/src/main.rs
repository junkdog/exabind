use exabind_core::fx::effect::open_all_categories;
use exabind_core::stateful_widgets::StatefulWidgets;
use exabind_core::{
    app::ExabindApp,
    event_handler::EventHandler,
    exabind_event::ExabindEvent,
    fx::effect::starting_up,
    parser::kde::parse_kglobalshortcuts,
    ui_state,
    widget::AnsiKeyboardTklLayout,
};
use ratatui::widgets::StatefulWidgetRef;
use ratatui::Frame;
use ratzilla::event::KeyCode;
use ratzilla::ratatui::Terminal;
use ratzilla::{WebGl2Backend, WebRenderer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use ratzilla::backend::webgl2::{FontAtlasData, WebGl2BackendOptions};
use tachyonfx::Duration;
use exabind_core::dispatcher::Dispatcher;

fn main() -> std::io::Result<()> {
    console_error_panic_hook::set_once();

    // need an event handler for WASM
    let events = EventHandler::new(core::time::Duration::from_millis(33));
    
    // Bundle KDE shortcuts data at compile time for web
    let keymap = parse_kglobalshortcuts(include_str!("../../test/kglobalshortcutsrc"));
    
    // Create backend with size and set background color

    let backend_options = WebGl2BackendOptions::new()
        .enable_console_debug_api()
        .enable_mouse_selection()
        .font_atlas(FontAtlasData::from_binary(include_bytes!("../bitmap_font.atlas")).unwrap())
        .measure_performance(true)
        .size((1650, 760));
    let backend = WebGl2Backend::new_with_options(backend_options)?;
    let terminal = Terminal::new(backend)?;
    
    let mut ui_state = ui_state::UiState::new();
    
    // Initialize keyboard layout and startup effect
    ui_state.screen = terminal.size()?;
    ui_state.reset_kbd_buffer(AnsiKeyboardTklLayout);
    ui_state.register_kbd_effect(starting_up());

    
    let app = Rc::new(RefCell::new(ExabindApp::new(&mut ui_state, events.sender(), keymap)));
    {
        let mut app_ref = app.borrow_mut();
        let widgets = app_ref.stateful_widgets().category_widgets();
        let open_categories_fx = open_all_categories(app_ref.sender(), widgets);
        app_ref.stage_mut().add_effect(open_categories_fx);
    }
    
    // Set up key event handling
    setup_key_event_handling(&terminal, events.sender());

    // Start the terminal with basic UI
    terminal.draw_web(move |frame| {
        // Process events
        while let Some(event) = events.try_next() {
            app.borrow_mut().apply_event(event, &mut ui_state);
        }

        // Update time and get elapsed duration
        let elapsed = app.borrow_mut().update_time();

        // Apply effects
        ui_state.apply_kbd_effects(elapsed);

        // Render UI
        {
            let app_ref = app.borrow();
            let stateful_widgets = app_ref.stateful_widgets();
            ui(frame, stateful_widgets, &mut ui_state);
        }

        // Process effects
        effects(elapsed, &mut app.borrow_mut(), frame);
    });
    
    Ok(())
}

fn setup_key_event_handling(
    terminal: &Terminal<WebGl2Backend>,
    sender: Sender<ExabindEvent>,
) {
    terminal.on_key_event(move |event| {
        match event.code {
            KeyCode::Left => {
                sender.dispatch(ExabindEvent::PreviousCategory);
            }
            KeyCode::Right => {
                sender.dispatch(ExabindEvent::NextCategory);
            }
            KeyCode::Esc => {
                sender.dispatch(ExabindEvent::DeselectCategory);
            }
            _ => {}
        }
    });
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
    ui_state.screen = f.area().as_size();
    
    if f.area().is_empty() || f.area().width == 2500 || f.area().height < 3 {
        return;
    }

    ui_state.render_kbd(f.buffer_mut());

    let area = f.area();

    // shortcuts window
    stateful_widgets.shortcuts
        .iter()
        .for_each(|w| w.render_ref(area, f.buffer_mut(), &mut ui_state.shortcuts));
}

