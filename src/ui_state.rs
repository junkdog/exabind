use crate::app::KeyMapContext;
use crate::buffer::blit_buffer;
use crate::fx::effect::outline_border;
use crate::styling::{ExabindTheme, Theme, CATPPUCCIN};
use crate::widget::{supplant_key_code, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout, KeyboardWidget, ShortcutsWidgetState};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect, Size};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, Clear, Widget};
use std::collections::HashMap;
use tachyonfx::{ref_count, BufferRenderer, Duration, Effect, RefCount, Shader};
use crate::fx::EffectStage;

pub struct UiState {
    pub screen: Size,
    kbd: KeyboardState,
    pub shortcuts: ShortcutsWidgetState,
}

struct KeyboardState {
    buf_base: RefCount<Buffer>,
    buf_work: RefCount<Buffer>,
    buf_shortcuts: RefCount<Buffer>,
    buf_shortcuts_visible: bool,
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
                buf_shortcuts_visible: false,
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

        if self.kbd.buf_shortcuts_visible {
            blit_buffer(&self.kbd.buf_shortcuts.borrow(), &mut buf, Offset::default());
        }
    }

    pub fn render_selection_outline(
        &mut self,
        context: &KeyMapContext,
    ) {
        let style = Theme.kbd_cap_outline_category(context.sorted_category_idx());
        let key_caps: HashMap<KeyCode, KeyCap> = AnsiKeyboardTklLayout::default()
            .key_cap_lookup();

        let keys_to_outline: Vec<KeyCap> = context.filtered_actions()
            .iter()
            .filter(|action| action.enabled_in_ui())
            .map(|action| action.shortcut())
            .flat_map(|shortcut| shortcut.keystroke())
            .filter_map(|key_code| key_caps.get(&supplant_key_code(key_code.clone())))
            .map(|key_cap| key_cap.clone())
            .collect();

        self.update_selected_shortcuts_outline(&keys_to_outline, style);
    }

    fn update_selected_shortcuts_outline(
        &mut self,
        shortcuts: &[KeyCap],
        border_style: Style,
    ) {
        let mut buf = self.kbd.buf_shortcuts.borrow_mut();
        let area = buf.area.clone();
        Clear.render(area, &mut buf);

        let mut key_caps: Vec<KeyCap> = shortcuts.iter()
            .map(|s| s.clone())
            .collect();


        let keycap_sort_value = |key_cap: &KeyCap| -> u32 {
            let a = key_cap.area;
            let width = area.width as u32;
            a.x as u32 + (a.y as u32 * width)
        };

        key_caps.sort_by(|a, b| keycap_sort_value(a).cmp(&keycap_sort_value(b)));
        key_caps.dedup();

        outline_border(&key_caps, border_style)
            .process(Duration::from_millis(17), &mut buf, area);
    }

    pub fn toggle_highlight_shortcuts(&mut self) {
        self.kbd.buf_shortcuts_visible = !self.kbd.buf_shortcuts_visible;
    }
}
