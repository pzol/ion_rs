#![feature(slice_patterns)]
use std::collections::BTreeMap;

#[macro_use] mod ion;
mod parser;
mod writer;
pub use parser::{ Parser, ParserError };
pub use writer::Writer;

pub type Dictionary = BTreeMap<String, Value>;
pub use ion::{ Ion, IonError, FromIon, Section, Value };
pub type Row = Vec<Value>;
