use ratatui::layout::Margin;
use ratatui::style::{Color, Style};
use tachyonfx::{CellFilter, Duration, Effect, Interpolation};
use crate::styling::Catppuccin;

pub fn key_press<C: Into<Color>>(key_press_delay: Duration, color: C) -> Effect {
    // border
    let key_borders = CellFilter::Outer(Margin::new(1, 1));
    let key_pad = CellFilter::Inner(Margin::new(0, 0));

    let c = color.into();
    let bg = Catppuccin::new().crust;

    use tachyonfx::fx::*;
    sequence(&[
        prolong_start(key_press_delay,
            fade_to(c, bg, (50, Interpolation::Linear))),
        fade_from(c, bg, (500, Interpolation::CubicOut)),
    ])
}

fn draw_single_border() -> Effect {
    use tachyonfx::fx::*;
    effect_fn_buf((), 1, |_state, ctx, cells| {
        let a = ctx.area;
        // draw nw corner
        // draw ne corner
        // draw sw corner
        // draw se corner
        // draw horizontal
        // draw vertical
    })

}

mod shader {

}