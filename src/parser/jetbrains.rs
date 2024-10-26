use crate::parser::xml::{xml_parser, XmlTag};
use crate::shortcut::{Action, Shortcut};
use anpa::core::{parse, StrParser};
use std::fmt::Display;
use std::io::Read;
use std::path::PathBuf;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct KeyMap {
    version: String,
    name: String,
    parent: Option<String>,
    actions: Vec<Action>,
}



impl KeyMap {
    pub fn valid_actions(&self) -> impl Iterator<Item=(&'static str, &Action)> {
        self.actions.iter()
            .filter(|a| a.is_bound())
            .map(|a| (categorize_action(a), a))
    }
}

impl Display for KeyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = |(category, action): (_, &Action)| format!("\t{} ({})", action.to_string(), category);
        let parent = self.parent.as_ref().map(|s| s.as_str()).unwrap_or("");
        // let actions = self.actions.iter().map(format).collect::<Vec<_>>().join("\n");
        let actions = self.valid_actions().map(format).collect::<Vec<_>>().join("\n");
        write!(f, "keymap name={} parent={}:\n{}", self.name, parent, actions)
    }
}

fn categorize_action(action: &Action) -> &'static str {
    match action.name() {
        n if n.contains("Bookmark")                => "bookmarks",
        n if n.starts_with("Goto")                 => "navigate",
        n if n.starts_with("JumpTo")               => "navigate",
        "Back" | "PreviousTab" | "NextTab"         => "navigate",
        n if n.contains("Hierarchy")               => "hierarchy",
        n if n.starts_with("Introduce")            => "refactor",
        n if n.starts_with("Refactorings.")        => "refactor",
        "ChangeSignature"
        | "Inline"
        | "ImplementMethods"
        | "ExtractMethod"
        | "Move"
        | "RenameElement"                          => "refactor",
        n if n.contains("EditorTab")               => "editor tab",
        n if n.contains("Split")                   => "editor tab",
        "MoveEditorToOppositeTabGroup"             => "editor tab",
        n if n.starts_with("Editor")               => "editor",
        n if n.starts_with("Edit")                 => "edit",
        n if n.starts_with("CommentBy")            => "edit",
        n if n.starts_with("$")                    => "edit",
        "ShowIntentionActions"                     => "edit",
        n if n.starts_with("Find")                 => "find",
        n if n.starts_with("Replace")              => "find",
        n if n.starts_with("SearchEverywhere.")    => "find",
        "NextOccurence"
        | "IncrementalSearch"
        | "ShowUsages"
        | "HighlightUsagesInFile"
        | "PreviousOccurence"                      => "find",
        n if n.starts_with("Activate")             => "tool window",
        n if n.contains("ToolWindow")              => "tool window",
        "HideActiveWindow"
        | "AutoIndentLines"
        | "HideAllWindows"                         => "tool window",
        n if n.starts_with("Build")                => "build",
        n if n.starts_with("Compile")              => "build",
        n if n.starts_with("Close")                => "close",
        n if n.starts_with("Collapse")             => "tree actions",
        n if n.starts_with("Expand")               => "tree actions",
        n if n.starts_with("Breakpoint")           => "breakpoints",
        n if n.starts_with("Debugger.")            => "debug/run",
        n if n.starts_with("XDebugger.")           => "debug/run",
        n if n.starts_with("Step")                 => "debug/run",
        n if n.starts_with("ForceStep")            => "debug/run",
        "RunToCursor"                              => "debug/run",
        n if n.starts_with("Run")                  => "debug/run",
        n if n.starts_with("Rerun")                  => "debug/run",
        "Stop"
        | "Pause"
        | "Debug"
        | "Resume"
        | "SmartStepInto"
        | "ChooseDebugConfiguration"
        | "ChooseRunConfiguration"
        | "XDebugger.AttachToProcess"
        | "EvaluateExpression"                     => "debug/run",
        "ParameterInfo"
        | "ToggleInlineHintsAction"
        | "ToggleInlayHintsGloballyAction"
        | "ExpressionTypeInfo"
        | "EditorContextInfo"
        | "ShowErrorDescription"                   => "code introspection",

        n if n.starts_with("MoveElement")          => "code formatting",
        n if n.starts_with("MoveLine")             => "code formatting",
        n if n.starts_with("MoveStatement")        => "code formatting",
        "ReformatCode"
        | "OptimizeImports"                        => "code formatting",
        n if n.contains("Completion")              => "code completion",
        n if n.starts_with("Terminal.")            => "terminal",
        n if n.starts_with("Diff.")                => "diff",
        "NextDiff" | "PreviousDiff"                => "diff",
        "OpenFile"                                 => "file",
        n if n.starts_with("FileChooser.")         => "file chooser",
        n if n.starts_with("Vcs.")                 => "vcs",
        n if n.starts_with("Git.")                 => "vcs",

        _                                          => "other",
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
    use std::collections::{HashMap, HashSet};
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
        // println!("{}", keymap);
        print_sorted_actions(keymap, true);

        Ok(())
    }

    fn print_sorted_actions(
        keymap: KeyMap,
        also_sort_by_category: bool,
    ) {
        let mut actions = keymap.valid_actions()
            .collect::<Vec<_>>();

        if also_sort_by_category {
            actions.sort_by(|(c1, a1), (c2, a2)| c1.cmp(c2).then(a1.name().cmp(a2.name())));
        } else {
            actions.sort_by(|(_, a1), (_, a2)| a1.name().cmp(a2.name()));
        }

        // count actions per category
        let category_counts:HashMap<&str, usize> = actions.iter()
            .map(|(category, _)| category)
            .fold(HashMap::new(), |mut acc, category| {
                let count = acc.entry(category).or_insert(0);
                *count += 1;
                acc
            });

        println!("Actions per category:");
        category_counts.iter().for_each(|(category, count)| {
            println!("\t{}: {}", category, count);
        });
        println!();

        print!("Actions by category, name:");
        actions.iter().for_each(|(category, action)| {
            println!("\t{} ({})", action.to_string(), category);
        });
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
        print_sorted_actions(keymap, true);
        // println!("{}", keymap);

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
        print_sorted_actions(keymap, true);

        Ok(())
    }

    #[test]
    fn read_keymap_from_default() -> std::io::Result<()> {
        // let keymap = parse_jetbrains_keymap(&input).unwrap();
        let keymap = PathBuf::from("./test/default.xml")
            .parse_jetbrains_keymap();

        // println!("{}", keymap);

        print_sorted_actions(keymap, true);
        // let mut actions = keymap.valid_actions()
        //     .collect::<Vec<_>>();
        //
        // actions.sort_by(|a, b| a.1.name().cmp(b.1.name()));
        //
        // actions.iter().for_each(|(category, action)| {
        //     println!("\t{} ({})", action.to_string(), category);
        // });

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