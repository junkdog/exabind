mod keyboard;
mod shortcut_categories;
mod shortcuts;

use crate::app::KeyMapContext;
use crate::styling::{ExabindTheme, Theme};
pub use keyboard::*;
pub use shortcuts::*;

pub fn shortcut_widgets(context: &KeyMapContext) -> Vec<ShortcutsWidget> {
    context
        .unordered_categories()
        .iter()
        .map(|category| shortcut_widget(context, category))
        .collect()
}

fn shortcut_widget(context: &KeyMapContext, category: &str) -> ShortcutsWidget {
    let (category_idx, actions) = context.filtered_actions_by_category(category);
    let base_color = Theme.shortcuts_base_color(category_idx);

    ShortcutsWidget::new(
        category.to_string(),
        Theme.shortcuts_widget_keystroke(),
        Theme.shortcuts_widget_label(),
        base_color,
        actions,
    )
}
