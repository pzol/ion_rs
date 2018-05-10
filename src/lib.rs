#![feature(slice_patterns)]
use std::collections::BTreeMap;

macro_rules! parse_next {
    ($row:expr, $err:expr) => ({
        match $row.next() {
            Some(v) => match v.parse() {
                Ok(v) => v,
                Err(_) => return Err($err)
            },
            None => return Err($err)
        }
    })
}

#[macro_use] mod ion;
mod parser;
mod writer;
pub use parser::{ Parser, ParserError };
pub use writer::Writer;

pub type Dictionary = BTreeMap<String, Value>;
pub use ion::{ Ion, IonError, FromIon, Section, Value };
pub type Row = Vec<Value>;
