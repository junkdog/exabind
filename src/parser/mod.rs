pub mod jetbrains;
pub mod kde;
mod xml;
mod core;

pub(self) use xml::xml_parser;
pub(self) use core::eat;