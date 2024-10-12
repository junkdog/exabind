use ratatui::layout::Margin;
use ratatui::style::{Color, Style};
use tachyonfx::{CellFilter, Duration, Effect, Interpolation};

pub fn key_press<C: Into<Color>>(key_press_delay: Duration, color: C) -> Effect {
    // border
    let key_borders = CellFilter::Outer(Margin::new(1, 1));
    let key_pad = CellFilter::Inner(Margin::new(0, 0));

    let c = color.into();
    let bg = Color::DarkGray;

    use tachyonfx::fx::*;
    repeating(sequence(&[
        // prolong_start(key_press_delay,
            // fade_to(c, bg, (50, Interpolation::ExpoIn))),
        fade_from(c, bg, (500, Interpolation::CircOut)),
    ]).with_cell_selection(key_pad))
}

// fn draw_single_border<C: Into<Color>>() -> Effect {
//     use tachyonfx::fx::*;
//     let key_borders = CellFilter::Outer(Margin::new(1, 1));
//
//
// }

mod shader {

}