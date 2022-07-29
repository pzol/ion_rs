#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
#[cfg(test)]
extern crate regex;

use std::collections::BTreeMap;

#[macro_use]
mod ion;
mod parser;
mod writer;

pub use self::ion::{FromIon, Ion, IonError, Section, Value};
pub use self::parser::{Parser, ParserError};
pub use self::writer::Writer;

pub type Dictionary = BTreeMap<String, Value>;
pub type Row = Vec<Value>;
