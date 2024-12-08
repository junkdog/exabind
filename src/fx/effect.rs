use crate::app::KeyMapContext;
use crate::color_cycle::{ColorCycle, IndexResolver, PingPongColorCycle, RepeatingColorCycle, RepeatingCycle};
use crate::dispatcher::Dispatcher;
use crate::exabind_event::ExabindEvent;
use crate::fx::key_cap_outline::KeyCapOutline;
use crate::fx::EffectStage;
use crate::styling::{Catppuccin, ExabindTheme, Theme, CATPPUCCIN};
use crate::widget::{draw_key_border, render_border_with, AnsiKeyboardTklLayout, KeyCap, KeyboardLayout, ShortcutsWidget};
use crossterm::event::KeyCode;
use ratatui::buffer::Cell;
use ratatui::layout::{Margin, Position, Rect, Size};
use ratatui::prelude::Buffer;
use ratatui::style::{Color, Style};
use std::fmt::Debug;
use std::sync::mpsc::Sender;
use std::time::Instant;
use tachyonfx::fx::{effect_fn_buf, parallel, prolong_start, sequence, sleep, sweep_in};
use tachyonfx::Motion::UpToDown;
use tachyonfx::{fx, CellFilter, Duration, Effect, EffectTimer, HslConvertable, Interpolation, IntoEffect, RangeSampler, SimpleRng};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UniqueEffectId {
    #[default]
    SelectedCategory,
    KeyCapOutline,
}

/// Creates an animated border effect for the selected category using color cycling.
///
/// # Arguments
/// * `base_color` - The primary color to base the cycling effect on
/// * `area` - The rectangular area where the effect should be rendered
///
/// # Returns
/// An Effect that animates a border around the specified area using cycled colors
pub fn selected_category(
    base_color: Color,
    area: Rect,
) -> Effect {
    let color_cycle = select_category_color_cycle(base_color, 1);

    let effect = fx::effect_fn_buf(Instant::now(), u32::MAX, move |started_at, ctx, buf| {
        let elapsed = started_at.elapsed().as_secs_f32();

        // speed n cells/s
        let idx = (elapsed * 30.0) as usize;

        let area = ctx.area;

        let mut update_cell = |(x, y): (u16, u16), idx: usize| {
            if let Some(cell) = buf.cell_mut((x, y)) { cell.set_fg(*color_cycle.color_at(idx)); }
        };

        (area.x..area.right()).enumerate().for_each(|(i, x)| {
            update_cell((x, area.y), idx + i);
        });

        let cell_idx_offset = area.width as usize;
        (area.y + 1..area.bottom() - 1).enumerate().for_each(|(i, y)| {
            update_cell((area.right() - 1, y), idx + i + cell_idx_offset);
        });

        let cell_idx_offset = cell_idx_offset + area.height.saturating_sub(2) as usize;
        (area.x..area.right()).rev().enumerate().for_each(|(i, x)| {
            update_cell((x, area.bottom() - 1), idx + i + cell_idx_offset);
        });

        let cell_idx_offset = cell_idx_offset + area.width as usize;
        (area.y + 1..area.bottom()).rev().enumerate().for_each(|(i, y)| {
            update_cell((area.x, y), idx + i + cell_idx_offset);
        });
    });

    effect.with_area(area)
}

/// Animates the opening of all category widgets with staggered timing.
///
/// # Arguments
/// * `sender` - Channel for dispatching [ExabindEvent]s
/// * `widgets` - Slice of [ShortcutsWidget]s to animate
///
/// # Returns
/// An Effect that:
/// 1. Opens all categories with randomized delays
/// 2. Waits for a short period
/// 3. Triggers category selection
pub fn open_all_categories(
    sender: Sender<ExabindEvent>,
    widgets: &[ShortcutsWidget]
) -> Effect {
    let mut rng = SimpleRng::default();

    let max_open_category_delay = 150 * widgets.len() as u32;
    let open_categories_fx = widgets.iter().map(|w| {
        let delay = Duration::from_millis(rng.gen_range(0..max_open_category_delay));
        prolong_start(delay, open_category(w.bg_color(), w.area()))
    }).collect::<Vec<_>>();

    sequence(&[
        prolong_start(300, parallel(&open_categories_fx)),
        sleep(500),
        dispatch_event(sender, ExabindEvent::AutoSelectCategory),
    ])
}

/// Creates an opening animation effect for a single category widget.
///
/// # Arguments
/// * `bg_color` - Background color for the category
/// * `area` - Rectangular area of the category widget
///
/// # Returns
/// A parallel Effect combining:
/// - Background slide-in effect
/// - Content sweep-in animation
/// - Border coalescing effect
pub fn open_category(
    bg_color: Color,
    area: Rect,
) -> Effect {
    use tachyonfx::{fx::*, Interpolation::*};

    let h = area.height as u32;
    let timer: EffectTimer = (200 + h * 10, Linear).into();
    let timer_c: EffectTimer = (200 + h * 10, ExpoOut).into();

    let border_cells = CellFilter::Outer(Margin::new(1, 1));
    let content_cells = CellFilter::Inner(Margin::new(1, 1));

    parallel(&[
        prolong_start(timer, sweep_in(UpToDown, area.height, 0, bg_color, timer))
            .with_cell_selection(content_cells.clone()),
        prolong_start(timer, coalesce(timer_c))
            .with_cell_selection(border_cells),
        // plays out first, but must come last to not be overridden by the above effects
        slide_in(UpToDown, area.height * 2, 0, CATPPUCCIN.crust, timer),
    ]).with_area(area)
}

/// Creates a key press animation effect.
///
/// # Arguments
/// * `key_press_delay` - Delay before the key press animation starts
/// * `key` - The KeyCap representing the pressed key
/// * `color` - Color for the key press effect
///
/// # Returns
/// An Effect that animates both the key border and key symbol
pub fn key_press<C: Into<Color>>(
    key_press_delay: Duration,
    key: KeyCap,
    color: C
) -> Effect {
    use tachyonfx::fx::*;

    // border
    let key_borders = CellFilter::Outer(Margin::new(1, 1));

    let c = color.into();
    let bg = Catppuccin::new().crust;

    parallel(&[
        // redraw singular border around key
        delay(key_press_delay, parallel(&[
            clear_cells(Duration::from_millis(750)),
            draw_single_border(key.clone(), Duration::from_millis(750)),
        ])).with_cell_selection(key_borders),
        // "click" fade; faded out during key_press_delay
        sequence(&[
            prolong_start(key_press_delay,
                fade_to(c, bg, (50, Interpolation::Linear))),
            fade_from(c, bg, (700, Interpolation::SineOut)),
        ]),
    ]).with_area(key.area)
}

/// Creates the initial startup animation sequence.
///
/// Types out "exabind" with randomized delays between characters,
/// followed by an Enter key press and persistent keyboard LED effects.
///
/// # Returns
/// A never-ending Effect combining the startup sequence and LED animations.
pub fn starting_up() -> Effect {
    let kbd = AnsiKeyboardTklLayout;
    let esc_area = kbd.key_area(KeyCode::Enter);

    let mut effects = vec![];

    let mut rng = SimpleRng::default();
    let initial_delay = Duration::from_millis(300);
    let mut accrued_delay = initial_delay.as_millis();

    "exabind".char_indices().for_each(|(_, c)| {
        let delta: u32 = rng.gen_range(100..200);
        accrued_delay += delta;

        let e = key_press(Duration::from_millis(accrued_delay), kbd.key_cap(c), Theme.kbd_key_press_color());
        effects.push(e);
    });

    accrued_delay += 300;
    let e = key_press(
        Duration::from_millis(accrued_delay),
        KeyCap::new(KeyCode::Enter, esc_area),
        Theme.kbd_key_press_color()
    );
    effects.push(e);

    effects.push(fx::delay(accrued_delay + 200, fx::parallel(&[
        fx::never_complete(led_kbd_border()),
        fx::fade_from_fg(CATPPUCCIN.crust, (800, Interpolation::SineOut))
    ])));

    fx::parallel(&effects)
}

/// Creates a color cycling effect for cell foregrounds.
///
/// # Arguments
/// * `colors` - ColorCycle instance defining the color sequence
/// * `step_duration` - Duration in milliseconds between color steps
/// * `predicate` - Function determining which cells should be affected
///
/// # Type Parameters
/// * `I` - Color index resolver type
/// * `P` - Predicate function type
///
/// # Returns
/// An Effect that cycles colors on cells matching the predicate.
pub fn color_cycle_fg<I, P>(
    colors: ColorCycle<I>,
    step_duration: u32,
    predicate: P,
) -> Effect where
    I: IndexResolver<Color> + Clone + Debug + Send + 'static,
    P: Fn(&Cell) -> bool + 'static
{
    use tachyonfx::fx::*;

    let duration = Duration::from_millis(u32::MAX);
    effect_fn((colors, None), duration, move |(colors, started_at), _ctx, cell_iter| {
        if started_at.is_none() {
            *started_at = Some(Instant::now());
        }

        let elapsed = started_at.as_ref().unwrap().elapsed().as_millis().max(1);
        let raw_color_idx = elapsed as u32 / step_duration;

        let color = |pos: Position| -> Color {
            let idx = (raw_color_idx + (pos.x / 2 + pos.y * 3 / 2) as u32) as usize;
            *colors.color_at(idx)
        };

        cell_iter
            .filter(|(_, c)| predicate(c))
            .map(|(pos, cell)| (color(pos), cell))
            .for_each(|(color, cell)| {
                cell.set_fg(color);
            });
    })
}

/// Creates an animated LED border effect for the keyboard.
///
/// Uses the theme's LED colors in a ping-pong pattern, affecting
/// all key symbols.
///
/// # Returns
/// A persistent Effect that animates the keyboard border lights.
pub fn led_kbd_border() -> Effect {
    let [color_1, color_2, color_3] = Theme.kbd_led_colors();

    let color_cycle = PingPongColorCycle::new(color_1, &[
        (40, color_2),
        (20, color_3),
    ]);

    color_cycle_fg(color_cycle, 100, |cell| {
        let symbol = cell.symbol();
        symbol != " " && !symbol.chars().next().map(is_box_drawing).unwrap_or(false)
    })
}


/// Creates an effect that dispatches an event as soon as it starts.
///
/// # Type Parameters
/// * `T` - Event type that implements Clone and 'static
///
/// # Arguments
/// * `sender` - Channel for sending the event
/// * `event` - Event to be dispatched
///
/// # Returns
/// An Effect that dispatches the specified event.
pub fn dispatch_event<T: Clone + Debug + Send + 'static>(
    sender: Sender<T>,
    event: T
) -> Effect {
    effect_fn_buf(Some(event), 1, move |e, _, _| {
        if let Some(e) = e.take() { sender.dispatch(e) }
    })
}

/// Creates an effect highlighting keyboard keys relevant to the selected category.
///
/// # Arguments
/// * `stage` - Effect stage for managing the animation
/// * `context` - Current keymap context
/// * `buffer_size` - Size of the rendering buffer
///
/// # Returns
/// A unique Effect that outlines and animates relevant key caps.
pub fn outline_selected_category_key_caps(
    stage: &mut EffectStage<UniqueEffectId>,
    context: &KeyMapContext,
    buffer_size: Size,
) -> Effect {
    let buf = Buffer::empty(Rect::from((Position::default(), buffer_size)));
    let outline = KeyCapOutline::new(buf, context).into_effect();

    let color = Theme.kbd_cap_outline_category(context.sorted_category_idx()
        .expect("selected category"))
        .fg
        .expect("fg color");

    let keycap_outline = CellFilter::FgColor(color);

    let fx = parallel(&[
        outline,
        sequence(&[
            sweep_in(UpToDown, 40, 40, CATPPUCCIN.crust, (350, Interpolation::QuadIn)),
            color_cycle_fg(select_category_color_cycle(color, 9), 33, |_| true),
        ]).with_cell_selection(keycap_outline),
    ]);

    stage.unique(UniqueEffectId::KeyCapOutline, fx)
}


fn draw_single_border(key_cap: KeyCap, duration: Duration) -> Effect {
    use tachyonfx::fx::*;
    let border_style = Style::default().fg(Catppuccin::new().base);

    effect_fn_buf((), duration, move |_state, _ctx, buf| {
        render_border_with(&[key_cap.clone()], buf, move |d, _pos, cell| {
            draw_key_border(d, cell);
            cell.set_style(border_style);
        });
    })
}

fn clear_cells(duration: Duration) -> Effect {
    use tachyonfx::fx::*;
    effect_fn((), duration, |_state, _ctx, cells| {
        cells.for_each(|(_, cell)| {
            cell.set_char(' ');
        });
    })
}

fn is_box_drawing(c: char) -> bool {
    ('\u{2500}'..='\u{257F}').contains(&c)
}

/// Creates a repeating color cycle based on a base color.
///
/// # Arguments
/// * `base_color` - Primary color to derive the cycle from
/// * `length_multiplier` - Factor to adjust the cycle length
///
/// # Returns
/// A ColorCycle instance with derived colors and adjusted steps.
fn select_category_color_cycle(
    base_color: Color,
    length_multiplier: usize
) -> ColorCycle<RepeatingCycle> {
    let color_step: usize = 7 * length_multiplier;

    let (h, s, l) = base_color.to_hsl();

    let color_l = Color::from_hsl(h, s, 80.0);
    let color_d = Color::from_hsl(h, s, 40.0);

    
    RepeatingColorCycle::new(base_color, &[
        (4 * length_multiplier, color_d),
        (2 * length_multiplier, color_l),
        (4 * length_multiplier, Color::from_hsl((h - 25.0) % 360.0, s, (l + 10.0).min(100.0))),
        (color_step, Color::from_hsl(h, (s - 20.0).max(0.0), (l + 10.0).min(100.0))),
        (color_step, Color::from_hsl((h + 25.0) % 360.0, s, (l + 10.0).min(100.0))),
        (color_step, Color::from_hsl(h, (s + 20.0).max(0.0), (l + 10.0).min(100.0))),
    ])
}

