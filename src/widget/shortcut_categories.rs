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
        let categories: Vec<_> = self.categories.iter()
            .map(|(name, count)| format!("{} ({})", name, count))
            .collect();

        let max_width = categories.iter()
            .map(|s| s.char_indices().count())
            .max()
            .unwrap_or(0);

        let mut area = area;
        area.width = 1 + max_width as u16;

        List::new(categories)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .render(area, buf, state);
    }
}
