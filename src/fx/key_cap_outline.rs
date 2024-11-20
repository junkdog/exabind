use std::collections::HashMap;
use bit_set::BitSet;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Offset, Position, Rect};
use ratatui::style::Style;
use tachyonfx::{blit_buffer, CellFilter, Duration, Effect, Shader};
use crate::app::{BoundShortcut, KeyMapContext};
use crate::styling::{ExabindTheme, Theme};
use crate::widget::{draw_key_border, render_border_with, supplant_key_code, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout};

#[derive(Clone)]
pub struct KeyCapOutline {
    buffer: Buffer, // blit to buffer
    cell_filter: Option<CellFilter>,
}

impl KeyCapOutline {
    pub fn new(
        buffer: Buffer,
        context: &KeyMapContext,
    ) -> Self {
        let style = Theme.kbd_cap_outline_category(context.sorted_category_idx());
        let mut buffer = buffer;
        Self::update_shortcuts_outline(&mut buffer, context.filtered_actions(), style);

        Self {
            buffer,
            cell_filter: None
        }
    }
}

impl Shader for KeyCapOutline {
    fn name(&self) -> &'static str {
        "key_cap_outline"
    }

    fn execute(&mut self, _: Duration, _area: Rect, buf: &mut Buffer) {
        blit_buffer(&self.buffer, buf, Offset::default());
    }

    fn done(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Shader> {
        Box::new(self.clone())
    }

    fn area(&self) -> Option<Rect> {
        Some(self.buffer.area().clone())
    }

    fn set_area(&mut self, _area: Rect) {
        // no-op; consider supporting via skip=true
    }

    fn set_cell_selection(&mut self, filter: CellFilter) {
        self.cell_filter = Some(filter)
    }
}

impl KeyCapOutline {
    fn update_shortcuts_outline(
        buf: &mut Buffer,
        shortcuts: Vec<BoundShortcut>,
        style: Style,
    ) {
        let key_caps: HashMap<KeyCode, KeyCap> = AnsiKeyboardTklLayout::default()
            .key_cap_lookup();

        let keys_to_outline: Vec<KeyCap> = shortcuts
            .iter()
            .filter(|action| action.enabled_in_ui())
            .map(|action| action.shortcut())
            .flat_map(|shortcut| shortcut.keystroke())
            .filter_map(|key_code| key_caps.get(&supplant_key_code(key_code.clone())))
            .map(|key_cap| key_cap.clone())
            .collect();


        let mut key_caps: Vec<KeyCap> = keys_to_outline.iter()
            .map(|s| s.clone())
            .collect();

        let area = buf.area.clone();
        let keycap_cmp = |key_cap: &KeyCap| -> u32 {
            let a = key_cap.area;
            let width = area.width as u32;
            a.x as u32 + (a.y as u32 * width)
        };

        key_caps.sort_by(|a, b| keycap_cmp(a).cmp(&keycap_cmp(b)));
        key_caps.dedup();

        outline_key_cap_borders(&key_caps, style)
            .process(Duration::from_millis(17), buf, area);
    }
}

// one-off effect
fn outline_key_cap_borders(key_caps: &[KeyCap], border_style: Style) -> Effect {
    use tachyonfx::fx::*;

    let key_caps = key_caps.iter().map(|k| k.clone()).collect::<Vec<_>>();
    effect_fn_buf((), Duration::from_millis(1), move |_state, ctx, buf| {
        let key_caps = key_caps.clone();

        let area = buf.area.clone();
        area.positions().for_each(|pos| {
            buf.cell_mut(pos).map(|c| c.skip = true);
        });

        let area_width = buf.area.right() as isize;
        let cell_bits = buf.area.bottom() as isize * area_width;
        let mut key_cap_cells = BitSet::with_capacity(cell_bits as usize);
        render_border_with(&key_caps, buf, move |d, pos, cell| {
            draw_key_border(d, cell);
            cell.set_style(border_style);
            cell.skip = false;
        });

        key_caps.iter()
            .map(|k| k.area)
            .flat_map(|a| a.positions())
            .for_each(|pos| {
                let idx = index_of_pos(area, pos);
                key_cap_cells.insert(idx);
            });

        let neighbors = |pos| -> [bool; 4] {
            let mut neighbors = [false; 4];
            let idx = index_of_pos(area, pos) as isize;

            let is_set = |idx: isize| -> bool {
                idx >= 0 && key_cap_cells.contains(idx as usize)
            };

            if pos.x > 0 && pos.x > area.left() {
                neighbors[0] = is_set(idx - area_width - 1);
                neighbors[2] = is_set(idx + area_width - 1);
            }
            if pos.x < (area.right() - 1) as _ {
                neighbors[1] = is_set(idx - area_width + 1);
                neighbors[3] = is_set(idx + area_width + 1);
            }

            neighbors
        };

        area.positions().for_each(|pos| {
            let mut cell = &mut buf[pos];

            match (cell.symbol(), neighbors(pos)) {
                (ch, [true, true, true, true]) if !"│ ".contains(ch) => {
                    // fkey and number rows have adjacent borders, so we need to
                    // make sure to not clear the border between them...
                    if (2..=3).contains(&pos.y) {
                        cell.skip = false;
                        cell.set_char('─');
                    } else {
                        cell.skip = true;
                        cell.set_char('X');
                    }
                },
                ("╨", [true, true, _, _]) => {
                    cell.skip = false;
                    cell.set_char('─');
                },
                ("╥", [_, _, true, true]) => {
                    cell.skip = false;
                    cell.set_char('─');
                },
                ("┬", [true, true, true, false]) => {
                    cell.skip = false;
                    cell.set_char('┌');
                },
                ("┬", [true, true, false, true]) => {
                    cell.skip = false;
                    cell.set_char('┐');
                },
                ("┴", [true, false, true, true]) => {
                    cell.skip = false;
                    cell.set_char('└');
                },
                ("┴", [false, true, true, true]) => {
                    cell.skip = false;
                    cell.set_char('┘');
                },
                ("┤", [true, false, true, false]) => {
                    cell.skip = false;
                    cell.set_char('│');
                },
                ("├", [false, true, false, true]) => {
                    cell.skip = false;
                    cell.set_char('│');
                },
                ("╫", [true, false, true, true])  => {
                    cell.skip = false;
                    cell.set_char('└');
                },
                ("╫", [false, true, true, true]) => {
                    cell.skip = false;
                    cell.set_char('┘');
                },
                _ => {
                    // cell.skip = false;
                }
            }
        });
    })
}

const fn index_of_pos(area: Rect, position: Position) -> usize {
    let y = (position.y - area.y) as usize;
    let x = (position.x - area.x) as usize;
    let width = area.width as usize;
    y * width + x
}