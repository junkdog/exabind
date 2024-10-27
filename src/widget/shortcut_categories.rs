use std::fmt::format;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{List, ListState, StatefulWidget};

pub struct ShortcutCategoriesWidget {
    categories: Vec<(String, usize)>,
}


impl ShortcutCategoriesWidget {
    pub fn new(
        categories: Vec<(String, usize)>,
    ) -> Self {
        Self { categories }
    }
}

impl StatefulWidget for ShortcutCategoriesWidget {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        List::new(self.categories.iter().map(|(name, count)| format!("{} ({})", name, count)))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .render(area, buf, state);
    }
}
