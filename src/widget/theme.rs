use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Layout, Rect};
use ratatui::layout::Constraint::Percentage;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use tachyonfx::HslConvertable;
use crate::styling::Catppuccin;

pub struct ColorDemoWidget;

impl ColorDemoWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for ColorDemoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let colors = Catppuccin::new();

        area.rows().zip(colors.named_colors()).for_each(|(row, (color_name, color))| {
            let areas = Layout::horizontal([Percentage(50), Percentage(50)])
                .split(row);

            let (h, s, l) = color.to_hsl();
            let text = Line::from(vec![
                Span::from(color_name),
                Span::from(format!(" ({:.0} {:.0} {:.0})", h, s, l))
            ]);

            text.clone()
                .style(Style::default().fg(color).bg(colors.crust))
                .alignment(Alignment::Center)
                .render(areas[0], buf);

            text
                .style(Style::default().fg(colors.text).bg(color))
                .alignment(Alignment::Center)
                .render(areas[1], buf);
        });
    }
}