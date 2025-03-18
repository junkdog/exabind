use crate::styling::{Catppuccin, ExabindTheme, Theme};
use crossterm::event::{KeyCode, ModifierKeyCode};
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Alignment, Margin, Rect, Size};
use ratatui::prelude::Position;
use ratatui::style::Style;
use ratatui::text::{Span, Text};
use ratatui::widgets::{Widget, WidgetRef};
use std::collections::HashMap;
// ref: https://upload.wikimedia.org/wikipedia/commons/3/3a/Qwerty.svg

// override with custom styles for key codes
pub struct KeyboardWidget {
    keys: Vec<KeyCap>,
    cap_style: Style,
    border_style: Option<Style>,
}

pub trait KeyboardLayout {
    fn key_area(&self, key_code: KeyCode) -> Rect;
    fn key_position(&self, key_code: KeyCode) -> Position;
    fn layout(&self) -> Vec<KeyCap>;
    fn key_cap(&self, c: char) -> KeyCap {
        let key_code = KeyCode::Char(c);
        KeyCap::new(key_code, self.key_area(key_code))
    }

    fn key_cap_lookup(&self) -> HashMap<KeyCode, KeyCap> {
        self.layout()
            .iter()
            .map(|key_cap| (key_cap.key_code, key_cap.clone()))
            .collect()
    }
}

#[derive(Default)]
pub struct AnsiKeyboardTklLayout;

macro_rules! kbd_layout {
    [$self:expr; $($key:expr),+ $(,)?] => {
        [
            $(
                KeyCap::new($key, $self.key_area($key)),
            )+
        ]
    };
}

impl KeyboardLayout for AnsiKeyboardTklLayout {
    fn key_area(&self, key_code: KeyCode) -> Rect {
        let size = match supplant_key_code(key_code) {
            KeyCode::Char(' ') => Size::new(SPACE_W, KEY_H),
            KeyCode::Char('\\') => Size::new(9, KEY_H),
            KeyCode::Tab => Size::new(TAB_W, KEY_H),
            KeyCode::CapsLock => Size::new(CAPSLOCK_W, KEY_H),
            KeyCode::Backspace => Size::new(KEY_W * 2 - 2, KEY_H),
            KeyCode::Enter => Size::new(13, KEY_H),
            KeyCode::Modifier(c) => match c {
                ModifierKeyCode::LeftShift => Size::new(SHIFT_L_W, KEY_H),
                ModifierKeyCode::RightShift => Size::new(SHIFT_R_W, KEY_H),
                ModifierKeyCode::LeftControl => Size::new(CTRL_L_W, KEY_H),
                ModifierKeyCode::LeftSuper => Size::new(SUPER_W, KEY_H),
                ModifierKeyCode::LeftHyper => Size::new(SUPER_W, KEY_H),
                ModifierKeyCode::LeftMeta => Size::new(SUPER_W, KEY_H),
                ModifierKeyCode::LeftAlt => Size::new(ALT_W, KEY_H),
                ModifierKeyCode::RightAlt => Size::new(ALT_W, KEY_H),
                ModifierKeyCode::RightControl => Size::new(CTRL_R_W, KEY_H),
                ModifierKeyCode::RightSuper => Size::new(SUPER_W, KEY_H),
                ModifierKeyCode::RightHyper => Size::new(SUPER_W, KEY_H),
                ModifierKeyCode::RightMeta => Size::new(SUPER_W, KEY_H),
                _ => Size::new(KEY_W, KEY_H),
            },
            _ => Size::new(KEY_W, KEY_H),
        };

        (self.key_position(key_code), size).into()
    }

    fn key_position(&self, key_code: KeyCode) -> Position {
        let offset = |row: &str, c: char| (KEY_W - 1) * row.find(c).map_or(0, |i| i as u16);

        let fn_key_x = |n: u8| -> u16 {
            // F1 aligns with '2', but n is not zero-indexed, so we align it with '1'
            let start = KEY_W - 1;

            // group gap is ~3
            let group_gap = 2 * ((n as u16 - 1) / 4);

            start + group_gap + n as u16 * (KEY_W - 1)
        };

        let key_offset = |n: u16| -> u16 { n * (KEY_W - 1) };

        use KeyCode::*;
        use ModifierKeyCode::*;

        let (x, y) = match key_code {
            Esc => (0, 0),
            F(n) => (fn_key_x(n), 0),
            Char(c) if NUMBER_ROW.contains(c) => (offset(NUMBER_ROW, c), 3),
            Char(c) if TOP_ROW.contains(c) => (TAB_W - 1 + offset(TOP_ROW, c), 5),
            Char(c) if MIDDLE_ROW.contains(c) => (CAPSLOCK_W - 1 + offset(MIDDLE_ROW, c), 7),
            Char(c) if BOTTOM_ROW.contains(c) => (SHIFT_L_W - 1 + offset(BOTTOM_ROW, c), 9),
            Char(' ') => (CTRL_L_W + SUPER_W + ALT_W - 3, 11),
            Char(_) => panic!("Invalid key code: {:?}", key_code),
            Modifier(LeftShift) => (0, 9),
            Modifier(RightShift) => (SHIFT_L_W - 1 + key_offset(BOTTOM_ROW.len() as u16), 9),
            Modifier(LeftControl) => (0, 11),
            Modifier(LeftSuper) => (CTRL_L_W - 1, 11),
            Modifier(LeftHyper) => (CTRL_L_W - 1, 11),
            Modifier(LeftMeta) => (CTRL_L_W - 1, 11),
            Modifier(LeftAlt) => (CTRL_L_W + SUPER_W - 2, 11),
            Modifier(RightAlt) => (CTRL_L_W + SUPER_W + ALT_W + SPACE_W - 4, 11),
            Modifier(RightControl) => (
                CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W + KEY_W - 7,
                11,
            ),
            Modifier(RightSuper) => (
                CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W - 6,
                11,
            ),
            Modifier(RightHyper) => (
                CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W - 6,
                11,
            ),
            Modifier(RightMeta) => (
                CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W - 6,
                11,
            ),
            Modifier(_) => (0, 0), // ignore
            Backspace => (key_offset(NUMBER_ROW.len() as u16), 3),
            Tab => (0, 5),
            CapsLock => (0, 7),
            Enter => (CAPSLOCK_W - 1 + key_offset(11), 7),
            Left => (NAV_KEY_X_START, 11),
            Right => (NAV_KEY_X_START + key_offset(2), 11),
            Up => (NAV_KEY_X_START + key_offset(1), 9),
            Down => (NAV_KEY_X_START + key_offset(1), 11),
            Home => (NAV_KEY_X_START + key_offset(1), 3),
            End => (NAV_KEY_X_START + key_offset(1), 5),
            PageUp => (NAV_KEY_X_START + key_offset(2), 3),
            PageDown => (NAV_KEY_X_START + key_offset(2), 5),
            BackTab => (0, 0),
            Delete => (NAV_KEY_X_START, 5),
            Insert => (NAV_KEY_X_START, 3),
            Null => (0, 0),
            ScrollLock => (NAV_KEY_X_START + KEY_W - 1, 0),
            NumLock => {
                let p = self.key_position(Menu);
                (p.x, p.y)
            }
            PrintScreen => (NAV_KEY_X_START, 0),
            Pause => (NAV_KEY_X_START + key_offset(2), 0),
            Menu => (CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W - 5, 11),
            KeypadBegin => (0, 0),
            Media(_) => (0, 0),
        };

        Position::new(x, y)
    }

    fn layout(&self) -> Vec<KeyCap> {
        use KeyCode::*;
        use ModifierKeyCode::*;

        kbd_layout![self;
            // function key row
            Esc, F(1), F(2),  F(3), F(4), F(5), F(6), F(7), F(8), F(9), F(10), F(11), F(12),

            // number row
            Char('`'), Char('1'), Char('2'), Char('3'), Char('4'), Char('5'), Char('6'), Char('7'),
            Char('8'), Char('9'), Char('0'), Char('-'), Char('='), Backspace,

            // top row
            Tab, Char('q'), Char('w'), Char('e'), Char('r'), Char('t'), Char('y'), Char('u'),
            Char('i'), Char('o'), Char('p'), Char('['), Char(']'), Char('\\'),

            // middle row
            CapsLock, Char('a'), Char('s'), Char('d'), Char('f'), Char('g'), Char('h'), Char('j'),
            Char('k'), Char('l'), Char(';'), Char('\''), Enter,

            // bottom row
            Modifier(LeftShift), Char('z'), Char('x'), Char('c'), Char('v'), Char('b'), Char('n'),
            Char('m'), Char(','), Char('.'), Char('/'), Modifier(RightShift),

            // bottom row
            Modifier(LeftControl), Modifier(LeftMeta), Modifier(LeftAlt), Char(' '),
            Modifier(RightAlt), Menu, Modifier(RightMeta), Modifier(RightControl),

            // nav keys
            PrintScreen, ScrollLock, Pause,

            Insert, Home, PageUp,
            Delete, End, PageDown,

            // cursor keys
            Up, Left, Down, Right,
        ]
        .into()
    }
}

pub fn render_border_with<F>(key_caps: &[KeyCap], buf: &mut Buffer, mut draw_border_fn: F)
where
    F: FnMut(char, Position, &mut Cell),
{
    for key_cap in key_caps {
        let area = key_cap.area;

        let mut draw_border = |decorate: char, x, y| {
            draw_border_fn(decorate, (x, y).into(), &mut buf[(x, y)]);
        };

        // draw key border, left
        let (x, y) = (area.x, area.y);
        draw_border('┌', x, y);
        draw_border('│', x, y + 1);
        draw_border('└', x, y + 2);

        // draw key border, right
        let (x, y) = (area.x + area.width - 1, area.y);
        draw_border('┐', x, y);
        draw_border('│', x, y + 1);
        draw_border('┘', x, y + 2);

        let mut draw_horizontal_border = |x, y| {
            let pos = (x, y).into();
            let cell = &mut buf[pos];
            // if " ─".contains(cell.symbol()) {
            draw_border_fn('─', pos, cell);
            // }
        };

        // draw top and bottom borders
        for x in area.x + 1..area.x + area.width - 1 {
            draw_horizontal_border(x, area.y);
            draw_horizontal_border(x, area.y + KEY_H - 1);
        }
    }
}

pub fn render_border(key_cap: KeyCap, border_style: Style, buf: &mut Buffer) {
    render_border_with(&[key_cap], buf, |d, _pos, cell| {
        draw_key_border(d, cell);
        cell.set_style(border_style);
    });
}

impl From<(KeyCode, Rect)> for KeyCap {
    fn from(val: (KeyCode, Rect)) -> Self {
        KeyCap::new(val.0, val.1)
    }
}

impl KeyboardWidget {
    pub fn new(keys: Vec<KeyCap>) -> Self {
        Self::new_with_style(keys, Theme.kbd_cap_text(), Some(Theme.kbd_cap_border()))
    }

    pub fn new_with_style(
        keys: Vec<KeyCap>,
        cap_style: Style,
        border_style: Option<Style>,
    ) -> Self {
        Self {
            keys,
            cap_style,
            border_style,
        }
    }
}

impl WidgetRef for KeyboardWidget {
    fn render_ref(&self, _area: Rect, buf: &mut Buffer) {
        self.keys
            .iter()
            .map(|key| KeyCapWidget::new(key.clone(), self.cap_style, self.border_style))
            .for_each(|w| w.render(Rect::default(), buf));
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyCap {
    pub key_code: KeyCode,
    pub area: Rect,
}

#[derive(Debug)]
pub struct KeyCapWidget {
    key_cap: KeyCap,
    cap_style: Style,
    border_style: Option<Style>,
}

impl KeyCapWidget {
    pub fn new(key_cap: KeyCap, cap_style: Style, border_style: Option<Style>) -> Self {
        let colors = Catppuccin::new();

        use KeyCode::*;
        let other_color = colors.mantle;
        let cap_style = match key_cap.key_code {
            Esc | Tab | CapsLock | Modifier(_) | Menu | Char(' ') | Enter | Backspace => {
                cap_style.bg(other_color)
            }

            F(_n @ 5..=8) => cap_style.bg(other_color),

            Left | Right | Up | Down => cap_style.bg(other_color),

            Delete | Insert | Home | End | PageUp | PageDown | PrintScreen | ScrollLock | Pause => {
                cap_style.bg(other_color)
            }

            _ => cap_style,
        };

        Self {
            key_cap,
            cap_style,
            border_style,
        }
    }

    pub fn render_keypad(&self, buf: &mut Buffer) {
        let key_string = match self.key_cap.key_code {
            KeyCode::Esc => "ESC".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(' ') => "␣".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Backspace => "⌫".to_string(),
            KeyCode::Tab => "⇥".to_string(),
            KeyCode::CapsLock => "CAPS".to_string(),
            KeyCode::Enter => "⏎".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PgUp".to_string(),
            KeyCode::PageDown => "PgDn".to_string(),
            KeyCode::BackTab => "⇤".to_string(),
            KeyCode::Delete => "Del".to_string(),
            KeyCode::Insert => "Ins".to_string(),
            KeyCode::Null => "Null".to_string(),
            KeyCode::ScrollLock => "ScrL".to_string(),
            KeyCode::NumLock => "NumLk".to_string(),
            KeyCode::PrintScreen => "Prnt".to_string(),
            KeyCode::Pause => "Paus".to_string(),
            KeyCode::Menu => "Menu".to_string(),
            KeyCode::KeypadBegin => "KP5".to_string(),
            KeyCode::Media(media) => format!("Media({:?})", media),
            KeyCode::Modifier(ModifierKeyCode::LeftShift) => "⇧".to_string(),
            KeyCode::Modifier(ModifierKeyCode::RightShift) => "⇧".to_string(),
            KeyCode::Modifier(ModifierKeyCode::LeftControl) => "CTRL".to_string(),
            KeyCode::Modifier(ModifierKeyCode::LeftSuper) => "⌘L".to_string(),
            KeyCode::Modifier(ModifierKeyCode::LeftHyper) => "Hyp".to_string(),
            KeyCode::Modifier(ModifierKeyCode::LeftMeta) => "Meta".to_string(),
            KeyCode::Modifier(ModifierKeyCode::LeftAlt) => "Alt".to_string(),
            KeyCode::Modifier(ModifierKeyCode::RightAlt) => "Alt".to_string(),
            KeyCode::Modifier(ModifierKeyCode::RightControl) => "CTRL".to_string(),
            KeyCode::Modifier(ModifierKeyCode::RightSuper) => "⌘R".to_string(),
            KeyCode::Modifier(ModifierKeyCode::RightHyper) => "Hyp".to_string(),
            KeyCode::Modifier(ModifierKeyCode::RightMeta) => "Meta".to_string(),
            KeyCode::Modifier(ModifierKeyCode::IsoLevel3Shift) => "Iso3".to_string(),
            KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift) => "Iso5".to_string(),
        };

        let alignment = match key_string.char_indices().count() {
            1 => Alignment::Center,
            _ => Alignment::Left,
        };

        Text::from(Span::from(key_string))
            .style(self.cap_style)
            .alignment(alignment)
            .render(self.key_cap.area.inner(Margin::new(1, 1)), buf);
    }
}

impl KeyCap {
    pub fn new(key_code: KeyCode, area: Rect) -> Self {
        Self { key_code, area }
    }
}

impl Widget for KeyCapWidget {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        if let Some(border_style) = self.border_style {
            render_border(self.key_cap.clone(), border_style, buf);
        }
        self.render_keypad(buf);
    }
}

impl WidgetRef for KeyCapWidget {
    fn render_ref(&self, _area: Rect, buf: &mut Buffer) {
        if let Some(border_style) = self.border_style {
            render_border(self.key_cap.clone(), border_style, buf);
        }
        self.render_keypad(buf);
    }
}

pub fn draw_key_border(decorate: char, cell: &mut Cell) {
    let current = cell.symbol().chars().next().unwrap();
    match decorate {
        '└' => match current {
            ' ' => cell.set_char('└'),
            '─' => cell.set_char('X'),
            '┘' => cell.set_char('╨'),
            '╡' => cell.set_char('╬'),
            '┐' => cell.set_char('╪'),
            '╩' => cell.set_char(current),
            '┌' => cell.set_char('├'),
            _ => cell.set_char('Y'),
            // n => panic!("Invalid border character: {}", n),
        },
        '┌' => match current {
            ' ' => cell.set_char('┌'),
            '─' => cell.set_char('┬'),
            '┘' => cell.set_char('╪'),
            '╡' => cell.set_char('╫'),
            '┤' => cell.set_char('╫'),
            '┬' => cell.set_char('╥'),
            '┐' => cell.set_char('╥'),
            '└' => cell.set_char('├'),
            '╨' => cell.set_char('╫'),
            '╫' => cell.set_char(current),
            '╪' => cell.set_char('╫'),
            n => cell.set_char(n),
            // n => panic!("Invalid border character: {}", n),
        },
        '┐' => match current {
            ' ' => cell.set_char('┐'),
            '─' => cell.set_char('┬'),
            '┌' => cell.set_char('╥'),
            '┘' => cell.set_char('┤'),
            '└' => cell.set_char('╪'),
            '╨' => cell.set_char('╫'),
            _ => cell.set_char('┐'),
            // n => panic!("Invalid border character: {}", n),
        },
        '┘' => match current {
            ' ' => cell.set_char('┘'),
            '┌' => cell.set_char('╪'),
            '└' => cell.set_char('╨'),
            _ => cell.set_char('X'),
            // n => panic!("Invalid border character: {}", n),
        },
        '│' => match current {
            ' ' => cell.set_char('│'),
            '│' => cell.set_char('║'),
            _ => cell.set_char('|'),
        },
        '─' => match current {
            ' ' | '─' => cell.set_char('─'),
            '┘' | '└' => cell.set_char('┴'),
            '╨' | '╥' => cell.set_char(current),
            _ => cell.set_char('#'), // should never happen
        },
        c => panic!("Invalid border character: {}", c),
    };
}

// fixme: this is a mess - do something about it + what about localization
// translate shifted key_codes to their unshifted counterparts
pub fn supplant_key_code(key_code: KeyCode) -> KeyCode {
    use KeyCode::*;

    match key_code {
        Char('"') => Char('\''),
        Char('<') => Char(','),
        Char('>') => Char('.'),
        Char('?') => Char('/'),
        Char(':') => Char(';'),
        Char('_') => Char('-'),
        Char('+') => Char('='),
        Char('{') => Char('['),
        Char('}') => Char(']'),
        Char('|') => Char('\\'),
        Char('~') => Char('`'),
        Char('!') => Char('1'),
        Char('@') => Char('2'),
        Char('#') => Char('3'),
        Char('$') => Char('4'),
        Char('%') => Char('5'),
        Char('^') => Char('6'),
        Char('&') => Char('7'),
        Char('*') => Char('8'),
        Char('(') => Char('9'),
        Char(')') => Char('0'),

        key_code => key_code,
    }
}

const NAV_KEY_X_START: u16 = 79;

const KEY_W: u16 = 6; // includes | delimited
const KEY_H: u16 = 3;

const NUMBER_ROW: &str = "`1234567890-=";
const TOP_ROW: &str = "qwertyuiop[]\\";
const MIDDLE_ROW: &str = "asdfghjkl;'";
const BOTTOM_ROW: &str = "zxcvbnm,./";

const TAB_W: u16 = 7;
const CAPSLOCK_W: u16 = 8;
const SHIFT_L_W: u16 = 10;
const SHIFT_R_W: u16 = 16;
const CTRL_L_W: u16 = 7;
const CTRL_R_W: u16 = 10;
const ALT_W: u16 = 8;
const SPACE_W: u16 = 31;

const SUPER_W: u16 = 6;
const MENU_W: u16 = 6;
