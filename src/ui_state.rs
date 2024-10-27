use std::collections::{HashMap, HashSet};
use crossterm::event::{KeyCode, ModifierKeyCode};
use crossterm::event::ModifierKeyCode::{LeftAlt, LeftControl, LeftShift};
use crate::styling::Catppuccin;
use crate::widget::{resolve_key_code, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout, KeyboardWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Rect, Size};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Clear, ListState, TableState, Widget};
use tachyonfx::{ref_count, BufferRenderer, CellIterator, Duration, Effect, RefCount, Shader};
use crate::app::KeyMapContext;
use crate::buffer::blit_buffer;
use crate::effect::outline_border;
use crate::parser::jetbrains::KeyMap;

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
                effects: Vec::new(),
            },
        }
    }

    pub fn kbd_size(&self) -> Size {
        self.kbd.buf_base.borrow().area.as_size()
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

    pub fn render_selection_outline(
        &mut self,
        category: &str,
        context: &KeyMapContext,
    ) {
        let key_caps: HashMap<KeyCode, KeyCap> = AnsiKeyboardTklLayout::default()
            .key_cap_lookup();

        let filter_shortcuts_on_selected_modifiers = context.filter_key_alt;

        let keymap = &context.keymap;
        let goto_keyset: HashSet<&KeyCode> = keymap.valid_actions()
            .filter(|(cat, _)| *cat == category)
            .filter(|(_, a)| !a.shortcuts().is_empty())
            .flat_map(|(_, a)| a.shortcuts())
            .filter(|shortcuts| !context.filter_key_control || shortcuts.uses_modifier(LeftControl))
            .filter(|shortcuts| !context.filter_key_shift   || shortcuts.uses_modifier(LeftShift))
            .filter(|shortcuts| !context.filter_key_alt     || shortcuts.uses_modifier(LeftAlt))
            .flat_map(|s| s.keystroke())
            .collect();

        let keys_to_outline: Vec<KeyCap> = goto_keyset.into_iter()
            .map(|key_code| {
                if let Some(key_cap) = key_caps.get(&resolve_key_code(key_code.clone())) {
                    key_cap.clone()
                } else {
                    panic!("Key code not found in layout: {:?}", key_code);
                }
            })
            .collect();

        self.update_selected_shortcuts_outline(&keys_to_outline);
    }

    fn update_selected_shortcuts_outline(
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

        // key_caps.sort_by(|a, b| keycap_sort_value(a).cmp(&keycap_sort_value(b)));

        // let kbd = KeyboardWidget::new_with_style(key_caps, cap_style, border_style);
        // kbd.render(area, &mut buf);
        let mut sorted_kbd_layout = AnsiKeyboardTklLayout::default().layout();
        sorted_kbd_layout.retain(|k| key_caps.iter().any(|s| s.key_code == k.key_code));

        // outline_border(&key_caps, border_style)
        outline_border(&sorted_kbd_layout, border_style)
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