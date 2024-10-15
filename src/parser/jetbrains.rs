use crate::parser::jetbrains::low_level_parser::{attribute, attribute_value, eat, name_parser};
use anpa::combinators::{left, many, many_to_vec, middle, no_separator, right, succeed};
use anpa::core::{ParserExt, ParserInto, StrParser};
use anpa::parsers::{item_if, item_while, seq, until_seq};
use anpa::{create_parser, defer_parser, item, left, or, or_diff, right, tuplify, variadic};

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


mod low_level_parser {
    use super::*;

    pub(super) fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
        right(succeed(item_while(|c: char| c.is_whitespace())), p)
    }

    pub(super) fn attribute_value<'a>() -> impl StrParser<'a, &'a str> {
        right(item!('"'), until_seq("\""))
    }

    pub(super) fn name_parser<'a>() -> impl StrParser<'a, &'a str> {
        item_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
    }

    pub(super) fn cdata<'a>() -> impl StrParser<'a, &'a str> {
        let valid_char = item_if(|c: char| c != ']');
        middle(seq("<![CDATA["), many(valid_char, true, no_separator()), seq("]]>"))
            .map(|s: &str| s.trim())
    }

    pub(super) fn attribute<'a>() -> impl StrParser<'a, Attribute<'a>> {
        tuplify!(
            left(eat(name_parser()), eat(until_seq("="))),
            eat(attribute_value()),
        ).map(|(key, value)| Attribute::new(key.trim(), value))
    }
}

fn xml_tag<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    let attributes = many_to_vec(attribute(), true, no_separator());
    let child_tags = many_to_vec(xml_tag_parser(), true, no_separator());

    let closing_tag = right!(succeed(item!('<')), item!('/'), name_parser(), eat(seq(">")));

    tuplify!(
        right(eat(item!('<')), name_parser()), // <tag
        left!(attributes, eat(seq(">"))),      // key="value"...>
        left!(child_tags, closing_tag),        // recurse children
    ).map(|(name, attributes, children)| XmlTag::new(name, attributes, children, None))
}

fn xml_tag_no_children<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    let attributes = many_to_vec(attribute(), true, no_separator());
    let closing_tag = right!(succeed(item!('<')), item!('/'), name_parser(), eat(seq(">")));

    tuplify!(
        right(item!('<'), name_parser()),
        left!(attributes, or!(seq("/>"), closing_tag)),
    ).map(|(name, attributes)| XmlTag::new(name, attributes, Vec::new(), None))
}

fn xml_tag_parser<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    defer_parser! {
        eat(or!(xml_tag_no_children(), xml_tag()))
    }
}

fn jetbrains_xml_parser<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    defer_parser! {
        eat(or!(
            xml_tag_no_children(),
            xml_tag(),
        ))
    }
}

mod tests {
    use super::*;
    use crate::parser::jetbrains::low_level_parser::{cdata, name_parser};
    use anpa::core::parse;

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
    fn parse_name() {
        let p = name_parser();
        let result = parse(p, "hello></hello>");

        assert_eq!(result.state, "></hello>");
        assert_eq!(result.result, Some("hello"));
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

        let result = parse(attribute(), r#" name =  "value"></hello>"#);
        assert_eq!(result.state, "></hello>");
        assert_eq!(result.result, Some(Attribute::new("name", "value")));
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
        let p = xml_tag_parser();
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