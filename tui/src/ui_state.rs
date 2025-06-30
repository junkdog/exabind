use crate::styling::{ExabindTheme, Theme, CATPPUCCIN};
use crate::widget::{KeyCap, KeyboardLayout, KeyboardWidget, ShortcutsWidgetState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect, Size};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Widget};
use tachyonfx::{ref_count, BufferRenderer, Duration, Effect, EffectManager, RefCount};
use crate::fx::effect::UniqueEffectId;

/// Represents the overall UI state of the application, managing the screen dimensions,
/// keyboard state, and shortcuts widget state.
pub struct UiState {
    /// Current screen dimensions
    pub screen: Size,
    /// Internal state for keyboard rendering and effects
    kbd: KeyboardState,
    /// State for the shortcuts widget
    pub shortcuts: ShortcutsWidgetState,
}

/// Internal state for managing keyboard rendering, effects, and active modifiers
struct KeyboardState {
    /// Base buffer containing the initial keyboard layout
    buf_base: RefCount<Buffer>,
    /// Working buffer for applying effects and modifications
    buf_work: RefCount<Buffer>,
    /// Effect stage for keyboard animations and visual effects
    /// Note: Must be processed before the main buffer effect stage
    effects: EffectManager<UniqueEffectId>,
    /// Currently active modifier keys
    active_modifiers: Vec<KeyCap>,
    /// Current offset for keyboard rendering position
    offset: Offset
}
impl UiState {
    pub fn new() -> Self {
        let area = Rect::new(0, 0, 95, 14);
        Self {
            kbd: KeyboardState {
                buf_base: ref_count(Buffer::empty(area)),
                buf_work: ref_count(Buffer::empty(area)),
                effects: EffectManager::default(),
                active_modifiers: Vec::new(),
                offset: Offset::default(),
            },
            shortcuts: ShortcutsWidgetState { table_state: Default::default() },
            screen: Size::default(),
        }
    }

    pub fn kbd_size(&self) -> Size {
        self.kbd.buf_base.borrow().area.as_size()
    }

    /// Sets the rendering offset for the keyboard.
    ///
    /// # Arguments
    /// * `offset` - The new offset position for rendering the keyboard
    pub fn set_kbd_offset(&mut self, offset: Offset) {
        self.kbd.offset = offset;
    }

    /// Resets the keyboard buffer with a new keyboard layout.
    ///
    /// # Arguments
    /// * `kbd` - Any type implementing the KeyboardLayout trait
    ///
    /// Initializes the base buffer with the theme surface style and renders
    /// the new keyboard layout.
    pub fn reset_kbd_buffer<K: KeyboardLayout>(&self, kbd: K) {
        let mut buf = self.kbd.buf_base.borrow_mut();

        let area = buf.area;
        Block::default()
            .style(Theme.kbd_surface())
            .render(area, &mut buf);

        let kbd = KeyboardWidget::new(kbd.layout());
        kbd.render(buf.area, &mut buf);
    }

    /// Updates the list of currently active modifier keys.
    ///
    /// # Arguments
    /// * `modifiers` - Vector of KeyCap representing active modifier keys
    pub fn update_active_modifiers(&mut self, modifiers: Vec<KeyCap>) {
        self.kbd.active_modifiers = modifiers;
    }

    pub fn apply_kbd_effects(&mut self, elapsed: Duration) {
        // copy base buffer to work buffer
        self.update_kbd_work_buffer();

        let mut work_buf = self.kbd.buf_work.borrow_mut();
        let area = work_buf.area;

        // process effects (led effects, shortcut outlines, etc)
        self.kbd.effects.process_effects(elapsed, &mut work_buf, area);

        // render active modifiers
        KeyboardWidget::new_with_style(
            self.kbd.active_modifiers.clone(),
            Style::default().fg(CATPPUCCIN.peach).bg(CATPPUCCIN.surface0).add_modifier(Modifier::BOLD),
            None,
        ).render(area, &mut work_buf);
    }

    /// Returns a mutable reference to the keyboard effects stage.
    pub fn kbd_effects_mut(&mut self) -> &mut EffectManager<UniqueEffectId> {
        &mut self.kbd.effects
    }

    pub fn register_kbd_effect(&mut self, effect: Effect) {
        self.kbd.effects.add_effect(effect);
    }

    /// Renders the work buffer to the destination buffer.
    ///
    /// # Arguments
    /// * `destination` - Target buffer to render the keyboard into
    pub fn render_kbd(&self, destination: &mut Buffer) {
        self.kbd.buf_work.borrow()
            .render_buffer(self.kbd.offset, destination);
    }

    /// Updates the work buffer with the current base buffer content.
    ///
    /// This is an internal method used to prepare the work buffer for
    /// applying effects and modifications.
    fn update_kbd_work_buffer(&mut self) {
        let mut buf = self.kbd.buf_work.borrow_mut();
        self.kbd.buf_base.borrow()
            .render_buffer(Offset::default(), &mut buf);
    }
}
