use crate::fx::EffectStage;
use crate::styling::{ExabindTheme, Theme, CATPPUCCIN};
use crate::widget::{KeyCap, KeyboardLayout, KeyboardWidget, ShortcutsWidgetState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect, Size};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, Widget};
use tachyonfx::{ref_count, BufferRenderer, Duration, Effect, RefCount};

pub struct UiState {
    pub screen: Size,
    kbd: KeyboardState,
    pub shortcuts: ShortcutsWidgetState,
}

struct KeyboardState {
    buf_base: RefCount<Buffer>,
    buf_work: RefCount<Buffer>,
    buf_shortcuts: RefCount<Buffer>,
    effects: EffectStage, // needs to run prior to main buffer effect stage
    active_modifiers: Vec<KeyCap>,
    offset: Offset
}

impl UiState {
    pub fn new() -> Self {
        let area = Rect::new(0, 0, 95, 14);
        Self {
            kbd: KeyboardState {
                buf_base: ref_count(Buffer::empty(area)),
                buf_work: ref_count(Buffer::empty(area)),
                buf_shortcuts: ref_count(Buffer::empty(area)),
                effects: EffectStage::default(),
                active_modifiers: Vec::new(),
                offset: Offset::default(),
            },
            shortcuts: ShortcutsWidgetState { table_state: Default::default() },
            screen: Size::default(),
        }
    }

    pub fn category_area(&self) -> Rect {
        let row_offset = 15;
        Rect::new(0, row_offset, self.screen.width, self.screen.height.saturating_sub(row_offset))
    }

    pub fn kbd_size(&self) -> Size {
        self.kbd.buf_base.borrow().area.as_size()
    }

    pub fn set_kbd_offset(&mut self, offset: Offset) {
        self.kbd.offset = offset;
    }

    pub fn reset_kbd_buffer<K: KeyboardLayout>(&self, kbd: K) {
        let mut buf = self.kbd.buf_base.borrow_mut();

        let area = buf.area.clone();
        Block::default()
            .style(Theme.kbd_surface())
            .render(area, &mut buf);

        let kbd = KeyboardWidget::new(kbd.layout());
        kbd.render(buf.area, &mut buf);
    }

    pub fn update_active_modifiers(&mut self, modifiers: Vec<KeyCap>) {
        self.kbd.active_modifiers = modifiers;
    }

    pub fn apply_kbd_effects(&mut self, elapsed: Duration) {
        // copy base buffer to work buffer
        self.update_kbd_buffer();

        let mut buf = self.kbd.buf_work.borrow_mut();
        let area = buf.area.clone();

        self.kbd.effects.process_effects(elapsed, &mut buf, area);

        KeyboardWidget::new_with_style(
            self.kbd.active_modifiers.clone(),
            Style::default().fg(CATPPUCCIN.peach).bg(CATPPUCCIN.surface0).add_modifier(Modifier::BOLD),
            None,
        ).render(area, &mut buf);
    }

    pub fn kbd_effects_mut(&mut self) -> &mut EffectStage {
        &mut self.kbd.effects
    }

    pub fn register_kbd_effect(&mut self, effect: Effect) {
        self.kbd.effects.add_effect(effect);
    }

    pub fn render_kbd(&self, destination: &mut Buffer) {
        self.kbd.buf_work.borrow()
            .render_buffer(self.kbd.offset, destination);
    }

    fn update_kbd_buffer(&mut self) {
        let mut buf = self.kbd.buf_work.borrow_mut();
        self.kbd.buf_base.borrow()
            .render_buffer(Offset::default(), &mut buf);
    }
}
