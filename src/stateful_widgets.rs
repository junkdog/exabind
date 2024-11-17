use crate::app::KeyMapContext;
use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crate::ui_state::UiState;
use crate::widget::ShortcutsWidget;
use crate::shortcut_widgets;
use ratatui::layout::{Offset, Position, Rect};
use rectangle_pack::{contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert, RectanglePackOk, TargetBin};
use std::collections::BTreeMap;
use std::sync::mpsc::Sender;

pub(super) struct StatefulWidgets {
    pub shortcuts: Vec<ShortcutsWidget>,
    sender: Sender<ExabindEvent>,
}

const KBD_WIDGET_ID: usize = !0;

impl StatefulWidgets {
    pub fn new(
        context: &KeyMapContext,
        sender: Sender<ExabindEvent>,
    ) -> Self {
        Self {
            shortcuts: shortcut_widgets(context),
            sender,
        }
    }

    pub fn category_widgets(&self) -> &[ShortcutsWidget] {
        &self.shortcuts
    }

    /// creates and packs shortcut widgets for all categories
    pub fn update_shortcut_category(
        &mut self,
        keymap_context: &KeyMapContext,
        ui_state: &mut UiState,
    ) {
        let screen = Rect::new(0, 0, ui_state.screen.width as _, ui_state.screen.height as _);
        let kbd = ui_state.kbd_size();
        let mut shortcuts: Vec<(usize, ShortcutsWidget)> = shortcut_widgets(keymap_context).into_iter()
            .enumerate()
            .map(|(idx, w)| (idx, w))
            .collect();

        let mut rects_to_place: GroupedRectsToPlace<usize, ()> = GroupedRectsToPlace::new();

        // enqueue the keyboard widget first
        rects_to_place.push_rect(KBD_WIDGET_ID, None, RectToInsert::new(kbd.width as _, kbd.height as _, 1));
        // and then the shortcuts
        for (idx, w) in &mut shortcuts {
            let a = w.area();
            rects_to_place.push_rect(
                idx.clone(),
                None,
                RectToInsert::new((a.width + 1) as _, a.height as _, 1) // +1 for padding
            )
        }

        // pack the rects
        self.pack_rects(rects_to_place, screen)
            .iter()
            .for_each(|(id, pos)| {
                match *id {
                    KBD_WIDGET_ID => ui_state.set_kbd_offset(Offset { x: pos.x as _, y: pos.y as _ }),
                    id         => {
                        let shortcut =  shortcuts.iter_mut().find(|(idx, w)| *idx == id).unwrap();
                        shortcut.1.position = *pos;
                    }
                }
            });

        let layout_order = |area: Rect| -> u32 {
            area.x as u32 + (area.y as u32 * screen.width as u32)
        };

        shortcuts.sort_by(|a, b| layout_order(a.1.area()).cmp(&layout_order(b.1.area())));
        let sorted_indices: Vec<usize> = shortcuts.iter().map(|(idx, _)| *idx).collect();
        self.sender.dispatch(ExabindEvent::CategoryWidgetNavigationOrder(sorted_indices));

        self.shortcuts = shortcuts.into_iter().map(|(_, w)| w).collect();
    }

    pub fn selected_category_area(&self, keymap_context: &KeyMapContext) -> Rect {
        let idx = keymap_context.category_idx();
        self.shortcuts[idx].area()
    }

    pub fn selected_category_widget(&self, keymap_context: &KeyMapContext) -> &ShortcutsWidget {
        let idx = keymap_context.category_idx();
        &self.shortcuts[idx]
    }

    fn pack_rects(
        &self,
        rects_to_pack: GroupedRectsToPlace<usize>,
        screen: Rect,
    ) -> Vec<(usize, Position)> {
        let packed_locations = |packed: RectanglePackOk<usize, _>| {
            packed.packed_locations()
                .iter()
                .map(|(id, (_, loc))| (id, Position::new(loc.x() as _, loc.y() as _)))
                .map(|(id, pos)| (*id, pos))
                .collect::<Vec<_>>()
        };

        let mut pack_rects_fn = |screen: Rect| {
            let mut target_bins = BTreeMap::new();
            target_bins.insert("main", TargetBin::new(screen.width as _, screen.height as _, 1));

            let res = pack_rects(
                &rects_to_pack,
                &mut target_bins,
                &volume_heuristic,
                &contains_smallest_box
            ).map(packed_locations);

            match res {
                Ok(locations) => Some((target_bins, locations)),
                Err(_) => None
            }
        };

        // loop pack_rects_fn until it returns Some, to pack
        // the rects into the shortest possible screen height
        let mut screen = screen;
        screen.width = screen.width.max(96);
        screen.height = 20;
        let mut packed_locations = pack_rects_fn(screen);
        while packed_locations.is_none() {
            screen.height += 2;
            packed_locations = pack_rects_fn(screen);
        }

        let (_, packed) = packed_locations.unwrap();
        packed.into_iter()
            .map(|(id, pos)| (id, Position::new(pos.x + screen.x, pos.y + screen.y)))
            .collect()
    }
}
