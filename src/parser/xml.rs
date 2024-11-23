use crate::parser::core::eat;
use anpa::combinators::{left, many, many_to_vec, middle, no_separator, right, succeed};
use anpa::core::{ParserExt, StrParser};
use anpa::parsers::{item_if, item_while, skip, until};
use anpa::{create_parser, defer_parser, left, or, right, skip, tuplify, variadic};
use anpa::whitespace::ascii_whitespace;

#[derive(Debug, PartialEq)]
pub(super) struct XmlTag<'a> {
    name: &'a str,
    attributes: Vec<Attribute<'a>>,
    content: Option<NodeContent<'a>>,
}

#[derive(Debug, PartialEq)]
struct Attribute<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Debug, PartialEq)]
enum NodeContent<'a> {
    Tags(Vec<XmlTag<'a>>),
    Text(&'a str),
}

impl XmlTag<'_> {
    fn with_text<'a>(
        name: &'a str,
        attributes: Vec<Attribute<'a>>,
        text: &'a str,
    ) -> XmlTag<'a> {
        XmlTag { name, attributes, content: Some(NodeContent::Text(text)) }
    }

    fn new<'a>(
        name: &'a str,
        attributes: Vec<Attribute<'a>>,
    ) -> XmlTag<'a> {
        XmlTag { name, attributes, content: None }
    }
}

impl XmlTag<'_> {
    pub(super) fn name(&self) -> &str {
        self.name
    }

    pub(super) fn children(&self) -> &[XmlTag<'_>] {
        static EMPTY: [XmlTag; 0] = [];
        match &self.content {
            Some(NodeContent::Tags(children)) => children,
            _                                 => &EMPTY,
        }
    }

    pub(super) fn text(&self) -> Option<&str> {
        match &self.content {
            Some(NodeContent::Text(text)) => Some(text),
            _ => None,
        }
    }

    pub(super) fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.iter().find(|a| a.name == name).map(|a| a.value)
    }
}

impl<'a> Attribute<'a> {
    fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }
}

fn comment<'a>() -> impl StrParser<'a, &'a str> {
    eat(right(skip("<!--"), until("-->")))
}

fn attribute_value<'a>() -> impl StrParser<'a, &'a str> {
    right(skip!('"'), until("\""))
}

fn name_parser<'a>() -> impl StrParser<'a, &'a str> {
    item_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
}

fn cdata<'a>() -> impl StrParser<'a, &'a str> {
    let valid_char = item_if(|c: char| c != ']');
    middle(skip("<![CDATA["), many(valid_char, true, no_separator()), skip("]]>"))
        .map(|s: &str| s.trim())
}

fn attribute<'a>() -> impl StrParser<'a, Attribute<'a>> {
    tuplify!(
        left(eat(name_parser()), eat(skip("="))),
        eat(attribute_value()),
    ).map(|(key, value)| Attribute::new(key.trim(), value))
}

fn xml_tag<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    let attributes = many_to_vec(attribute(), true, no_separator());
    let child_tags = many_to_vec(xml_parser(), false, no_separator());
    let closing_tag = eat(right!(succeed(skip('<')), skip('/'), name_parser(), eat(skip(">")), eat(ascii_whitespace())));
    let comments = many_to_vec(comment(), true, no_separator());
    let text = eat(item_while(|c: char| c != '<'));

    tuplify!(
        right!(comments, eat(skip('<')), name_parser()), // (discard comments) <tag
        attributes,                                      // key="value"...>
        eat(or!(
            // recurse children
            middle(left!(skip('>'), comments), child_tags, closing_tag)
                .map(|children| Some(NodeContent::Tags(children))),

            // or; self-contained tag <.../>
            skip("/>").map(|_| None),

            // or; empty tag <...></tag>
            left(skip('>'), closing_tag)
                .map(|_| None),

            // or; *text*</tag>
           middle(skip('>'), text, closing_tag)
                .map(|text| Some(NodeContent::Text(text.trim()))),
        )
    )).map(|(name, attributes, content)| XmlTag { name, attributes, content })
}

pub(super) fn xml_parser<'a>() -> impl StrParser<'a, XmlTag<'a>> {
    defer_parser!(eat(xml_tag()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use anpa::core::parse;
    use crate::parser::xml::NodeContent::Tags;

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

    #[test]
    fn parse_self_contained_xml_tag() {
        let p = xml_parser();
        let result = parse(p, "<tag></tag>");
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("tag", vec![])));

        let p = xml_parser();
        let result = parse(p, "<tag/>");
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("tag", vec![])));

        let p = xml_parser();
        let result = parse(p, "<tag key=\"value\"/>");
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("tag", vec![Attribute::new("key", "value")])));

        let input = r#"<keymap version="1" name="Eclipse copy" parent="Eclipse"/>"#;
        let result = parse(p, input);

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("keymap", vec![
            Attribute::new("version", "1"),
            Attribute::new("name", "Eclipse copy"),
            Attribute::new("parent", "Eclipse")]))
        );
    }

    #[test]
    fn parse_tag_with_text() {
        let p = xml_parser();
        let result = parse(p, "<tag key=\"value\">This is a text</tag>");

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::with_text("tag", vec![Attribute::new("key", "value")], "This is a text")));

        let result = parse(p, r#"<tag>
            text
        </tag>"#);

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::with_text("tag", Vec::new(), "text")));
    }

    #[test]
    fn parse_tag_with_children() {
        let result = parse(xml_parser(), "<child key=\"value\">\n        </child>");
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("child", vec![Attribute::new("key", "value")])));

        let p = xml_parser();
        let result = parse(p, r#"<tag> <child/></tag>"#);

        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag {
            name: "tag",
            attributes: Vec::new(),
            content: Some(Tags(vec![XmlTag { name: "child", attributes: vec![], content: None }]))
        }));
    }

    #[test]
    fn test_jetbrains_xml_parser() {
        let p = xml_parser();

        let result = parse(p, r#"<keymap ></keymap>"#);
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("keymap", vec![])));

        let result = parse(p, r#"<keymap version="1" name="Eclipse copy" parent="Eclipse"></keymap>"#);
        assert_eq!(result.state, "");
        assert_eq!(result.result, Some(XmlTag::new("keymap", vec![Attribute::new("version", "1"), Attribute::new("name", "Eclipse copy"), Attribute::new("parent", "Eclipse")])));
    }
}