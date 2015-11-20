#![feature(slice_patterns, convert)]
use std::collections::BTreeMap;

mod ion;
mod parser;
mod validator;
mod writer;
pub use parser::{ Parser, ParserError };
pub use validator::{ Validator, ValidationError };
pub use writer::Writer;

pub type Dictionary = BTreeMap<String, Value>;
pub use ion::{ Ion, IonError, FromIon, Section, Value };
pub type Row = Vec<Value>;

