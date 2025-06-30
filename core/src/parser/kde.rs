use crate::keymap::KeyMap;
use crate::parser::kde::line::kglobalshortcuts_parser;
use crate::shortcut::{Action, Shortcut};
use anpa::combinators::{attempt, many, many_to_vec, middle, no_separator, right, separator};
use anpa::core::{parse, ParserExt, StrParser};
use anpa::parsers::{item_while, skip, until};
use anpa::{or, right, tuplify};
use crossterm::event::{KeyCode, MediaKeyCode};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum ParsedLine<'a> {
    SectionHeader(&'a str),
    SectionFriendlyName(&'a str),
    Shortcut {
        id: &'a str,
        shortcut: &'a str,
        default_shortcut: &'a str,
        label: &'a str,
    }
}

pub fn parse_kglobalshortcuts(input: &str) -> KeyMap {
    let res = parse(kglobalshortcuts_parser(), input);
    let parsed_lines = res.result.unwrap();

    let rlines: Vec<ParsedLine> = parsed_lines.into_iter().rev().collect();

    let actions: HashMap<String, Vec<Action>> = rlines
        .split_inclusive(|line| matches!(line, ParsedLine::SectionHeader(_)))
        .filter_map(application_actions)
        .collect();

    KeyMap::new("KDE", actions)
}

fn parse_shortcuts(s: &str) -> Vec<Shortcut> {
    let hmm = s.to_string();
    // let res = parse(keys::shortcut_keystrokes(), s);
    let res = parse(keys::shortcut_keystrokes(), hmm.as_str());
    let shortcuts: Vec<Shortcut> = res.result.unwrap();
    shortcuts.clone()
}

fn application_actions(rlines: &[ParsedLine]) -> Option<(String, Vec<Action>)> {
    let mut category = "";
    let mut actions = Vec::new();

    rlines.iter().rev().for_each(|l| match l {
        ParsedLine::SectionHeader(s)                 => category = s,
        ParsedLine::SectionFriendlyName(s)           => category = s,
        ParsedLine::Shortcut { shortcut, label, .. } => {
            let shortcuts = parse_shortcuts(shortcut);
            actions.push(Action::new_filter_empty(label, &"", shortcuts));
        }
    });

    actions.retain(|a| !a.shortcuts().is_empty());
    actions.iter_mut()
        .for_each(|a| a.update_category(category));

    if actions.is_empty() {
        None
    } else {
        Some((category.to_string(), actions))
    }
}

// line parsers
mod line {
    use super::*;

    fn empty_line<'a>() -> impl StrParser<'a, ()> {
        attempt(right!(
            item_while(|c| c == ' '),
            skip('\n')
        ))
    }

    fn section_header<'a>() -> impl StrParser<'a, ParsedLine<'a>> {
        middle(skip('['), item_while(|c| c != ']'), skip(']'))
            .map(ParsedLine::SectionHeader)
    }

    fn section_friendly_name<'a>() -> impl StrParser<'a, ParsedLine<'a>> {
        right(skip("_k_friendly_name="), item_while(|c| c != '\n'))
            .map(ParsedLine::SectionFriendlyName)
    }

    fn shortcut<'a>() -> impl StrParser<'a, ParsedLine<'a>> {
        let id = until("=");
        let shortcut = until(",");
        let default_shortcut = until(",");
        let label = item_while(|c| c != '\n');

        tuplify!(id, shortcut, default_shortcut, label)
            .map(|(id, shortcut, default_shortcut, label)| ParsedLine::Shortcut {
                id,
                shortcut,
                default_shortcut,
                label,
            })
    }

    pub(super) fn kglobalshortcuts_parser<'a>() -> impl StrParser<'a, Vec<ParsedLine<'a>>> {
        let empty_lines = many(empty_line(), true, no_separator());
        let config_line = or!(section_header(), section_friendly_name(), shortcut());
        let config_entry = right!(empty_lines, config_line);

        many_to_vec(config_entry, true, no_separator())
    }

    #[cfg(test)]
    mod tests {
        use super::*;


        #[test]
        fn test_shortcut() {
            let input = "ExposeClass=none,Ctrl+F7,Toggle Present Windows (Window class)";
            let res = parse(shortcut(), input);
            assert_eq!(res.result, Some(ParsedLine::Shortcut {
                id: "ExposeClass",
                shortcut: "none",
                default_shortcut: "Ctrl+F7",
                label: "Toggle Present Windows (Window class)",
            }));

            let input = "Switch to Desktop 10=none,,Switch to Desktop 10";
            let res = parse(shortcut(), input);
            assert_eq!(res.result, Some(ParsedLine::Shortcut {
                id: "Switch to Desktop 10",
                shortcut: "none",
                default_shortcut: "",
                label: "Switch to Desktop 10",
            }));
        }

        #[test]
        fn test_section_header() {
            let input = "[section37]";
            let res = parse(section_header(), input);
            assert_eq!(res.result, Some(ParsedLine::SectionHeader("section37")));
            assert_eq!(res.state, "");
        }

        #[test]
        fn test_section_friendly_name() {
            let input = "_k_friendly_name=section 31 covert ops\n";
            let res = parse(section_friendly_name(), input);
            assert_eq!(res.result, Some(ParsedLine::SectionFriendlyName("section 31 covert ops")));
            assert_eq!(res.state, "\n");
        }

        #[test]
        fn test_section_friendly_name_no_linebreak() {
            let input = "_k_friendly_name=section 31 covert ops";
            let res = parse(section_friendly_name(), input);
            assert_eq!(res.result, Some(ParsedLine::SectionFriendlyName("section 31 covert ops")));
            assert_eq!(res.state, "");
        }
    }
}

// keystroke parsers
mod keys {
    use super::*;


    fn shortcut_keystroke<'a>() -> impl StrParser<'a, Shortcut> {
        many_to_vec(key_code(), true, separator(or!(skip('+')), false))
            .map(|keys| keys.into_iter().filter(|k| k != &KeyCode::Null).collect())
            .map(Shortcut::new)
    }

    fn key_code<'a>() -> impl StrParser<'a, KeyCode> {
        use crossterm::event::{KeyCode::*, ModifierKeyCode::*};

        // special case for backslashes so that we can use \ as end delimiter in item_while
        or!(
            skip(r#"\\\\"#).map(|_| "\\"),
            skip("none").map(|_| ""),
            item_while(|c| c != '+' && c != ',' && c != '\\'),
        ).map(|k: &str| match k {
            s if s.chars().count() == 1 => Char(s.chars().next().unwrap().to_lowercase().next().unwrap()),
            "Ctrl"                      => Modifier(LeftControl),
            "Alt"                       => Modifier(LeftAlt),
            "Shift"                     => Modifier(LeftShift),
            "Super"                     => Modifier(LeftSuper),
            "Hyper"                     => Modifier(LeftHyper),
            "Meta"                      => Modifier(LeftMeta),
            // Function keys
            "F1"                        => F(1),
            "F2"                        => F(2),
            "F3"                        => F(3),
            "F4"                        => F(4),
            "F5"                        => F(5),
            "F6"                        => F(6),
            "F7"                        => F(7),
            "F8"                        => F(8),
            "F9"                        => F(9),
            "F10"                       => F(10),
            "F11"                       => F(11),
            "F12"                       => F(12),
            // Navigation keys
            "Up"                        => Up,
            "Down"                      => Down,
            "Left"                      => Left,
            "Right"                     => Right,
            "PgUp"                      => PageUp,
            "PgDown"                    => PageDown,
            "Tab"                       => Tab,
            "Backtab"                   => BackTab,
            // Special keys
            "Esc"                       => Esc,
            "Del"                       => Delete,
            "Space"                     => Char(' '),
            // Media keys
            "Media Play"                => Media(MediaKeyCode::Play),
            "Media Pause"               => Media(MediaKeyCode::Pause),
            "Media Stop"                => Media(MediaKeyCode::Stop),
            "Media Next"                => Media(MediaKeyCode::TrackNext),
            "Media Previous"            => Media(MediaKeyCode::TrackPrevious),
            // Volume controls
            "Volume Up"                 => Media(MediaKeyCode::RaiseVolume),
            "Volume Down"               => Media(MediaKeyCode::LowerVolume),
            "Volume Mute"               => Media(MediaKeyCode::MuteVolume),
            // Other special keys
            "Print"                     => PrintScreen,
            "Num"                       => NumLock,

            _                           => Null,
        })
    }

    pub(super) fn shortcut_keystrokes<'a>() -> impl StrParser<'a, Vec<Shortcut>> {
        fn valid_shortcut(s: &Shortcut) -> bool {
            !s.keystroke().is_empty()
        }

        many_to_vec(shortcut_keystroke(), true, separator(skip("\\t"), false))
            .map(|shortcuts| shortcuts.into_iter()
                .filter(valid_shortcut)
                .collect()
            )
    }

    #[cfg(test)]
    mod tests {
        use crossterm::event::ModifierKeyCode;
        use super::*;

        #[test]
        fn test_key_code() {
            let input = "Ctrl+F7";
            let res = parse(key_code(), input);
            assert_eq!(res.result, Some(KeyCode::Modifier(ModifierKeyCode::LeftControl)));
            assert_eq!(res.state, "+F7");

            let input = "F12";
            let res = parse(key_code(), input);
            assert_eq!(res.result, Some(KeyCode::F(12)));
            assert_eq!(res.state, "");

            let input = "Media Play";
            let res = parse(key_code(), input);
            assert_eq!(res.result, Some(KeyCode::Media(MediaKeyCode::Play)));
            assert_eq!(res.state, "");

            let input = "Volume Down";
            let res = parse(key_code(), input);
            assert_eq!(res.result, Some(KeyCode::Media(MediaKeyCode::LowerVolume)));
            assert_eq!(res.state, "");
        }

        #[test]
        fn test_shortcut_keystroke() {
            let input = "Ctrl+F7";
            let res = parse(shortcut_keystroke(), input);
            assert_eq!(res.result, Some(Shortcut::new(vec![
                KeyCode::Modifier(ModifierKeyCode::LeftControl),
                KeyCode::F(7),
            ])));

            let input = "Ctrl+F7+Shift";
            let res = parse(shortcut_keystroke(), input);
            assert_eq!(res.result, Some(Shortcut::new(vec![
                KeyCode::Modifier(ModifierKeyCode::LeftControl),
                KeyCode::Modifier(ModifierKeyCode::LeftShift),
                KeyCode::F(7),
            ])));
        }

        #[test]
        fn test_shortcut_with_multiple_bindings() {
            let input = "Meta+Down\\tVolume Down";
            let res = parse(shortcut_keystrokes(), input);
            assert_eq!(res.result, Some(vec![
                Shortcut::new(vec![
                    KeyCode::Modifier(ModifierKeyCode::LeftMeta),
                    KeyCode::Down,
                ]),
                Shortcut::new(vec![
                    KeyCode::Media(MediaKeyCode::LowerVolume),
                ]),
            ]));

            let input = r#"Ctrl+\\\\\tMeta+0"#;
            let res = parse(shortcut_keystrokes(), input);
            assert_eq!(res.result, Some(vec![
                Shortcut::new(vec![
                    KeyCode::Modifier(ModifierKeyCode::LeftControl),
                    KeyCode::Char('\\'),
                ]),
                Shortcut::new(vec![
                    KeyCode::Modifier(ModifierKeyCode::LeftMeta),
                    KeyCode::Char('0'),
                ]),
            ]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anpa::core::parse;
    use std::io::Read;

    #[test]
    fn test_parse_shortcuts() {
        let input = "Ctrl+Alt+Esc\\tMeta+Alt+Down";
        let shortcuts = parse_shortcuts(input);

        use crossterm::event::{KeyCode::*, ModifierKeyCode::*};
        assert_eq!(shortcuts, vec![
            Shortcut::new(vec![
                Modifier(LeftControl),
                Modifier(LeftAlt),
                Esc,
            ]),
            Shortcut::new(vec![
                Modifier(LeftMeta),
                Modifier(LeftAlt),
                Down,
            ]),
        ]);

        let input = "Meta+Ctrl+F";
        let shortcuts = parse_shortcuts(input);
        assert_eq!(shortcuts, vec![
            Shortcut::new(vec![
                Modifier(LeftMeta),
                Modifier(LeftControl),
                Char('f'),
            ]),
        ]);
    }

    #[test]
    fn test_parse_keymap() -> std::io::Result<()> {
        let mut input = String::new();
        let mut f = std::fs::File::open("./test/kglobalshortcutsrc")?;
        f.read_to_string(&mut input)?;

        let keymap = parse_kglobalshortcuts(&input);
        println!("{}", keymap);

        Ok(())
    }

    #[test]
    fn test_kglobalshortcuts_parser()  -> std::io::Result<()> {
        let mut input = String::new();
        let mut f = std::fs::File::open("./test/kglobalshortcutsrc")?;
        f.read_to_string(&mut input)?;

        let res = parse(kglobalshortcuts_parser(), &input);
        assert!(res.result.is_some());

        let parsed_lines = res.result.unwrap();
        println!("{:?}", parsed_lines);

        Ok(())
    }
}