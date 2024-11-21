use std::cell::Cell;
use std::sync::mpsc::Sender;
use crate::styling::{Catppuccin, ExabindTheme, Theme, CATPPUCCIN};
use crate::widget::{draw_key_border, render_border_with, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout, ShortcutsWidget};
use bit_set::BitSet;
use crossterm::event::KeyCode;
use ratatui::layout::{Margin, Position, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use std::time::Instant;
use ratatui::prelude::Buffer;
use tachyonfx::fx::{effect_fn_buf, fade_from, fade_from_fg, never_complete, parallel, prolong_start, sequence, sleep, sweep_in, Direction};
use tachyonfx::{fx, CellFilter, Duration, Effect, EffectTimer, HslConvertable, Interpolatable, Interpolation, IntoEffect, RangeSampler, SimpleRng};
use tachyonfx::fx::Direction::{DownToUp, UpToDown};
use tachyonfx::Interpolation::Linear;
use crate::app::KeyMapContext;
use crate::color_cycle::{PingPongColorCycle, RepeatingColorCycle};
use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crate::fx::EffectStage;
use crate::fx::key_cap_outline::KeyCapOutline;

pub fn selected_category(
    base_color: Color,
    area: Rect,
) -> Effect {

    let color_step: usize = 10;

    let (h, s, l) = base_color.to_hsl();

    let color_l = Color::from_hsl(h, s, 80.0);
    let color_d = Color::from_hsl(h, s, 40.0);

    let color_cycle = RepeatingColorCycle::new(base_color, &[
        (5,          color_d),
        (4,          color_l),
        (2,          Color::from_hsl((h - 20.0) % 360.0, s, (l + 20.0).min(100.0))),
        (color_step, Color::from_hsl(h, (s - 30.0).max(0.0), (l + 20.0).min(100.0))),
        (color_step, Color::from_hsl((h + 20.0) % 360.0, s, (l + 20.0).min(100.0))),
        (color_step, Color::from_hsl(h, (s + 30.0).max(0.0), (l + 20.0).min(100.0))),
    ]);

    let effect = fx::effect_fn_buf(Instant::now(), u32::MAX, move |started_at, ctx, buf| {
        let elapsed = started_at.elapsed().as_secs_f32();

        // speed n cells/s
        let idx = (elapsed * 30.0) as usize;

        let area = ctx.area;

        let mut update_cell = |(x, y): (u16, u16), idx: usize| {
            buf.cell_mut((x, y)).map(|cell| {
                cell.set_fg(color_cycle.color_at(idx).clone());
            });
        };

        (area.x..area.right()).enumerate().for_each(|(i, x)| {
            update_cell((x, area.y), idx + i);
        });

        let cell_idx_offset = area.width as usize;
        (area.y + 1..area.bottom() - 1).enumerate().for_each(|(i, y)| {
            update_cell((area.right() - 1, y), idx + i + cell_idx_offset);
        });

        let cell_idx_offset = cell_idx_offset + area.height.saturating_sub(2) as usize;
        (area.x..area.right()).rev().enumerate().for_each(|(i, x)| {
            update_cell((x, area.bottom() - 1), idx + i + cell_idx_offset);
        });

        let cell_idx_offset = cell_idx_offset + area.width as usize;
        (area.y + 1..area.bottom()).rev().enumerate().for_each(|(i, y)| {
            update_cell((area.x, y), idx + i + cell_idx_offset);
        });
    });

    effect.with_area(area)
}

pub fn animate_in_all_categories(
    sender: Sender<ExabindEvent>,
    widgets: &[ShortcutsWidget]
) -> Effect {
    let mut rng = SimpleRng::default();

    let max_open_category_delay = 150 * widgets.len() as u32;
    let effects = widgets.iter().map(|w| {
        let delay = Duration::from_millis(rng.gen_range(0..max_open_category_delay));
        let bg_color = w.bg_color();
        let area = w.area();
        prolong_start(delay, open_category(bg_color, area))
    }).collect::<Vec<_>>();

    sequence(&[
        prolong_start(300, parallel(&effects)),
        sleep(500),
        dispatch_event(sender, ExabindEvent::AutoSelectNextCategory),
    ])
}

pub fn open_category(
    bg_color: Color,
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

// note: never-ending effect
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

    let color_cycle = PingPongColorCycle::new(color_1, &[
        (40, color_2),
        (20, color_3),
    ]);

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
            colors.color_at(idx).clone()
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


/// dispatches `event` over 1ms
pub fn dispatch_event<T: Clone + 'static>(
    sender: Sender<T>,
    event: T
) -> Effect {
    effect_fn_buf((), 1, move |_, _, _| sender.dispatch(event.clone()))
}

pub fn outline_selected_category_key_caps(
    stage: &mut EffectStage,
    context: &KeyMapContext,
    buffer_size: Size,
) -> Effect {
    let buf = Buffer::empty(Rect::from((Position::default(), buffer_size)));
    let outline = KeyCapOutline::new(buf, context).into_effect();

    let color = Theme.kbd_cap_outline_category(context.sorted_category_idx().expect("no category selected"))
        .fg
        .expect("fg color");

    let keycap_outline = CellFilter::FgColor(color);

    let fx = parallel(&[
        outline,
        sweep_in(Direction::UpToDown, 40, 40, CATPPUCCIN.crust, (350, Interpolation::QuadIn))
            .with_cell_selection(keycap_outline),
    ]);

    stage.unique("key_cap_outline", fx)
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

