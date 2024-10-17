use crossterm::event::KeyCode;

#[derive(Debug)]
struct KeyMap {
    version: String,
    name: String,
    parent: String,
    actions: Vec<Action>,
}

#[derive(Debug)]
struct Action {
    id: String,
    shortcuts: Vec<Shortcut>,
}

#[derive(Debug)]
struct Shortcut {
    keystroke: Vec<KeyCode>,
}

mod parser {
    use anpa::core::{ParserExt, StrParser};
    use anpa::parsers::item_while;
    use crossterm::event::KeyCode;

    fn as_keycode(key: &str) -> KeyCode {
        use crossterm::event::{KeyCode::*, ModifierKeyCode::*};

        match key {
            c if c.len() == 1 => Char(c.chars().next().unwrap()),
            "ctrl"            => Modifier(LeftControl),
            "shift"           => Modifier(LeftShift),
            "alt"             => Modifier(LeftAlt),
            "minus"           => Char('-'),
            "subtract"        => Char('-'),
            "plus"            => Char('+'),
            "f1"              => F(1),
            "f2"              => F(2),
            "f3"              => F(3),
            "f4"              => F(4),
            "f5"              => F(5),
            "f6"              => F(6),
            "f7"              => F(7),
            "f8"              => F(8),
            "f9"              => F(9),
            "f10"             => F(10),
            "f11"             => F(11),
            "f12"             => F(12),
            _                 => panic!("Unknown key: {}", key),
        }
    }

    fn parse_key<'a>() -> impl StrParser<'a, KeyCode> {
        item_while(|c: char| c.is_ascii_alphanumeric()).map(as_keycode)
    }
}


// fn jetbrains_xml_parser<'a>() -> impl StrParser<'a, XmlTag<'a>> {
//     xml_tag_parser()
// }

