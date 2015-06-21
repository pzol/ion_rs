#![feature(collections)]
use std::collections::BTreeMap;
use std::str::FromStr;

mod display;
mod parser;
mod validator;
pub use parser::{ Parser, ParserError };
pub use validator::{ Validator, ValidationError };

pub type Dictionary = BTreeMap<String, Value>;
pub type Row = Vec<Value>;

#[derive(Debug)]
pub struct Ion {
    sections: BTreeMap<String, Section>
}

impl Ion {
    pub fn new(map: BTreeMap<String, Section>) -> Ion {
        Ion { sections: map }
    }

    pub fn get(&self, key: &str) -> Option<&Section> {
        self.sections.get(key)
    }

    pub fn iter(&self) -> ::std::collections::btree_map::Iter<String, Section> {
        self.sections.iter()
    }
}

impl FromStr for Ion {
    type Err = Vec<parser::ParserError>;

    fn from_str(s: &str) -> Result<Ion, Vec<parser::ParserError>> {
        let mut p = Parser::new(s);
        match p.read() {
            Some(ion) => Ok(Ion::new(ion)),
            None      => Err(p.errors)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub dictionary: Dictionary,
    pub rows: Vec<Row>
}

impl Section {
    pub fn new() -> Section {
        Section { dictionary: Dictionary::new(), rows: Vec::new() }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    // Float(f64),
    Boolean(bool),
    // Datetime(String),
    Array(Row),
    Dictionary(Dictionary)
}

impl Value {
    pub fn type_str(&self) -> &'static str {
        match *self {
            Value::String(..)     => "string",
            Value::Integer(..)    => "integer",
            Value::Boolean(..)    => "boolean",
            Value::Array(..)      => "array",
            Value::Dictionary(..) => "dictionary"
        }
    }

    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        match *self {
            Value::Dictionary(ref v) => Some(v),
            _ => None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match *self {
            Value::String(ref v) => Some(v),
            _ => None
        }
    }
}

