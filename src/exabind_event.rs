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
    OpenCategoryFxSandbox,
    NextShortcut,
    PreviousShortcut,
    NextCategory,
    PreviousCategory,
    ToggleFilterKey(ModifierKeyCode),
    ActivateUiElement(UiElement),
    CategoryWidgetNavigationOrder(Vec<usize>)
}

#[derive(Debug, Clone)]
pub enum UiElement {
    Category,
    Shortcut,
}
