use anpa::core::{ParserExt, ParserInto, StrParser};
use anpa::parsers::{item_if, item_while, seq, until_seq};
use anpa::combinators::{get_parsed, left, many, many_to_vec, middle, no_separator, right, separator};
use anpa::{create_parser, defer_parser, item, left, or, right, tuplify, variadic};
use crossterm::event::KeyCode;
use crate::parser::jetbrains::low_level_parser::{attribute, attribute_value, eat};

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
struct XmlTag<'a> {
    name: &'a str,
    attributes: Vec<Attribute<'a>>,
    children: Vec<XmlTag<'a>>,
    value: Option<&'a str>
}

impl XmlTag<'_> {
    fn new<'a>(
        name: &'a str,
        attributes: Vec<Attribute<'a>>,
        children: Vec<XmlTag<'a>>,
        value: Option<&'a str>
    ) -> XmlTag<'a> {
        XmlTag { name, attributes, children, value }
    }
}

#[derive(Debug, PartialEq)]
enum TagType {
    Open,
    SelfContained,
}

#[derive(Debug, PartialEq)]
enum Xml<'a> {
    Element(XmlTag<'a>),
    Attribute(Attribute<'a>),
    Comment(&'a str),
    OpenXmlTag(&'a str, Vec<Attribute<'a>>, TagType),
    CloseXmlTag(&'a str),
    SelfContainedXmlTag(&'a str, Vec<Attribute<'a>>),
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


mod low_level_parser {
    use anpa::combinators::{many, middle, no_separator, right, succeed};
    use anpa::parsers::{item_if, item_while};
    use anpa::item;
    use super::*;

    fn whitespace<'a>() -> impl StrParser<'a, ()> {
        item_while(|c: char| c.is_whitespace()).map(|_| ())
    }

    pub(super) fn attribute_value<'a>() -> impl StrParser<'a, &'a str> {
        let valid_char = item_if(|c: char| c != '"');
        middle(item!('"'), many(valid_char, true, no_separator()), item!('"'))
    }

    pub(super) fn cdata<'a>() -> impl StrParser<'a, &'a str> {
        let valid_char = item_if(|c: char| c != ']');
        middle(seq("<![CDATA["), many(valid_char, true, no_separator()), seq("]]>"))
            .map(|s: &str| s.trim())
    }

    pub(super) fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
        right(succeed(item_while(|c: char| c.is_whitespace())), p)
    }

    pub(super) fn attribute<'a>() -> impl StrParser<'a, Attribute<'a>> {
        let name = item_while(|c: char| c.is_alphabetic() || c.is_numeric() || "_-".contains(c));

        tuplify!(
            left(eat(name), eat(item!('='))),
            eat(attribute_value()),
        ).map(|(key, value)| Attribute::new(key, value))
    }
}

fn comment<'a>() -> impl StrParser<'a, Xml<'a>> {
    right!(seq("<!--"), until_seq("-->"))
        .map(|s: &str| Xml::Comment(s.trim()))
}

// fn tag_open<'a>() -> impl StrParser<'a, Xml<'a>> {
//     let name = item_while(|c: char| c != ' ' && c != '/');
//     let attributes = many_to_vec(attribute(), true, no_separator());
//
//     let end_tag = or!(
//         eat(seq(">")),
//         eat(seq("/>")),
//     ).map(|s: &str| {
//         match s {
//             ">" => TagType::Open,
//             "/>" => TagType::SelfContained,
//             _ => unreachable!()
//         }
//     });
//
//     tuplify!(
//         right(eat(item!('<')), name),
//         attributes,
//         end_tag
//     ).map(|(name, attributes, tag_type)| Xml::OpenXmlTag(name, attributes, tag_type))
// }

fn xml_tag_self_contained<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    let name = item_while(|c: char| !" />".contains(c));
    let attributes = many_to_vec(attribute(), true, no_separator());

    tuplify!(
        right(item!('<'), name),
        left(attributes, eat(seq("/>")))
    ).map(|(name, attributes)| XmlTag::new(name, attributes, Vec::new(), None))
}

fn xml_tag_open<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    let name = item_while(|c: char| !" >".contains(c));
    let attributes = many_to_vec(attribute(), true, no_separator());
    let children = many_to_vec(jetbrains_xml_parser(), true, no_separator());

    tuplify!(
        right(item!('<'), name),
        left!(attributes, eat(item!('>'))),
        left!(children, eat(seq("</")), name, eat(seq(">"))),
    ).map(|(name, attributes, children)| XmlTag::new(name, attributes, children, None))
}

fn jetbrains_xml_parser<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    defer_parser! {
        eat(or!(
            xml_tag_open(),
            xml_tag_self_contained(),
        ))
    }
}

mod tests {
    use anpa::core::parse;
    use crate::parser::jetbrains::low_level_parser::cdata;
    use crate::parser::jetbrains::{comment};
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
            let p = cdata();
            let result = parse(p, input);

            assert_eq!(result.state, "");
            assert_eq!(result.result, Some("This is a CDATA"));
        });
    }

    #[test]
    fn parse_attribute_value() {
        let p = attribute_value();
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
            assert_eq!(result.result, Some(Attribute::new("name", "value")));
        });
    }

    // #[test]
    // fn parse_open_tag() {
    //     let p = tag_open();
    //     let result = parse(p, "<tag key=\"value\">");
    //
    //     assert_eq!(result.result, Some(Xml::OpenXmlTag("tag", vec![Attribute::new("key", "value")], TagType::Open)));
    //     assert_eq!(result.state, "");
    //
    //     let result = parse(p, "<tag key=\"value\" key2=\"value2\">");
    //
    //     assert_eq!(result.state, "");
    //     assert_eq!(result.result, Some(Xml::OpenXmlTag("tag", vec![Attribute::new("key", "value"), Attribute::new("key2", "value2")], TagType::Open)));
    //
    //     let result = parse(p, "<tag key=\"value\" key2=\"value2\" key3=\"value3\"/>");
    //
    //     assert_eq!(result.state, "");
    //     assert_eq!(result.result, Some(Xml::OpenXmlTag("tag", vec![Attribute::new("key", "value"), Attribute::new("key2", "value2"), Attribute::new("key3", "value3")], TagType::SelfContained)));
    // }

    #[test]
    fn parse_self_contained_xml_tag() {
        let p = jetbrains_xml_parser();
        let result = parse(p, "<tag key=\"value\"/>");

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("tag", vec![Attribute::new("key", "value")], Vec::new(), None)));

        let input = r#"<keymap version="1" name="Eclipse copy" parent="Eclipse"/>"#;
        let result = parse(p, input);

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("keymap", vec![Attribute::new("version", "1"), Attribute::new("name", "Eclipse copy"), Attribute::new("parent", "Eclipse")], Vec::new(), None)));
    }

    #[test]
    fn test_jetbrains_xml_parser() {
        let p = jetbrains_xml_parser();

        let result = parse(p, r#"<keymap ></keymap>"#);
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("keymap", Vec::new(), Vec::new(), None)));

        let result = parse(p, r#"<keymap version="1" name="Eclipse copy" parent="Eclipse"></keymap>"#);
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("keymap", vec![Attribute::new("version", "1"), Attribute::new("name", "Eclipse copy"), Attribute::new("parent", "Eclipse")], Vec::new(), None)));


        let result = parse(p, r#"<keymap version="1" name="Eclipse copy" parent="Eclipse">
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
</keymap>"#);

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("tag", vec![Attribute::new("key", "value")], vec![XmlTag::new("tag2", vec![Attribute::new("key2", "value2")], Vec::new(), None)], None)));
    }
}