use crate::styling::Catppuccin;
use crate::widget::{KeyCap, KeyboardLayout, KeyboardWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Clear, Widget};
use tachyonfx::{ref_count, BufferRenderer, CellIterator, Duration, Effect, RefCount, Shader};
use crate::buffer::blit_buffer;
use crate::effect::outline_border;

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
            blit_buffer(&self.kbd.buf_shortcuts.borrow(), &mut buf, Offset::default());
            // self.kbd.buf_shortcuts.borrow()
            //     .render_buffer(Offset::default(), &mut buf);
        }
    }

    pub fn update_selected_shortcuts(
        &mut self,
        shortcuts: &[KeyCap]
    ) {
        let colors = Catppuccin::new();
        let cap_style = Style::default().fg(colors.base);
        let border_style = Style::default().fg(colors.lavender);

        let mut buf = self.kbd.buf_shortcuts.borrow_mut();
        let area = buf.area.clone();
        Clear.render(area, &mut buf);
        let default_style = buf.cell((area.x, area.y)).unwrap().style().clone();

        let mut key_caps: Vec<KeyCap> = shortcuts.iter()
            .map(|s| s.clone())
            .collect();


        fn keycap_sort_value(key_cap: &KeyCap) -> u32 {
            let a = key_cap.area;
            a.left() as u32 + (a.top() as u32 * a.width as u32)
            // a.x as u32 + (a.y as u32 * a.width as u32)
        }

        key_caps.sort_by(|a, b| keycap_sort_value(a).cmp(&keycap_sort_value(b)));

        // let kbd = KeyboardWidget::new_with_style(key_caps, cap_style, border_style);
        // kbd.render(area, &mut buf);
        outline_border(&key_caps, border_style)
            .process(Duration::from_millis(17), &mut buf, area);

        // mark all cells with default style as skip
        // CellIterator::new(&mut buf, area,  None)
        //     .filter(|(_, c)| c.symbol() == " " && c.style() == default_style)
        //     .for_each(|(pos, c)| c.skip = true);
    }

    pub fn toggle_highlight_shortcuts(&mut self) {
        self.kbd.buf_shortcuts_visible = !self.kbd.buf_shortcuts_visible;
    }
}