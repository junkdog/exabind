use std::ops::DerefMut;
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};
use tachyonfx::{fx, ref_count, BufferRenderer, Duration, Effect, RefCount, Shader};
use tachyonfx::widget::EffectTimeline;
use crate::styling::Catppuccin;
use crate::widget::{KeyboardLayout, KeyboardWidget};

pub struct UiState {
    kbd: KeyboardState,
}

struct KeyboardState {
    buf_base: RefCount<Buffer>,
    buf_work: RefCount<Buffer>,
    effects: Vec<Effect>
}

impl UiState {
    pub fn new() -> Self {
        Self {
            kbd: KeyboardState {
                buf_base: ref_count(Buffer::empty(Rect::new(0, 0, 95, 14))),
                buf_work: ref_count(Buffer::empty(Rect::new(0, 0, 95, 14))),
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
    }
}