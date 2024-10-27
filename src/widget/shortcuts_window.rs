use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Margin, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Clear, List, ListItem, Row, StatefulWidgetRef, Table, TableState, Widget, WidgetRef};
use tachyonfx::HslConvertable;
use crate::shortcut::{Action, Shortcut};

pub struct ShortcutsWindow {
    title: &'static str,
    shortcuts: Vec<Action>,
    size: Size,
    row_style: Style,
    selected_row_style: Style,
    keystroke_style: Style,
    action_name_style: Style,
    max_shortcut_title_width: u16,
    max_shortcut_keystroke_width: u16,
}

impl ShortcutsWindow {
    pub fn new(
        title: &'static str,
        keystroke_style: Style,
        action_name_style: Style,
        base_color: Color,
        shortcuts: Vec<Action>
    ) -> Self {
        let width_name = shortcuts.iter()
            .map(Action::name)
            .map(str::len)
            .max()
            .unwrap_or(0);

        let width_shortcut = shortcuts.iter()
            .flat_map(Action::shortcuts)
            .map(|s| s.to_string().char_indices().count())
            .max()
            .unwrap_or(0);

        let width = width_name + width_shortcut + 3;
        let height = shortcuts.iter()
            .flat_map(Action::shortcuts)
            .count() + 2; // 2 from margin

        let (h, s, l) = base_color.to_hsl();
        let row_style = Style::default().bg(base_color);

        let selected_row_style = Style::default()
            .bg(Color::from_hsl(h, s, 0.0_f32.max(l - 15.0)))
            .add_modifier(Modifier::BOLD);

        Self {
            title,
            shortcuts,
            size: Size::new(width as _, height as _),
            row_style,
            selected_row_style,
            keystroke_style,
            action_name_style,
            max_shortcut_title_width: width_name as _,
            max_shortcut_keystroke_width: width_shortcut as _,
        }
    }

    fn area(&self) -> Rect {
        // 3 from margin + delimiter between name and shortcut
        let width = self.max_shortcut_title_width + self.max_shortcut_keystroke_width + 3;

        let height = self.shortcuts.iter()
            .flat_map(Action::shortcuts)
            .count() + 2; // 2 from margin

        Rect::new(0, 0, width, height as _)
    }

    fn shortcut_as_table_row<'a>(&self, name: &'a str, shortcut: &Shortcut) -> Row<'a> {
        let name = Text::from(name)
            .style(self.action_name_style);

        let shortcuts = Text::from(shortcut.to_string())
            .style(self.keystroke_style);

        Row::new([shortcuts, name])
    }

    fn rows(&self) -> impl Iterator<Item = Row> + '_ {
        self.shortcuts.iter()
            .flat_map(move |action| {
                let name = action.name();
                action.shortcuts()
                    .iter()
                    .map(move |shortcut| self.shortcut_as_table_row(name, shortcut))
            })
    }
}



impl StatefulWidgetRef for ShortcutsWindow {
    type State = TableState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let translated_area: Rect = (area.as_position(), self.size).into();
        let translated_area = translated_area.intersection(area);

        let constraints = [
            Constraint::Length(self.max_shortcut_keystroke_width),
            Constraint::Length(self.max_shortcut_title_width),
        ];

        Clear.render(translated_area, buf);

        Block::new()
            .style(self.row_style)
            .render(translated_area, buf);

        let content_area = translated_area.inner(Margin::new(1, 1));

        Table::new(self.rows(), constraints)
            .column_spacing(1)
            .row_highlight_style(self.selected_row_style)
            .render(content_area, buf);
    }
}
