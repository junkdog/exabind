use ratatui::layout::Margin;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use tachyonfx::{CellFilter, Duration, Effect, Interpolation};
use crate::styling::Catppuccin;
use crate::widget::{render_border, KeyCap, KeyCapWidget};

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
            clear_cells(Duration::from_millis(550)),
            draw_single_border(key.clone(), Duration::from_millis(550)),
        ])).with_cell_selection(key_borders),
        // "click" fade out
        sequence(&[
            prolong_start(key_press_delay,
                fade_to(c, bg, (50, Interpolation::Linear))),
            fade_from(c, bg, (500, Interpolation::SineOut)),
        ]),
    ]).with_area(key.area)
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

mod shader {

}