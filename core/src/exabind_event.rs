use crate::key_event::{KeyEvent, ModifierKeyCode};

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
    NextCategory,
    PreviousCategory,
    ToggleFilterKey(ModifierKeyCode),
    CategoryWidgetNavigationOrder(Vec<usize>)
}
