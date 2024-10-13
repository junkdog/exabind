use anpa::core::{ParserExt, ParserInto, StrParser};
use anpa::parsers::{item_if, item_while, seq, until_seq};
use anpa::combinators::{get_parsed, left, many, middle, no_separator};
use anpa::{create_parser, item, right, tuplify, variadic};
use crate::parser::jetbrains::low_level::{attribute_value, eat};

#[derive(Debug, PartialEq)]
struct Attribute<'a> {
    key: &'a str,
    value: &'a str,
}

impl<'a> Attribute<'a> {
    fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, PartialEq)]
enum Xml<'a> {
    Element,
    Attribute(Attribute<'a>),
    Text,
    Comment(&'a str),
    OpenXmlTag(&'a str, Vec<Attribute<'a>>),
    CloseXmlTag(&'a str),
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

struct Element<'a> {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Xml<'a>>,
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

    pub(super) fn attribute_value<'a>() -> impl StrParser<'a, &'a str> {
        let valid_char = item_if(|c: char| c != '"');
        middle(item!('"'), many(valid_char, true, no_separator()), item!('"')).into_type()
    }

    pub(super) fn cdata<'a>() -> impl StrParser<'a, &'a str> {
        let valid_char = item_if(|c: char| c != ']');
        middle(seq("<![CDATA["), many(valid_char, true, no_separator()), seq("]]>"))
            .into_type()
            .map(|s: &str| s.trim())
    }

    pub(super) fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
        right(succeed(item_while(|c: char| c.is_whitespace())), p)
    }
}

fn comment<'a>() -> impl StrParser<'a, Xml<'a>> {
    right!(seq("<!--"), until_seq("-->"))
        .into_type()
        .map(move |s: &str| Xml::Comment(s.trim()))
}

fn attribute<'a>() -> impl StrParser<'a, Xml<'a>> {
    let name = item_while(|c: char| c.is_alphabetic() || c.is_numeric() || c == '_');
    let key_value_parser = tuplify!(
        left(eat(name), eat(item!('='))),
        eat(attribute_value()),
    );

    key_value_parser.map(|(key, value)| Xml::Attribute(Attribute::new(key, value)))
}

mod tests {
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
            assert_eq!(result.result, Some(Xml::Comment("This is a comment")));
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
            assert_eq!(result.result, Some("This is a CDATA"));
        });
    }

    #[test]
    fn parse_attribute_value() {
        let p = low_level::attribute_value();
        let result = parse(p, r#""This is a value" "#);

        assert_eq!(result.state, " ");
        assert_eq!(result.result, Some("This is a value"));
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
            assert_eq!(result.result, Some(Xml::Attribute(Attribute::new("name", "value"))));
        });
    }
}