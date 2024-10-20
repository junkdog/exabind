use crossterm::event::KeyCode;
use crate::styling::Catppuccin;
use crate::widget::{KeyCap, KeyboardLayout, KeyboardWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};
use tachyonfx::{ref_count, BufferRenderer, Duration, Effect, RefCount, Shader};
use crate::parser::jetbrains::Action;

pub struct UiState {
    kbd: KeyboardState,
}

struct KeyboardState {
    buf_base: RefCount<Buffer>,
    buf_work: RefCount<Buffer>,
    buf_shortcuts: RefCount<Buffer>,
    buf_shortcuts_visible: bool,
    effects: Vec<Effect>
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
                effects: Vec::new()
            }
        }
    }

    pub fn reset_kbd_buffer<K: KeyboardLayout>(&self, kbd: K) {
        let mut buf = self.kbd.buf_base.borrow_mut();

        let area = buf.area.clone();
        Block::default()
            .style(Style::default().bg(Catppuccin::new().crust))
            .render(area, &mut buf);

        let kbd = KeyboardWidget::new(kbd.layout());
        kbd.render(buf.area, &mut buf);
    }

    pub fn apply_kbd_effects(&mut self, elapsed: Duration) {
        // copy base buffer to work buffer
        self.update_kbd_buffer();

        let mut buf = self.kbd.buf_work.borrow_mut();
        let area = buf.area.clone();

        for effect in self.kbd.effects.iter_mut() {
            effect.process(elapsed, &mut buf, area);
        }

        self.kbd.effects.retain(Effect::running);
    }

    pub fn register_kbd_effect(&mut self, effect: Effect) {
        self.kbd.effects.push(effect);
    }

    pub fn render_kbd(&self, destination: &mut Buffer) {
        self.kbd.buf_work.borrow()
            .render_buffer(Offset::default(), destination);
    }

    fn update_kbd_buffer(&mut self) {
        let mut buf = self.kbd.buf_work.borrow_mut();
        self.kbd.buf_base.borrow()
            .render_buffer(Offset::default(), &mut buf);

        if self.kbd.buf_shortcuts_visible {
            self.kbd.buf_shortcuts.borrow()
                .render_buffer(Offset::default(), &mut buf);
        }
    }

    pub fn update_selected_shortcuts(
        &mut self,
        shortcuts: &[KeyCap]
    ) {
        let colors = Catppuccin::new();
        let cap_style = Style::default().fg(colors.base);
        let border_style = Style::default().fg(colors.sapphire);

        let mut buf = self.kbd.buf_shortcuts.borrow_mut();
        let area = buf.area.clone();

        let kbd = KeyboardWidget::new_with_style(shortcuts.iter().map(|s| s.clone()).collect(), cap_style, border_style);
        kbd.render(area, &mut buf);
    }

    pub fn toggle_highlight_shortcuts(&mut self) {
        self.kbd.buf_shortcuts_visible = !self.kbd.buf_shortcuts_visible;
    }
}