use crossterm::event::ModifierKeyCode;
use ratatui::crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum ExabindEvent {
    Tick,
    Resize(u16, u16),
    Shutdown,
    /// A key event.
    KeyPress(KeyEvent),
    StartupAnimation,
    SelectedCategoryFxSandbox,
    AutoSelectCategory,
    DeselectCategory,
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
