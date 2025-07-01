use exabind_core::{
    parser::kde::parse_kglobalshortcuts,
    app::ExabindApp,
    ui_state,
    key_event::{KeyEvent, KeyCode as ExabindKeyCode, KeyModifiers},
    exabind_event::ExabindEvent,
};
use ratzilla::event::KeyCode;
use ratzilla::ratatui::style::Color;
use ratzilla::ratatui::Terminal;
use ratzilla::{CanvasBackend, WebRenderer};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

fn main() -> std::io::Result<()> {
    console_error_panic_hook::set_once();
    
    // Create a simple event channel for WASM
    let (sender, _receiver) = mpsc::channel::<ExabindEvent>();
    
    // Mock keymap for now - in a real implementation you'd load this differently
    let keymap = parse_kglobalshortcuts("");
    
    let mut ui_state = ui_state::UiState::new();
    let app = Rc::new(RefCell::new(ExabindApp::new(&mut ui_state, sender.clone(), keymap)));
    
    // Create backend with canvas size and background color
    let mut backend = CanvasBackend::new_with_size(1600, 900)?;
    backend.set_background_color(Color::Rgb(30, 30, 46)); // Catppuccin base
    let terminal = Terminal::new(backend)?;
    
    // Set up key event handling
    let sender_clone = sender.clone();
    terminal.on_key_event(move |event| {
        if let Some(key_event) = map_key_event(&event.code) {
            let _ = sender_clone.send(ExabindEvent::KeyPress(key_event));
            
            // Handle basic navigation keys
            match event.code {
                KeyCode::Char('q') => {
                    let _ = sender_clone.send(ExabindEvent::Shutdown);
                }
                KeyCode::Up => {
                    let _ = sender_clone.send(ExabindEvent::PreviousCategory);
                }
                KeyCode::Down => {
                    let _ = sender_clone.send(ExabindEvent::NextCategory);
                }
                KeyCode::Esc => {
                    let _ = sender_clone.send(ExabindEvent::DeselectCategory);
                }
                _ => {}
            }
        }
    });
    
    // Start the terminal with basic UI
    terminal.draw_web(move |frame| {
        // TODO: Implement UI rendering here
        // For now, just clear the screen
    });
    
    Ok(())
}

fn map_key_event(key: &KeyCode) -> Option<KeyEvent> {
    let modifiers = KeyModifiers::empty();
    
    // Map key codes from ratzilla to exabind
    let code = match key {
        KeyCode::Esc => ExabindKeyCode::Esc,
        KeyCode::Enter => ExabindKeyCode::Enter,
        KeyCode::Tab => ExabindKeyCode::Tab,
        KeyCode::Backspace => ExabindKeyCode::Backspace,
        KeyCode::Delete => ExabindKeyCode::Delete,
        KeyCode::Home => ExabindKeyCode::Home,
        KeyCode::End => ExabindKeyCode::End,
        KeyCode::PageUp => ExabindKeyCode::PageUp,
        KeyCode::PageDown => ExabindKeyCode::PageDown,
        KeyCode::Up => ExabindKeyCode::Up,
        KeyCode::Down => ExabindKeyCode::Down,
        KeyCode::Left => ExabindKeyCode::Left,
        KeyCode::Right => ExabindKeyCode::Right,
        KeyCode::Char(c) => ExabindKeyCode::Char(*c),
        KeyCode::F(n) => ExabindKeyCode::F(*n),
        _ => return None, // Unsupported key
    };
    
    Some(KeyEvent::new(code, modifiers))
}