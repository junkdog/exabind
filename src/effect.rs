use std::time::Instant;
use crossterm::event::KeyCode;
use crate::styling::Catppuccin;
use crate::widget::{render_border, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout};
use ratatui::layout::{Margin, Position};
use ratatui::style::{Color, Style};
use tachyonfx::{fx, CellFilter, Duration, Effect, Interpolatable, Interpolation, RangeSampler, SimpleRng};
use tachyonfx::fx::{prolong_end, prolong_start};
use crate::effect;

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
        let delta: u32 = rng.gen_range(150..300);
        accrued_delay += delta;

        let e = key_press(Duration::from_millis(accrued_delay), kbd.key_cap(c), Catppuccin::new().sapphire);
        effects.push(e);
    });
    accrued_delay += 500;
    let e = key_press(Duration::from_millis(accrued_delay), KeyCap::new(KeyCode::Enter, esc_area), Catppuccin::new().sapphire);
    effects.push(e);

    effects.push(fx::delay(accrued_delay + 600, fx::parallel(&[
        fx::never_complete(led_kbd_border()),
        fx::fade_from_fg(Catppuccin::new().crust, (1200, Interpolation::SineOut))
    ])));

    fx::parallel(&effects)
}

pub fn fade_in_keys() -> Effect {
    use tachyonfx::{CellFilter::*, fx::*};
    let key_borders = CellFilter::Outer(Margin::new(1, 1));
    let key_pad = CellFilter::Inner(Margin::new(0, 0));

    let color_cap = Catppuccin::new().surface0;
    let color_border = Catppuccin::new().mauve;
    let bg = Catppuccin::new().crust;

    parallel(&[
        prolong_start(700, never_complete(fade_to_fg(color_cap, (1500, Interpolation::SineIn))))
            .with_cell_selection(Text),
        never_complete(fade_to_fg(color_border, (1500, Interpolation::SineIn)))
            .with_cell_selection(Not(Text.into())),
    ])
}

pub fn led_kbd_border() -> Effect {
    use tachyonfx::{CellFilter::*, fx::*};

    let colors = Catppuccin::new();
    let color_1 = colors.blue;
    let color_2 = colors.green;
    let color_3 = colors.mauve;

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

        // velocity; 10 colors per second
        let elapsed = started_at.as_ref().unwrap().elapsed().as_millis().max(1).saturating_sub(500);
        let raw_color_idx = (elapsed / 100) as u32;

        let color = |pos: Position| -> Color {
            let idx = if elapsed < 1200 {
                let factor = 1.0 / (elapsed as f32 / 2000.0);
                // let factor = (elapsed as f32 / 1200.0) / 1.0;
                let raw = pos.x / 2 + pos.y * 3 / 2;
                let idx = (raw as f32 * factor) as u32;
                (raw_color_idx + idx) as usize
            } else {
                (raw_color_idx + (pos.x / 2 + pos.y * 3 / 2) as u32) as usize
            };
            colors[idx % colors.len()]
        };

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut(Position::new(x, y)).unwrap();
                if let Some(ch) = cell.symbol().chars().next() {
                    if !is_box_drawing(ch) && ch != ' ' {
                        cell.set_fg(color(Position::new(x, y)));
                    }
                }
            }
        }
    }).with_cell_selection(Outer(Margin::new(1, 1)))

}

fn draw_single_border(key_cap: KeyCap, duration: Duration) -> Effect {
    use tachyonfx::fx::*;
    let border_style = Style::default().fg(Catppuccin::new().base);

    effect_fn_buf((), duration, move |_state, ctx, buf| {
        render_border(key_cap.clone(), border_style, buf)
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