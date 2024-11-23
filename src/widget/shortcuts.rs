use crate::app::BoundShortcut;
use crate::styling::{Catppuccin, CATPPUCCIN};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Position, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border::Set;
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Clear, Row, StatefulWidgetRef, Table, TableState, Widget};
use tachyonfx::{HslConvertable, Interpolatable};

pub struct ShortcutsWidget {
    title: String,
    category: String,
    shortcuts: Vec<BoundShortcut>,
    pub selected_shortcut: Option<usize>,
    size: Size,
    pub position: Position,
    max_shortcut_title_width: u16,
    max_shortcut_keystroke_width: u16,
    border_color: Color,
    bg_color: Color,
    table: Table<'static>,
}

pub struct ShortcutsWidgetState {
    pub table_state: TableState,
}

impl ShortcutsWidget {
    pub fn new(
        title: String,
        keystroke_style: Style,
        action_name_style: Style,
        base_color: Color,
        shortcuts: Vec<BoundShortcut>,
    ) -> Self {
        let width_name = shortcuts.iter()
            .map(BoundShortcut::name)
            .map(str::len)
            .max()
            .unwrap_or(0);

        let width_shortcut = shortcuts.iter()
            .map(BoundShortcut::shortcut)
            .map(|s| s.to_string().char_indices().count())
            .max()
            .unwrap_or(0);

        let width = width_name + width_shortcut + 3;
        let height = shortcuts.iter()
            .map(BoundShortcut::shortcut)
            .count();

        let (h, s, l) = base_color.to_hsl();

        let selected_row_style = Style::default()
            .bg(Color::from_hsl(h, s, 0.0_f32.max(l - 15.0)))
            .add_modifier(Modifier::BOLD);

        let constraints = [
            Constraint::Length(width_shortcut as _),
            Constraint::Length(width_name as _),
        ];

        let bg_color = CATPPUCCIN.crust.lerp(&base_color, 0.15);
        let border_color = CATPPUCCIN.crust.lerp(&base_color, 0.85);
        let border_style = Style::default().fg(border_color);

        let mut title2 = title.clone();
        title2.insert(0, ' ');
        title2.push_str(" ◢");

        let table = Table::new(rows(&shortcuts, action_name_style, keystroke_style), constraints)
            .block(Block::bordered()
                .border_set(SHORTCUT_SET_2)
                .title(Span::styled(title2.clone(), border_style
                    .bg(bg_color)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)))
                .style(border_style))
            .style(Style::default().bg(bg_color))
            .column_spacing(1)
            .row_highlight_style(selected_row_style);

        Self {
            title: title2,
            category: title,
            shortcuts,
            table,
            position: Position::default(),
            size: Size::new((width + 2) as _, (height + 2) as _), // border padding
            selected_shortcut: None,
            max_shortcut_title_width: width_name as _,
            max_shortcut_keystroke_width: width_shortcut as _,
            bg_color,
            border_color,
        }
    }

    pub fn bg_color(&self) -> Color {
        self.bg_color
    }

    pub fn border_color(&self) -> Color {
        self.border_color
    }

    pub fn initial_state(&self) -> ShortcutsWidgetState {
        let table_state = TableState::default();

        ShortcutsWidgetState {
            table_state,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn clear_selected_shortcut(&mut self) {
        self.selected_shortcut = None;
    }

    pub fn select_next_shortcut(&mut self) {
        if let Some(selected) = self.selected_shortcut {
            if selected < self.shortcuts.len() - 1 {
                self.selected_shortcut = Some(selected + 1);
            }
        } else {
            self.selected_shortcut = Some(0);
        }
    }

    pub fn select_previous_shortcut(&mut self) {
        if let Some(selected) = self.selected_shortcut {
            if selected > 0 {
                self.selected_shortcut = Some(selected - 1);
            }
        } else {
            self.selected_shortcut = Some(0);
        }
    }

    pub fn area(&self) -> Rect {
        // 3 from margin + delimiter between name and shortcut
        let width = self.max_shortcut_title_width + self.max_shortcut_keystroke_width + 3;

        let height = self.shortcuts.iter()
            .map(BoundShortcut::shortcut)
            .count() + 2; // 2 from margin

        Rect::new(self.position.x, self.position.y, width, height as _)
    }
}

fn shortcut_as_table_row<'a>(
    bound_shortcut: &BoundShortcut,
    action_name_style: Style,
    keystroke_style: Style,
) -> Row<'static> {
    let shortcut = bound_shortcut.shortcut();

    if bound_shortcut.enabled_in_ui() {
        let name = Text::from(bound_shortcut.name().to_string())
            .style(action_name_style);

        let shortcuts = Text::from(shortcut.to_string())
            .style(keystroke_style);

        Row::new([shortcuts, name])
    } else {
        let name = Text::from(bound_shortcut.name().to_string())
            .style(Style::default().fg(Catppuccin::new().surface2));

        let shortcuts = Text::from(shortcut.to_string())
            .style(Style::default().fg(Catppuccin::new().surface2));

        Row::new([shortcuts, name])
    }
}

fn rows(
    shortcuts: &[BoundShortcut],
    action_name_style: Style,
    keystroke_style: Style,
) -> impl Iterator<Item = Row<'static>> + '_ {
    shortcuts.iter()
        .map(move |action| {
            shortcut_as_table_row(
                action,
                action_name_style,
                keystroke_style,
            )
        })
}


impl StatefulWidgetRef for ShortcutsWidget {
    type State = ShortcutsWidgetState;

    fn render_ref(&self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let translated_area: Rect = self.area();
        let translated_area = translated_area.intersection(*buf.area());

        Clear.render(translated_area, buf);

        StatefulWidgetRef::render_ref(&self.table, translated_area, buf, &mut state.table_state);

        let border_south = translated_area.rows().last().unwrap_or_default();
        for xy in border_south.positions() {
            if let Some(c) = buf.cell_mut(xy) {
                let style = c.style();
                c.set_style(style.bg(CATPPUCCIN.crust));
            };
        }

        let top_left = translated_area.as_position();
        if let Some(c) = buf.cell_mut(top_left) {
            let style = c.style();
            c.set_style(style.bg(CATPPUCCIN.crust));
        }
    }
}

const SHORTCUT_SET: Set = Set {
    top_left:          "◢",
    top_right:         "▜",
    bottom_left:       "▙",
    bottom_right:      "╱",
    vertical_left:     "▏",
    vertical_right:    "▕",
    horizontal_top:    "▔",
    horizontal_bottom: "▁",
};

const SHORTCUT_SET_2: Set = Set {
    top_left:          "◢",
    top_right:         "▜",
    bottom_left:       "▔",
    bottom_right:      "▔",
    vertical_left:     "▏",
    vertical_right:    "▕",
    horizontal_top:    "▔",
    horizontal_bottom: "▔",
};