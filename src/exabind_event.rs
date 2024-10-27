use crossterm::event::ModifierKeyCode;
use ratatui::crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum ExabindEvent {
    Tick,
    Shutdown,
    /// A key event.
    KeyPress(KeyEvent),
    ToggleHighlightShortcuts,
    StartupAnimation,
    NextShortcut,
    PreviousShortcut,
    NextCategory,
    PreviousCategory,
    ToggleFilterKey(ModifierKeyCode),
}

