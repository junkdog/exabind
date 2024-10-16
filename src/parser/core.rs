use anpa::combinators::{right, succeed};
use anpa::core::StrParser;
use anpa::parsers::item_while;

pub(super) fn eat<'a, O>(p: impl StrParser<'a, O>) -> impl StrParser<'a, O> {
    right(succeed(item_while(|c: char| c.is_whitespace())), p)
}