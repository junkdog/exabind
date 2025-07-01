use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crate::fx::effect::{outline_selected_category_key_caps, starting_up, UniqueEffectId};
use crate::fx::effect;
use crate::input::InputProcessor;
use crate::{KeyMap, Shortcut};
use crate::stateful_widgets::StatefulWidgets;
use crate::ui_state::UiState;
use crate::widget::{AnsiKeyboardTklLayout, KeyCap, KeyboardLayout};
use crate::key_event::{KeyCode, ModifierKeyCode};
use crate::key_event::ModifierKeyCode::{LeftAlt, LeftControl, LeftMeta, LeftShift};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect, Size};
use std::sync::mpsc::Sender;
#[cfg(feature = "web-time")]
use web_time::Instant;
#[cfg(not(feature = "web-time"))]
use std::time::Instant;
use tachyonfx::fx::consume_tick;
use tachyonfx::{fx, CellFilter, Duration, Effect, EffectManager, Interpolation};

pub struct ExabindApp {
    running: bool,
    keymap_context: KeyMapContext,
    sender: Sender<ExabindEvent>,
    last_tick: Instant,
    input_processor: InputProcessor,
    effects: EffectManager<UniqueEffectId>,
    stateful_widgets: StatefulWidgets,
}

pub struct KeyMapContext {
    pub keymap: KeyMap,
    categories: Vec<(String, usize)>,
    ordered_categories: Vec<usize>,
    current_category: Option<usize>,
    pub current_action: Option<usize>,
    pub filter_key_control: bool,
    pub filter_key_alt: bool,
    pub filter_key_shift: bool,
    pub filter_key_meta: bool,
}

impl KeyMapContext {
    pub fn apply_event(&mut self, event: &ExabindEvent) {
        if let ExabindEvent::CategoryWidgetNavigationOrder(order) = event {
            self.ordered_categories = order.clone();
        }
    }

    pub fn unordered_categories(&self) -> Vec<&str> {
        self.categories.iter().map(|(cat, _)| cat.as_str()).collect()
    }

    pub fn deselect_category(&mut self) {
        self.current_category = None;
        self.current_action = None;
    }

    pub fn next_category(&mut self) {
        let last_index = self.categories.len() - 1;
        match self.current_category {
            None                           => self.current_category = Some(0),
            Some(idx) if last_index == idx => self.current_category = Some(0),
            Some(idx)                      => self.current_category = Some(idx + 1)
        }

        self.current_action = None;
    }

    pub fn previous_category(&mut self) {
        match self.current_category {
            None      => self.current_category = Some(0),
            Some(0)   => self.current_category = Some(self.categories.len() - 1),
            Some(idx) => self.current_category = Some(idx - 1),
        }
        self.current_action = None;
    }

    pub fn category(&self) -> Option<&str> {
        let idx = self.current_category?;
        let category_idx = self.ordered_categories[idx];
        Some(self.categories[category_idx].0.as_str())
    }

    pub fn category_idx(&self) -> Option<usize> {
        self.current_category
    }

    pub fn sorted_category_idx(&self) -> Option<usize> {
        Some(self.ordered_categories[self.current_category?])
    }

    pub fn current_modifier_keys(&self) -> Vec<KeyCap> {
       let layout = AnsiKeyboardTklLayout
           .layout();


        // fixme: don't do this
        let find_mod_key = |modifier_key_code| {
            layout.iter()
                .find(|key| key.key_code == KeyCode::Modifier(modifier_key_code))
                .expect("modifier key lookup should always succeed")
                .clone()
        };

        let mut modifiers = Vec::new();
        if self.filter_key_control { modifiers.push(find_mod_key(LeftControl)); }
        if self.filter_key_meta    { modifiers.push(find_mod_key(LeftMeta)); }
        if self.filter_key_alt     { modifiers.push(find_mod_key(LeftAlt)); }
        if self.filter_key_shift   { modifiers.push(find_mod_key(LeftShift)); }

        modifiers
    }

    pub fn toggle_filter_key(&mut self, key_code: ModifierKeyCode) {
        use ModifierKeyCode::*;
        match key_code {
            LeftShift | RightShift => self.filter_key_shift = !self.filter_key_shift,
            LeftControl | RightControl => self.filter_key_control = !self.filter_key_control,
            LeftAlt | RightAlt => self.filter_key_alt = !self.filter_key_alt,
            LeftMeta | RightMeta => self.filter_key_meta = !self.filter_key_meta,
            LeftSuper | RightSuper => self.filter_key_meta = !self.filter_key_meta,
            // LeftSuper | RightSuper => (),
            // LeftHyper | RightHyper => (),
            // LeftMeta | RightMeta =>(),
            _ => panic!("Invalid modifier key code: {:?}", key_code),
        }
    }

    pub fn filtered_actions(&self) -> Vec<BoundShortcut> {
        match self.category() {
            Some(category) => self.filtered_actions_by_category(category).1,
            None           => Vec::new(),
        }
    }

    pub fn filtered_actions_by_category(&self, category: &str) -> (usize, Vec<BoundShortcut>) {
        let keymap = &self.keymap;

        let uses_any_modifier_keys = || -> bool {
            self.filter_key_control
                || self.filter_key_shift
                || self.filter_key_alt
                || self.filter_key_meta
        };

        let uses_active_modifier_keys = |shortcut: &Shortcut| -> bool {
            !uses_any_modifier_keys() || (
                self.filter_key_control  == shortcut.uses_modifier(LeftControl)
                    && self.filter_key_shift == shortcut.uses_modifier(LeftShift)
                    && self.filter_key_alt   == shortcut.uses_modifier(LeftAlt)
                    && self.filter_key_meta  == shortcut.uses_modifier(LeftMeta)
            )
        };

        let index_of_category = keymap.categories().iter().position(|(cat, _)| cat == category).unwrap();
        (index_of_category, keymap.actions_by_category(category)
            .iter()
            .flat_map(|action| {
                action.shortcuts()
                    .iter()
                    .map(|shortcut| BoundShortcut {
                        label: action.name().to_string(),
                        enabled_in_ui: uses_active_modifier_keys(shortcut),
                        shortcut: shortcut.clone(),
                    })
            })
            .collect())
    }
}

impl ExabindApp {
    pub fn new(
        ui_state: &mut UiState,
        sender: Sender<ExabindEvent>,
        keymap: KeyMap,
    ) -> Self {
        let categories = keymap.categories();
        let ordered_categories = if categories.is_empty() {
            Vec::new()
        } else {
            (0..categories.len()).collect()
        };
        let keymap_context = KeyMapContext {
            categories,
            ordered_categories,
            current_category: None,
            current_action: None,
            filter_key_control: false,
            filter_key_alt: false,
            filter_key_shift: false,
            filter_key_meta: false,
            keymap,
        };
        let mut widgets = StatefulWidgets::new(&keymap_context, sender.clone());
        widgets.update_shortcut_category(&keymap_context, ui_state);
        Self {
            running: true,
            input_processor: InputProcessor::new(sender.clone()),
            sender,
            keymap_context,
            last_tick: Instant::now(),
            effects: EffectManager::default(),
            stateful_widgets: widgets,
        }
    }

    pub fn keymap_context(&self) -> &KeyMapContext {
        &self.keymap_context
    }

    pub fn register_effect(&mut self, effect: Effect) {
        self.effects.add_effect(effect);
    }

    pub fn stage_mut(&mut self) -> &mut EffectManager<UniqueEffectId> {
        &mut self.effects
    }

    pub fn sender(&self) -> Sender<ExabindEvent> {
        self.sender.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stateful_widgets(&self) -> &StatefulWidgets {
        &self.stateful_widgets
    }

    pub fn update_time(&mut self) -> Duration {
        let now = Instant::now();
        let last_frame_duration: Duration = now.duration_since(self.last_tick).into();
        self.last_tick = now;
        last_frame_duration
    }

    pub fn process_effects(&mut self, last_frame_duration: Duration, buf: &mut Buffer, area: Rect) {
        self.effects.process_effects(last_frame_duration, buf, area);
    }

    pub fn apply_event(&mut self, event: ExabindEvent, ui_state: &mut UiState) {
        use ExabindEvent::*;

        self.keymap_context.apply_event(&event);

        match event {
            Tick                      => (),
            Shutdown                  => self.running = false,
            KeyPress(_)               => self.input_processor.apply(&event),
            StartupAnimation          => ui_state.register_kbd_effect(starting_up()),
            AutoSelectCategory => {
                if self.keymap_context.category().is_none() {
                    self.dispatch(NextCategory)
                }
            }
            Resize(w, h) => {
                ui_state.screen = Size::new(w, h);
                self.update_selected_category(ui_state);
            }
            DeselectCategory          => {
                self.keymap_context.deselect_category();
                self.stage_mut()
                    .add_unique_effect(UniqueEffectId::SelectedCategory, consume_tick());
                ui_state.kbd_effects_mut()
                    .add_unique_effect(UniqueEffectId::KeyCapOutline, consume_tick());
            },
            NextCategory              => {
                self.keymap_context.next_category();
                self.update_selected_category(ui_state);
            },
            PreviousCategory          => {
                self.keymap_context.previous_category();
                self.update_selected_category(ui_state);
            },
            ToggleFilterKey(key_code) => {
                self.keymap_context.toggle_filter_key(key_code);
                self.update_selected_category(ui_state);

                ui_state.update_active_modifiers(self.keymap_context.current_modifier_keys());
            },
            CategoryWidgetNavigationOrder(_) => {
                let size = ui_state.kbd_size();
                let stage = ui_state.kbd_effects_mut();
                if self.keymap_context.current_category.is_some() {
                    let fx = outline_selected_category_key_caps(stage, self.keymap_context(), size);
                    stage.add_effect(fx);
                }
            },
            SelectedCategoryFxSandbox => {
                let widget = self.stateful_widgets.selected_category_widget(&self.keymap_context);
                let area = widget.area();
                let fx = effect::open_category(widget.bg_color(), area);
                self.register_effect(fx);
            }
        }
    }

    fn update_selected_category(&mut self, ui_state: &mut UiState) {
        if self.keymap_context.current_category.is_none() {
            return;
        }

        self.stateful_widgets.update_shortcut_category(&self.keymap_context, ui_state);

        let widget = self.stateful_widgets
            .selected_category_widget(&self.keymap_context);

        let area = widget.area();
        let color = widget.border_color();
        let fx = fx::parallel(&[
            effect::selected_category(color, area),
            fx::fade_from_fg(color, (200, Interpolation::BounceInOut))
                .with_area(area)
                .with_filter(CellFilter::Outer(Margin::new(1, 1))),
        ]) ;

        self.stage_mut().add_unique_effect(UniqueEffectId::SelectedCategory, fx);
    }
}


#[derive(Debug, Clone)]
pub struct BoundShortcut {
    label: String,
    enabled_in_ui: bool,
    shortcut: Shortcut,
}

impl BoundShortcut {
    pub fn name(&self) -> &str {
        self.label.as_str()
    }

    pub fn enabled_in_ui(&self) -> bool {
        self.enabled_in_ui
    }

    pub fn shortcut(&self) -> &Shortcut {
        &self.shortcut
    }
}

impl Dispatcher<ExabindEvent> for ExabindApp {
    fn dispatch(&self, event: ExabindEvent) {
        self.sender().dispatch(event)
    }
}