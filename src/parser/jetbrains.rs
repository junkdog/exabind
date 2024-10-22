use crate::crossterm::format_keycode;
use crate::parser::xml::{xml_parser, XmlTag};
use anpa::core::{parse, StrParser};
use crossterm::event::KeyCode;
use std::fmt::Display;
use std::io::Read;
use std::path::PathBuf;
use crate::shortcut::{Action, Shortcut};

#[derive(Debug, Clone)]
pub struct KeyMap {
    version: String,
    name: String,
    parent: Option<String>,
    actions: Vec<Action>,
}



impl KeyMap {
    pub fn valid_actions(&self) -> impl Iterator<Item=&Action> {
        self.actions.iter().filter(|a| a.is_bound())
    }
}

impl Display for KeyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = |action: &Action| format!("\t{}", action.to_string());
        let parent = self.parent.as_ref().map(|s| s.as_str()).unwrap_or("");
        // let actions = self.actions.iter().map(format).collect::<Vec<_>>().join("\n");
        let actions = self.valid_actions().map(format).collect::<Vec<_>>().join("\n");
        write!(f, "keymap name={} parent={}:\n{}", self.name, parent, actions)
    }
}


mod parser {
    use crate::parser::core::eat;
    use anpa::combinators::{many_to_vec, no_separator, not_empty};
    use anpa::core::{parse, ParserExt, StrParser};
    use anpa::parsers::item_while;
    use crossterm::event::KeyCode;

    fn as_keycode(key: &str) -> KeyCode {
        use crossterm::event::{KeyCode::*, ModifierKeyCode::*};
        let lowercase = key.to_lowercase();
        match lowercase.as_str() {
            c if c.len() == 1    => Char(c.chars().next().unwrap()),
            "ctrl" | "control"   => Modifier(LeftControl),
            "shift"              => Modifier(LeftShift),
            "alt"                => Modifier(LeftAlt),
            "tab"                => Tab,
            "enter"              => Enter,
            "back_space"         => Backspace,
            "delete"             => Delete,
            "insert"             => Insert,
            "home"               => Home,
            "end"                => End,
            "page_up"            => PageUp,
            "page_down"          => PageDown,
            "left"               => Left,
            "right"              => Right,
            "up"                 => Up,
            "down"               => Down,
            "context_menu"       => Menu,
            "back_quote"         => Char('`'),
            "close_bracket"      => Char(']'),
            "open_bracket"       => Char('['),
            "space"              => Char(' '),
            "minus" | "subtract" => Char('-'),
            "divide"             => Char('/'),
            "multiply"           => Char('*'),
            "slash"              => Char('/'),
            "back_slash"         => Char('\\'),
            "plus" | "add"       => Char('+'),
            "equals"             => Char('='),
            "comma"              => Char(','),
            "period"             => Char('.'),
            "semicolon"          => Char(';'),
            "cancel"             => Esc, // fixme for the time being
            "numpad0"            => Esc, // fixme for the time being
            "numpad1"            => Esc, // fixme for the time being
            "numpad2"            => Esc, // fixme for the time being
            "numpad3"            => Esc, // fixme for the time being
            "numpad4"            => Esc, // fixme for the time being
            "numpad5"            => Esc, // fixme for the time being
            "numpad6"            => Esc, // fixme for the time being
            "numpad7"            => Esc, // fixme for the time being
            "numpad8"            => Esc, // fixme for the time being
            "numpad9"            => Esc, // fixme for the time being
            "f1"                 => F(1),
            "f2"                 => F(2),
            "f3"                 => F(3),
            "f4"                 => F(4),
            "f5"                 => F(5),
            "f6"                 => F(6),
            "f7"                 => F(7),
            "f8"                 => F(8),
            "f9"                 => F(9),
            "f10"                => F(10),
            "f11"                => F(11),
            "f12"                => F(12),
            "quote"              => Char('"'),
            "pause"              => Pause,
            "escape" | "esc"     => Esc,
            "back_tab"           => BackTab,
            "scroll_lock"        => ScrollLock,
            "num_lock"           => NumLock,
            "print_screen"       => PrintScreen,
            "menu"               => Menu,
            // k                  => Esc,
            k                    => panic!("Unknown key: '{}'", k),
        }
    }

    pub fn keycode_parser<'a>() -> impl StrParser<'a, KeyCode> {
        not_empty(item_while(|c: char| c.is_ascii_alphanumeric() || c == '_'))
            .map(as_keycode)
    }

    pub(super) fn parse_keycodes<'a>(input: &'a str) -> Vec<KeyCode> {
        let p = many_to_vec(eat(keycode_parser()), true, no_separator());
        parse(p, input).result.unwrap()
    }
}

fn as_shortcut(keyboard_shortcut_node: &XmlTag<'_>) -> Shortcut {
    debug_assert!(keyboard_shortcut_node.name() == "keyboard-shortcut");

    let keystroke = keyboard_shortcut_node
        .attribute("first-keystroke")
        .map(|s| parser::parse_keycodes(&s))
        .unwrap_or_default();

    Shortcut::new(keystroke)
}

fn as_action(action_node: &XmlTag<'_>) -> Action {
    debug_assert!(action_node.name() == "action");

    let id = action_node.attribute("id").expect("id to be present").to_string();
    let shortcuts = action_node.children()
        .iter()
        .filter(|c| c.name() == "keyboard-shortcut")
        .map(as_shortcut)
        .collect();

    Action::new(id, shortcuts)
}

fn as_keymap(xml: XmlTag<'_>) -> KeyMap {
    KeyMap {
        version: xml.attribute("version").expect("version to be present").to_string(),
        name: xml.attribute("name").expect("name to be present").to_string(),
        parent: xml.attribute("parent").map(|s| s.to_string()),
        actions: xml.children()
            .iter()
            .filter(|c| c.name() == "action")
            .map(as_action)
            .collect(),
    }
}

pub fn parse_jetbrains_keymap(input: &str) -> Option<KeyMap> {
    let res = parse(xml_parser(), input);
    debug_assert!(res.state.is_empty(), "Unparsed: '{}'", res.state);
    res.result.map(as_keymap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode::*, ModifierKeyCode::*};
    use std::io::Read;
    use std::path::PathBuf;
    use zip::read::ZipArchive;

    #[test]
    fn parse_keycode() {
        let p = parser::keycode_parser();
        let res = parse(p, "ctrl ");
        assert_eq!(res.state, " ");
        assert_eq!(res.result.unwrap(), Modifier(LeftControl));
    }

    #[test]
    fn parse_keycodes() {
        let input = "ctrl shift a";
        let key_codes = parser::parse_keycodes(input);
        assert_eq!(key_codes, vec![Modifier(LeftControl), Modifier(LeftShift), Char('a')]);
    }

    #[test]
    fn test_jetbrains_xml_parser() {
        let input = r#"<keymap version="1" name="Eclipse copy" parent="Eclipse">
    <action id="$Copy">
        <keyboard-shortcut first-keystroke="ctrl c" />
    </action>
    <action id="$Redo">
        <keyboard-shortcut first-keystroke="shift ctrl z" />
    </action>
    <action id=":cursive.repl.actions/jump-to-repl">
        <keyboard-shortcut first-keystroke="ctrl 2" />
    </action>
    <action id=":cursive.repl.actions/run-last-sexp">
        <keyboard-shortcut first-keystroke="ctrl 3" />
    </action>
    <action id=":cursive.repl.actions/sync-files">
        <keyboard-shortcut first-keystroke="shift ctrl r" />
    </action>
    <action id="ActivateMavenProjectsToolWindow">
        <keyboard-shortcut first-keystroke="f2" />
    </action>
    <action id="Build">
        <keyboard-shortcut first-keystroke="ctrl f9" />
    </action>
    <action id="BuildProject">
        <keyboard-shortcut first-keystroke="ctrl b" />
    </action>
    <action id="ChooseDebugConfiguration">
        <keyboard-shortcut first-keystroke="alt d" />
    </action>
    <action id="ChooseRunConfiguration">
        <keyboard-shortcut first-keystroke="alt r" />
    </action>
    <action id="CloseActiveTab" />
    <action id="CloseContent">
        <keyboard-shortcut first-keystroke="ctrl w" />
    </action>
    <action id="CollapseAll">
        <keyboard-shortcut first-keystroke="ctrl subtract" />
    </action>
    <action id="CollapseAllRegions">
        <keyboard-shortcut first-keystroke="shift ctrl divide" />
        <keyboard-shortcut first-keystroke="ctrl minus" />
    </action>
</keymap>"#;
        let keymap = parse_jetbrains_keymap(input).unwrap();
        println!("{}", keymap);
    }

    #[test]
    fn parse_user_keymap_file() -> std::io::Result<()> {
        let mut input = String::new();
        let mut f = std::fs::File::open("./test/Eclipse copy.xml")?;
        f.read_to_string(&mut input)?;

        let keymap = parse_jetbrains_keymap(&input).unwrap();
        println!("{}", keymap);

        Ok(())
    }

    #[test]
    fn read_keymap_from_eclipse_plugin_jar() -> std::io::Result<()> {
        let mut input = String::new();
        let f = std::fs::File::open("./test/keymap-eclipse.jar")?;

        let mut archive = ZipArchive::new(f)?;
        archive
            .by_name("keymaps/Eclipse.xml")
            .map(|mut f| f.read_to_string(&mut input))?
            .expect("parsable xml");

        let keymap = parse_jetbrains_keymap(&input).unwrap();
        println!("{}", keymap);

        Ok(())
    }

    #[test]
    fn read_keymap_from_vs_plugin_jar() -> std::io::Result<()> {
        let mut input = String::new();
        let f = std::fs::File::open("./test/keymap-visualStudio.jar")?;

        let mut archive = ZipArchive::new(f)?;
        archive
            .by_name("keymaps/Visual Studio.xml")
            .map(|mut f| f.read_to_string(&mut input))?
            .expect("parsable xml");

        let keymap = parse_jetbrains_keymap(&input).unwrap();
        println!("{}", keymap);

        Ok(())
    }

    #[test]
    fn read_keymap_from_default() -> std::io::Result<()> {
        // let keymap = parse_jetbrains_keymap(&input).unwrap();
        let keymap = PathBuf::from("./test/default.xml")
            .parse_jetbrains_keymap();

        println!("{}", keymap);

        Ok(())
    }
}

impl JetbrainsKeymapSource for &str {
    fn parse_jetbrains_keymap(&self) -> KeyMap {
        parse_jetbrains_keymap(self)
            .expect("valid keymap")
    }
}

impl JetbrainsKeymapSource for PathBuf {
    fn parse_jetbrains_keymap(&self) -> KeyMap {
        let mut input = String::new();
        let mut f = std::fs::File::open(self)
            .expect("file to be present");
        f.read_to_string(&mut input)
            .expect("parsable xml");

        parse_jetbrains_keymap(&input)
            .expect("valid keymap")
    }
}

pub trait JetbrainsKeymapSource {
    fn parse_jetbrains_keymap(&self) -> KeyMap;
}