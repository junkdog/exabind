use crate::styling::{Catppuccin, ExabindTheme, Theme, CATPPUCCIN};
use crate::widget::{draw_key_border, render_border_with, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout, ShortcutsWidget};
use bit_set::BitSet;
use crossterm::event::KeyCode;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use std::time::Instant;
use tachyonfx::fx::{never_complete, parallel, prolong_start, sequence};
use tachyonfx::{fx, CellFilter, Duration, Effect, EffectTimer, Interpolatable, Interpolation, IntoEffect, RangeSampler, SimpleRng};
use tachyonfx::fx::Direction::{DownToUp, UpToDown};
use tachyonfx::Interpolation::Linear;

pub fn fill_bartilt<T: Into<EffectTimer>>(timer: T) -> Effect {
    let rng = SimpleRng::default();
    let fill = fx::effect_fn((), timer, move |_state, ctx, cells| {
        cells
            .enumerate()
            .filter(|(_, (_, cell))| cell.symbol() == "▔")
            .for_each(|(idx, (_, cell))| {
                match idx % 2 {
                    0 => cell.set_char('◤'),
                    1 => cell.set_char('◢'),
                    _ => unreachable!(),
                };
                let s = cell.style();
                cell.set_style(s.add_modifier(Modifier::REVERSED));
            });
    });

    never_complete(fill)
}

pub fn animate_in_all_categories(
    widgets: &[ShortcutsWidget]
) -> Effect {
    let mut rng = SimpleRng::default();

    let max_open_category_delay = 150 * widgets.len() as u32;
    let effects = widgets.iter().map(|w| {
        let delay = Duration::from_millis(rng.gen_range(0..max_open_category_delay));
        let bg_color = w.bg_color();
        let border_color = w.border_color();
        let area = w.area();
        prolong_start(delay, open_category(bg_color, border_color, area))
    }).collect::<Vec<_>>();

    fx::prolong_start(300, parallel(&effects))
}

pub fn open_category(
    bg_color: Color,
    border_color: Color,
    area: Rect,
) -> Effect {
    use tachyonfx::{fx::*, Interpolation::*};

    let h = area.height as u32;
    let timer: EffectTimer = (200 + h * 10, Linear).into();
    let timer_c: EffectTimer = (200 + h * 10, ExpoOut).into();

    let border_cells = CellFilter::Outer(Margin::new(1, 1));
    let content_cells = CellFilter::Inner(Margin::new(1, 1));

    parallel(&[
        sequence(&[
            // prolong_start(timer, fade_from_fg(bg_color, timer_c))
            prolong_start(timer, sweep_in(UpToDown, area.height, 0, bg_color, timer))
                .with_cell_selection(content_cells.clone()),
        ]),
        // prolong_start(timer, fade_from_fg(bg_color, timer_c))
        prolong_start(timer, coalesce(timer_c))
            .with_cell_selection(border_cells),
        slide_in(UpToDown, area.height * 2, 0, CATPPUCCIN.crust, timer),
    ]).with_area(area)
}

pub fn key_press<C: Into<Color>>(
    key_press_delay: Duration,
    key: KeyCap,
    color: C
) -> Effect {
    use tachyonfx::fx::*;

    // border
    let key_borders = CellFilter::Outer(Margin::new(1, 1));
    let key_pad = CellFilter::Inner(Margin::new(0, 0));

    let c = color.into();
    let bg = Catppuccin::new().crust;

    parallel(&[
        // redraw singular border around key
        delay(key_press_delay, parallel(&[
            clear_cells(Duration::from_millis(750)),
            draw_single_border(key.clone(), Duration::from_millis(750)),
        ])).with_cell_selection(key_borders),
        // "click" fade; faded out during key_press_delay
        sequence(&[
            prolong_start(key_press_delay,
                fade_to(c, bg, (50, Interpolation::Linear))),
            fade_from(c, bg, (700, Interpolation::SineOut)),
        ]),
    ]).with_area(key.area)
}

pub fn starting_up() -> Effect {
    let kbd = AnsiKeyboardTklLayout::default();
    let esc_area = kbd.key_area(KeyCode::Enter);

    let mut effects = vec![];

    let mut rng = SimpleRng::default();
    let initial_delay = Duration::from_millis(300);
    let mut accrued_delay = initial_delay.as_millis();

    "exabind".char_indices().enumerate().for_each(|(i, (_, c))| {
        let delta: u32 = rng.gen_range(100..200);
        accrued_delay += delta;

        let e = key_press(Duration::from_millis(accrued_delay), kbd.key_cap(c), Theme.kbd_key_press_color());
        effects.push(e);
    });

    accrued_delay += 300;
    let e = key_press(
        Duration::from_millis(accrued_delay),
        KeyCap::new(KeyCode::Enter, esc_area),
        Theme.kbd_key_press_color()
    );
    effects.push(e);

    effects.push(fx::delay(accrued_delay + 200, fx::parallel(&[
        fx::never_complete(led_kbd_border()),
        fx::fade_from_fg(CATPPUCCIN.crust, (800, Interpolation::SineOut))
    ])));

    fx::parallel(&effects)
}

pub fn fade_in_keys() -> Effect {
    use tachyonfx::{fx::*, CellFilter::*};

    let c = Catppuccin::new();
    let color_cap = c.surface0;
    let color_border = c.mauve;

    parallel(&[
        prolong_start(700, never_complete(fade_to_fg(color_cap, (1500, Interpolation::SineIn))))
            .with_cell_selection(Text),
        never_complete(fade_to_fg(color_border, (1500, Interpolation::SineIn)))
            .with_cell_selection(Not(Text.into())),
    ])
}

pub fn led_kbd_border() -> Effect {
    use tachyonfx::{fx::*, CellFilter::*};

    let [color_1, color_2, color_3] = Theme.kbd_led_colors();

    let mut color_cycle: Vec<Color> = vec![];
    (0..40).for_each(|i| {
        let color = color_1.lerp(&color_2, i as f32 / 39.0);
        color_cycle.push(color);
    });
    (0..40).for_each(|i| {
        let color = color_2.lerp(&color_3, i as f32 / 39.0);
        color_cycle.push(color);
    });
    let mut color_cycle_reversed = color_cycle.iter().rev().cloned().collect::<Vec<_>>();
    color_cycle.append(&mut color_cycle_reversed);

    let initial_state = (color_cycle, None);
    effect_fn_buf(initial_state, Duration::from_millis(1), |(colors, started_at), _ctx, buf| {
        if started_at.is_none() {
            *started_at = Some(Instant::now());
        }

        let area = buf.area.clone();

        let elapsed = started_at.as_ref().unwrap().elapsed().as_millis().max(1);
        let raw_color_idx = (elapsed / 100) as u32;

        let color = |pos: Position| -> Color {
            let idx = (raw_color_idx + (pos.x / 2 + pos.y * 3 / 2) as u32) as usize;
            colors[idx % colors.len()]
        };

        area.positions().for_each(|pos| {
            let cell = buf.cell_mut(pos).unwrap();
            if let Some(ch) = cell.symbol().chars().next() {
                if !is_box_drawing(ch) && ch != ' ' {
                    cell.set_fg(color(pos));
                }
            }
        });
    }).with_cell_selection(Outer(Margin::new(1, 1)))

}

pub fn outline_border(key_caps: &[KeyCap], border_style: Style) -> Effect {
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

fn draw_single_border(key_cap: KeyCap, duration: Duration) -> Effect {
    use tachyonfx::fx::*;
    let border_style = Style::default().fg(Catppuccin::new().base);

    effect_fn_buf((), duration, move |_state, ctx, buf| {
        render_border_with(&[key_cap.clone()], buf, move |d, pos, cell| {
            draw_key_border(d, cell);
            cell.set_style(border_style);
        });
    })
}

fn clear_cells(duration: Duration) -> Effect {
    use tachyonfx::fx::*;
    effect_fn((), duration, |_state, _ctx, cells| {
        cells.for_each(|(_, cell)| {
            cell.set_char(' ');
        });
    })
}

fn is_box_drawing(c: char) -> bool {
    ('\u{2500}'..='\u{257F}').contains(&c)
}

const fn index_of_pos(area: Rect, position: Position) -> usize {
    let y = (position.y - area.y) as usize;
    let x = (position.x - area.x) as usize;
    let width = area.width as usize;
    y * width + x
}