use crate::styling::Catppuccin;
use ratatui::prelude::Modifier;
use ratatui::style::{Color, Style};
use tachyonfx::Interpolatable;

#[derive(Debug, Clone, Copy)]
pub struct Theme;

pub trait ExabindTheme {
    fn kbd_surface(&self) -> Style;
    fn kbd_cap_border(&self) -> Style;
    fn kbd_cap_text(&self) -> Style;
    fn kbd_cap_outline_category(&self, category_index: usize) -> Style;
    fn kbd_led_colors(&self) -> [Color; 3];

    fn kbd_key_press_color(&self) -> Color;

    fn shortcuts_widget_keystroke(&self) -> Style;
    fn shortcuts_widget_label(&self) -> Style;
    fn shortcuts_base_color(&self, category_index: usize) -> Color;
}

impl ExabindTheme for Theme {
    fn kbd_surface(&self) -> Style {
        Style::default().bg(COLORS.crust)
    }

    fn kbd_cap_border(&self) -> Style {
        Style::default().fg(COLORS.mantle).bg(COLORS.crust)
    }

    fn kbd_cap_text(&self) -> Style {
        Style::default().fg(COLORS.mantle).bg(COLORS.crust)
    }

    fn kbd_cap_outline_category(&self, category_index: usize) -> Style {
        let base_color = Self.shortcuts_base_color(category_index);
        Style::default().fg(COLORS.crust.lerp(&base_color, 0.85))
    }

    fn kbd_led_colors(&self) -> [Color; 3] {
        [COLORS.blue, COLORS.green, COLORS.mauve]
    }

    fn kbd_key_press_color(&self) -> Color {
        COLORS.sapphire
    }

    fn shortcuts_widget_keystroke(&self) -> Style {
        Style::default()
            .fg(COLORS.flamingo)
            .add_modifier(Modifier::BOLD)
    }

    fn shortcuts_widget_label(&self) -> Style {
        Style::default().fg(COLORS.text)
    }

    fn shortcuts_base_color(&self, category_index: usize) -> Color {
        static BASE_COLORS: [Color; 14] = [
            COLORS.rosewater,
            COLORS.flamingo,
            COLORS.pink,
            COLORS.mauve,
            COLORS.red,
            COLORS.maroon,
            COLORS.peach,
            COLORS.yellow,
            COLORS.green,
            COLORS.teal,
            COLORS.sky,
            COLORS.sapphire,
            COLORS.blue,
            COLORS.lavender,
        ];

        BASE_COLORS[category_index % BASE_COLORS.len()]
    }
}

const COLORS: Catppuccin = Catppuccin::new();
