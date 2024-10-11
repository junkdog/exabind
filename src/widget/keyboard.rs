use crossterm::event::{KeyCode, ModifierKeyCode};
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Alignment, Offset, Rect, Size};
use ratatui::prelude::{Color, Position};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Widget, WidgetRef};
use tachyonfx::CellIterator;
// ref: https://upload.wikimedia.org/wikipedia/commons/3/3a/Qwerty.svg

// override with custom styles for key codes
pub struct KeyboardWidget {

}


pub trait KeyboardLayout {
    fn key_area(&self, key_code: KeyCode) -> Rect;
    fn key_position(&self, key_code: KeyCode) -> Position;
    fn layout(&self) -> Vec<(KeyCode, Rect)>;
}

#[derive(Default)]
pub struct AnsiKeyboardTklLayout;

macro_rules! kbd_layout {
    [$self:expr; $($key:expr),+ $(,)?] => {
        [
            $(
                ($key, $self.key_area($key)),
            )+
        ]
    };
}

impl KeyboardLayout for AnsiKeyboardTklLayout {
    fn key_area(&self, key_code: KeyCode) -> Rect {
        let size = match key_code {
            KeyCode::Char(' ') => Size::new(SPACE_W, KEY_H),
            KeyCode::Char('\\') => Size::new(8, KEY_H),
            KeyCode::Tab => Size::new(TAB_W, KEY_H),
            KeyCode::CapsLock => Size::new(CAPSLOCK_W, KEY_H),
            KeyCode::Backspace => Size::new(KEY_W * 2 - 1, KEY_H),
            KeyCode::Enter => Size::new(12, KEY_H),
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
            }
            _ => Size::new(KEY_W, KEY_H)
        };

        (self.key_position(key_code), size).into()
    }

    fn key_position(&self, key_code: KeyCode) -> Position {
        let offset = |row: &str, c: char| 1 + KEY_W * row.find(c).map_or(0, |i| i as u16);

        let fn_key_x = |n: u8| -> u16 {
            // F1 aligns with '2', but n is not zero-indexed, so we align it with '1'
            let start = 1 + KEY_W;

            // group gap is ~3
            let group_gap = 2 * (((n as u16 - 1) / 4));

            return start + group_gap + n as u16 * KEY_W;
        };

        let (x, y) = match key_code {
            KeyCode::Esc => (1, 0),
            KeyCode::F(n) => (fn_key_x(n), 0),
            KeyCode::Char(c) if NUMBER_ROW.contains(c) => (offset(NUMBER_ROW, c), 3),
            KeyCode::Char(c) if TOP_ROW.contains(c) => (TAB_W + offset(TOP_ROW, c), 5),
            KeyCode::Char(c) if MIDDLE_ROW.contains(c) => (CAPSLOCK_W + offset(MIDDLE_ROW, c), 7),
            KeyCode::Char(c) if BOTTOM_ROW.contains(c) => (SHIFT_L_W + offset(BOTTOM_ROW, c), 9),
            KeyCode::Char(' ') => (1 + CTRL_L_W + SUPER_W + ALT_W, 11),
            KeyCode::Char(c) => (0, 0),
            KeyCode::Backspace => (1 + 13 * KEY_W, 3),
            KeyCode::Tab => (1, 5),
            KeyCode::CapsLock => (1, 7),
            KeyCode::Enter => (1 + CAPSLOCK_W + 11 * KEY_W, 7),
            KeyCode::Left => (NAV_KEY_X_START, 11),
            KeyCode::Right => (NAV_KEY_X_START + KEY_W + KEY_W, 11),
            KeyCode::Up => (NAV_KEY_X_START + KEY_W, 9),
            KeyCode::Down => (NAV_KEY_X_START + KEY_W, 11),
            KeyCode::Home => (NAV_KEY_X_START + KEY_W, 3),
            KeyCode::End => (NAV_KEY_X_START + KEY_W, 5),
            KeyCode::PageUp => (NAV_KEY_X_START + KEY_W * 2, 3),
            KeyCode::PageDown => (NAV_KEY_X_START + KEY_W * 2, 5),
            KeyCode::BackTab => (0, 0),
            KeyCode::Delete => (NAV_KEY_X_START, 5),
            KeyCode::Insert => (NAV_KEY_X_START, 3),
            KeyCode::Null => (0, 0),
            KeyCode::ScrollLock => (NAV_KEY_X_START + KEY_W, 0),
            KeyCode::NumLock => (0, 0),
            KeyCode::PrintScreen => (NAV_KEY_X_START, 0),
            KeyCode::Pause => (NAV_KEY_X_START + KEY_W * 2, 0),
            KeyCode::Menu => (1 + CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W, 11),
            KeyCode::KeypadBegin => (0, 0),
            KeyCode::Media(_) => (0, 0),
            KeyCode::Modifier(c) => match c {
                ModifierKeyCode::LeftShift => (1, 9),
                ModifierKeyCode::RightShift => (1 + SHIFT_L_W + KEY_W * BOTTOM_ROW.len() as u16, 9),

                ModifierKeyCode::LeftControl => (1, 11),
                ModifierKeyCode::LeftSuper => (1 + CTRL_L_W, 11),
                ModifierKeyCode::LeftHyper => (1 + CTRL_L_W, 11),
                ModifierKeyCode::LeftMeta => (1 + CTRL_L_W, 11),
                ModifierKeyCode::LeftAlt => (1 + CTRL_L_W + SUPER_W, 11),
                ModifierKeyCode::RightAlt => (1 + CTRL_L_W + SUPER_W + ALT_W + SPACE_W, 11),
                ModifierKeyCode::RightControl => (1 + CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W + KEY_W, 11),
                ModifierKeyCode::RightSuper => (1 + CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W, 11),
                ModifierKeyCode::RightHyper => (1 + CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W, 11),
                ModifierKeyCode::RightMeta => (1 + CTRL_L_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W, 11),
                ModifierKeyCode::IsoLevel3Shift => (0, 11), // ignore
                ModifierKeyCode::IsoLevel5Shift => (0, 11), // ignore
            },
        };

        Position::new(x, y)
    }

    fn layout(&self) -> Vec<(KeyCode, Rect)> {
        use KeyCode::*;
        use ModifierKeyCode::*;

        kbd_layout![self;
            // function key row
            Esc, F(1), F(2),  F(3), F(4), F(5), F(6), F(7), F(8), F(9), F(10), F(11), F(12),

            // number row
            Char('`'), Char('1'), Char('2'), Char('3'), Char('4'), Char('5'), Char('6'), Char('7'),
            Char('8'), Char('9'), Char('0'), Char('-'), Char('='), Backspace,

            // // top row
            Tab, Char('q'), Char('w'), Char('e'), Char('r'), Char('t'), Char('y'), Char('u'),
            Char('i'), Char('o'), Char('p'), Char('['), Char(']'), Char('\\'),

            // // middle row
            CapsLock, Char('a'), Char('s'), Char('d'), Char('f'), Char('g'), Char('h'), Char('j'),
            Char('k'), Char('l'), Char(';'), Char('\''), Enter,

            // bottom row
            Modifier(LeftShift), Char('z'), Char('x'), Char('c'), Char('v'), Char('b'), Char('n'),
            Char('m'), Char(','), Char('.'), Char('/'), Modifier(RightShift),

            // bottom row
            Modifier(LeftControl), Modifier(LeftSuper), Modifier(LeftAlt), Char(' '),
            Modifier(RightAlt), Menu, Modifier(RightSuper), Modifier(RightControl),
            //
            PrintScreen, ScrollLock, Pause,
            //
            Insert, Home, PageUp,
            Delete, End, PageDown,
            //
            // // cursor keys
            Up, Left, Down, Right,
        ].into()
    }
}

impl KeyboardWidget {
    pub fn new() -> Self {
        Self {
        }
    }

    fn draw_key_border(
        decorate: char,
        cell: &mut Cell,
    ) {
        let current = cell.symbol().chars().next().unwrap();
        match decorate {
            '└' => match current {
                ' ' | '─' => cell.set_char('└'),
                '┘' => cell.set_char('╨'),
                '╡' => cell.set_char('╬'),
                '┐' => cell.set_char('╪'),
                '╩' => cell.set_char(current),
                n => panic!("Invalid border character: {}", n),
            },
            '┌' => match current {
                ' ' | '─' => cell.set_char('┌'),
                '┘' => cell.set_char('╪'),
                '╡' => cell.set_char('╫'),
                '┤' => cell.set_char('╫'),
                '┐' => cell.set_char('╥'),
                '│' => cell.set_char(current),
                '└' => cell.set_char('├'),
                '╨' => cell.set_char('╫'),
                '╫' => cell.set_char(current),
                '╪' => cell.set_char('╫'),
                n => panic!("Invalid border character: {}", n),
            },
            '┐' => match current {
                ' ' | '─' => cell.set_char('┐'),
                '┌' => cell.set_char('╥'),
                '┘' => cell.set_char('┤'),
                '└' => cell.set_char('╪'),
                '╨' => cell.set_char('╫'),
                n => panic!("Invalid border character: {}", n),
            },
            '┘' => match current {
                ' ' | '─' => cell.set_char('┘'),
                '┌' => cell.set_char('╪'),
                '└' => cell.set_char('╨'),
                n => panic!("Invalid border character: {}", n),
            },
            '│' => match current {
                ' ' => cell.set_char('│'),
                '│' => cell.set_char('║'),
                // n => panic!("Invalid border character: {}", n),
                _ => cell.set_char('|'),
            },
            _ => panic!("Invalid border character"),
        };
    }

    fn render_key(key_code: KeyCode, area: Rect, buf: &mut Buffer) {
        // draw key border, left
        let (x, y) = (area.x.saturating_sub(1), area.y);
        Self::draw_key_border('┌', &mut buf[(x, y + 0)]);
        Self::draw_key_border('│', &mut buf[(x, y + 1)]);
        Self::draw_key_border('└', &mut buf[(x, y + 2)]);

        // horizontal borders
        for x in area.x..area.x + area.width - 1 {
            let cell = &mut buf[(x, area.y + 0)];
            if cell.symbol() == " " {
                cell.set_char('─');
            }

            let cell = &mut buf[(x, area.y + KEY_H - 1)];
            if cell.symbol() == " " {
                cell.set_char('─');
            }
        }

        // draw key border, right
        let (x, y) = (area.x + area.width - 1, area.y);
        Self::draw_key_border('┐', &mut buf[(x, y + 0)]);
        Self::draw_key_border('│', &mut buf[(x, y + 1)]);
        Self::draw_key_border('┘', &mut buf[(x, y + 2)]);

        let key_string = match key_code {
            KeyCode::Esc => "ESC".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) if c == ' ' => "␣".to_string(),
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
            KeyCode::Modifier(modifier) => match modifier {
                ModifierKeyCode::LeftShift => "⇧".to_string(),
                ModifierKeyCode::RightShift => "⇧".to_string(),
                ModifierKeyCode::LeftControl => "CTRL".to_string(),
                ModifierKeyCode::LeftSuper => "⌘L".to_string(),
                ModifierKeyCode::LeftHyper => "Hyp".to_string(),
                ModifierKeyCode::LeftMeta => "Meta".to_string(),
                ModifierKeyCode::LeftAlt => "Alt".to_string(),
                ModifierKeyCode::RightAlt => "Alt".to_string(),
                ModifierKeyCode::RightControl => "CTRL".to_string(),
                ModifierKeyCode::RightSuper => "⌘R".to_string(),
                ModifierKeyCode::RightHyper => "Hyp".to_string(),
                ModifierKeyCode::RightMeta => "Meta".to_string(),
                ModifierKeyCode::IsoLevel3Shift => "Iso3".to_string(),
                ModifierKeyCode::IsoLevel5Shift => "Iso5".to_string(),
            },
        };

        let alignment = if key_string.char_indices().count() != 3 { Alignment::Center } else { Alignment::Left };
        Text::from(Span::from(key_string))
            .alignment(alignment)
            .render(area.offset(Offset { x: 0, y: 1 }), buf);
    }
}

impl WidgetRef for KeyboardWidget {
    fn render_ref(
        &self,
        area: Rect,
        buf: &mut Buffer
    ) {
        AnsiKeyboardTklLayout::default()
            .layout()
            .iter()
            .filter(|(_, key_area)| area.union(*key_area) == area)
            .for_each(|(key_code, area)| Self::render_key(*key_code, *area, buf));
    }
}

// const NAV_KEY_X_START: u16 = 1 + CTRL_W + SUPER_W + ALT_W + SPACE_W + ALT_W + MENU_W + 4 + SHIFT_R_W + 4;
const NAV_KEY_X_START: u16 = 78;

const KEY_W: u16 = 5; // includes | delimited
const KEY_H: u16 = 3;

const NUMBER_ROW: &str = "`1234567890-=";
const TOP_ROW: &str = "qwertyuiop[]\\";
const MIDDLE_ROW: &str = "asdfghjkl;'";
const BOTTOM_ROW: &str = "zxcvbnm,./";

const TAB_W: u16 = 6;
const CAPSLOCK_W: u16 = 7;
const SHIFT_L_W: u16 = 9;
const SHIFT_R_W: u16 = 15;
const CTRL_L_W: u16 = 6;
const CTRL_R_W: u16 = 9;
const ALT_W: u16 = 7;
const SPACE_W: u16 = 30;

const SUPER_W: u16 = 5;
const MENU_W: u16 = 5;