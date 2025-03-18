use crate::app::KeyMapContext;
use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crate::ui_state::UiState;
use crate::widget::{shortcut_widgets, ShortcutsWidget};
use packr2::{pack, PackerConfig, RectInput, Size, SkylinePacker};
use ratatui::layout::{Offset, Position, Rect};
use std::sync::mpsc::Sender;

pub(super) struct StatefulWidgets {
    pub shortcuts: Vec<ShortcutsWidget>,
    sender: Sender<ExabindEvent>,
}

const KBD_WIDGET_ID: usize = !0;

impl StatefulWidgets {
    pub fn new(context: &KeyMapContext, sender: Sender<ExabindEvent>) -> Self {
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
        let screen = Rect::new(
            0,
            0,
            ui_state.screen.width as _,
            ui_state.screen.height as _,
        );
        let kbd = ui_state.kbd_size();
        let mut shortcuts: Vec<(usize, ShortcutsWidget)> = shortcut_widgets(keymap_context)
            .into_iter()
            .enumerate()
            .collect();

        let mut rects_to_place: Vec<RectInput<usize>> = vec![];

        // enqueue the keyboard widget first
        rects_to_place.push(RectInput {
            key: KBD_WIDGET_ID,
            size: Size::new(kbd.width as _, kbd.height as _),
        });
        // and then the shortcuts
        for (idx, w) in &mut shortcuts {
            let a = w.area();
            rects_to_place.push(RectInput {
                key: *idx,
                size: Size::new((a.width + 1) as _, a.height as _), // +1 for padding
            })
        }

        // pack the rects
        self.pack_rects(&mut rects_to_place, screen)
            .iter()
            .for_each(|(id, pos)| match *id {
                KBD_WIDGET_ID => ui_state.set_kbd_offset(Offset {
                    x: pos.x as _,
                    y: pos.y as _,
                }),
                id => {
                    let shortcut = shortcuts.iter_mut().find(|(idx, _)| *idx == id).unwrap();
                    shortcut.1.position = *pos;
                }
            });

        let layout_order =
            |area: Rect| -> u32 { area.x as u32 + (area.y as u32 * screen.width as u32) };

        shortcuts.sort_by(|a, b| layout_order(a.1.area()).cmp(&layout_order(b.1.area())));
        let sorted_indices: Vec<usize> = shortcuts.iter().map(|(idx, _)| *idx).collect();
        self.sender
            .dispatch(ExabindEvent::CategoryWidgetNavigationOrder(sorted_indices));

        self.shortcuts = shortcuts.into_iter().map(|(_, w)| w).collect();
    }

    // pub fn selected_category_area(&self, keymap_context: &KeyMapContext) -> Rect {
    //     let idx = keymap_context.category_idx().expect("no category selected");
    //     self.shortcuts[idx].area()
    // }

    pub fn selected_category_widget(&self, keymap_context: &KeyMapContext) -> &ShortcutsWidget {
        let idx = keymap_context.category_idx().unwrap_or(0); //.expect("no category selected");
        &self.shortcuts[idx]
    }

    fn pack_rects(
        &self,
        rects_to_pack: &mut Vec<RectInput<usize>>,
        screen: Rect,
    ) -> Vec<(usize, Position)> {
        // loop pack_rects_fn until it returns Some, to pack
        // the rects into the shortest possible screen height
        let mut screen = screen;
        screen.width = screen.width.max(96);
        screen.height = 20;
        loop {
            let packed = pack(
                rects_to_pack,
                SkylinePacker::new(PackerConfig {
                    max_width: screen.width as _,
                    max_height: screen.height as _,
                    allow_flipping: false,
                }),
            );
            if packed.len() == rects_to_pack.len() {
                break packed;
            }
            screen.height += 2;
        }
        .into_iter()
        .map(|r| {
            (
                r.key,
                Position::new(r.rect.x as u16 + screen.x, r.rect.y as u16 + screen.y),
            )
        })
        .collect()
    }
}
