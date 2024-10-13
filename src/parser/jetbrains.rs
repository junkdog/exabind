use anpa::core::{ParserExt, ParserInto, StrParser};
use anpa::parsers::{item_if, item_while, seq, until_seq};
use anpa::combinators::{get_parsed, left, many, middle, no_separator};
use anpa::{create_parser, item, right, tuplify, variadic};
use crate::parser::jetbrains::low_level::{attribute_value, eat};

#[derive(Debug, PartialEq)]
enum Xml {
    Element,
    Attribute(String, String),
    Text,
    Comment(String),
}

// example:
// <keymap version="1" name="Eclipse copy" parent="Eclipse">
//     <action id="$Copy">
//         <keyboard-shortcut first-keystroke="ctrl c" />
//     </action>
//     <action id="$Redo">
//         <keyboard-shortcut first-keystroke="shift ctrl z" />
//     </action>
//     <action id=":cursive.repl.actions/jump-to-repl">
//         <keyboard-shortcut first-keystroke="ctrl 2" />
//     </action>
//     <action id=":cursive.repl.actions/run-last-sexp">
//         <keyboard-shortcut first-keystroke="ctrl 3" />
//     </action>
//     <action id=":cursive.repl.actions/sync-files">
//         <keyboard-shortcut first-keystroke="shift ctrl r" />
//     </action>
//     <action id="ActivateMavenProjectsToolWindow">
//         <keyboard-shortcut first-keystroke="f2" />
//     </action>
//     <action id="Build">
//         <keyboard-shortcut first-keystroke="ctrl f9" />
//     </action>
//     <action id="BuildProject">
//         <keyboard-shortcut first-keystroke="ctrl b" />
//     </action>
//     <action id="ChooseDebugConfiguration">
//         <keyboard-shortcut first-keystroke="alt d" />
//     </action>
//     <action id="ChooseRunConfiguration">
//         <keyboard-shortcut first-keystroke="alt r" />
//     </action>
//     <action id="CloseActiveTab" />
//     <action id="CloseContent">
//         <keyboard-shortcut first-keystroke="ctrl w" />
//     </action>
//     <action id="CollapseAll">
//         <keyboard-shortcut first-keystroke="ctrl subtract" />
//     </action>
//     <action id="CollapseAllRegions">
//         <!-- this is a comment -->
//         <keyboard-shortcut first-keystroke="shift ctrl divide" />
//         <keyboard-shortcut first-keystroke="<![CDATA[ctrl minus]]>" />
//     </action>
// </keymap>

struct KeyMap {
    version: String,
    name: String,
    parent: String,
    actions: Vec<Action>,
}

struct Action {
    id: String,
    shortcuts: Vec<Shortcut>,
}

struct Shortcut {
    first_keystroke: String,
}

struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Xml>,
}


mod low_level {
    use anpa::combinators::{many, middle, no_separator, right, succeed};
    use anpa::parsers::{item_if, item_while};
    use anpa::core::ParserInto;
    use anpa::item;
    use super::*;

    fn whitespace<'a>() -> impl StrParser<'a, ()> {
        item_while(|c: char| c.is_whitespace()).map(|_| ())
    }

    pub(super) fn attribute_value<'a>() -> impl StrParser<'a, String> {
        let valid_char = item_if(|c: char| c != '"');
        middle(item!('"'), many(valid_char, true, no_separator()), item!('"')).into_type()
    }

    pub(super) fn cdata<'a>() -> impl StrParser<'a, String> {
        let valid_char = item_if(|c: char| c != ']');
        middle(seq("<![CDATA["), many(valid_char, true, no_separator()), seq("]]>"))
            .into_type()
            .map(|s: &str| s.trim().to_string())
    }

    pub(super) fn eat<'a>(p: impl StrParser<'a, String>) -> impl StrParser<'a, String> {
        right(succeed(item_while(|c: char| c.is_whitespace())), p)
    }
}

fn comment<'a>() -> impl StrParser<'a, Xml> {
    right!(seq("<!--"), until_seq("-->"))
        .into_type()
        .map(|s: &str| Xml::Comment(s.trim().to_string()))
}

fn attribute<'a>() -> impl StrParser<'a, Xml> {
    let name = item_while(|c: char| c.is_alphabetic() || c.is_numeric() || c == '_')
        .map(String::from);
    let key_value_parser = tuplify!(
        left(eat(name), eat(item!('=').map(String::from))),
        eat(attribute_value()),
    );

    key_value_parser.map(|(key, value)| Xml::Attribute(key, value))
}

// fn start_tag<'a>() -> impl StrParser<'a, Xml> {
//
// }

mod tests {
    use std::result;
    use anpa::core::parse;
    use super::*;

    #[test]
    fn test_comment() {
        [
            "<!-- This is a comment -->",
            "<!--This is a comment-->",
            r#"<!--

                This is a comment

            -->"#,
        ].iter().for_each(|input| {
            let p = comment();
            let result = parse(p, input);

            assert_eq!(result.state, "");
            assert_eq!(result.result, Some(Xml::Comment("This is a comment".to_string())));
        });
    }

    #[test]
    fn test_cdata() {
        [
            "<![CDATA[This is a CDATA]]>",
            r#"<![CDATA[

                This is a CDATA

            ]]>"#,
        ].iter().for_each(|input| {
            let p = low_level::cdata();
            let result = parse(p, input);

            assert_eq!(result.state, "");
            assert_eq!(result.result, Some("This is a CDATA".to_string()));
        });
    }

    #[test]
    fn parse_attribute_value() {
        let p = low_level::attribute_value();
        let result = parse(p, r#""This is a value" "#);

        assert_eq!(result.state, " ");
        assert_eq!(result.result, Some("This is a value".to_string()));
    }

    #[test]
    fn parse_attribute() {
        [
            r#"name="value" "#,
            r#"name =  "value" "#,
        ].iter().for_each(|s|{
            let p = attribute();
            let result = parse(p, s);
            assert_eq!(result.state, " ");
            assert_eq!(result.result, Some(Xml::Attribute("name".to_string(), "value".to_string())));
        });

    }
}