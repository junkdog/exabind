use anpa::combinators::{attempt, left, many, many_to_vec, middle, no_separator, right, succeed};
use anpa::core::{ParserExt, ParserInto, StrParser};
use anpa::parsers::{item_if, item_while, seq, until_seq};
use anpa::{create_parser, defer_parser, item, left, or, or_diff, right, tuplify, variadic};


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




// fn jetbrains_xml_parser<'a>() -> impl StrParser<'a, XmlTag<'a>> {
//     xml_tag_parser()
// }

