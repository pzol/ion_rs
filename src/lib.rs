#![feature(collections, slice_patterns, convert)]
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

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.dictionary.get(name)
    }

    pub fn rows_without_header(&self) -> &[Row] {
        if self.rows.len() > 2 {
            let row = &self.rows[1];
            if let [Value::String(ref s), ..] = &row[..1] {
                if s.starts_with("-") {
                    return &self.rows[2..]
                }
            }
        }

        &self.rows
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

    pub fn as_string(&self) -> Option<&String> {
        match *self {
            Value::String(ref v) => Some(v),
            _ => None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref v) => Some(v.as_str()),
            _ => None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match *self {
            Value::Integer(v) => Some(v),
            _ => None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            Value::Boolean(v) => Some(v),
            _ => None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Array(ref v) => Some(v),
            _ => None
        }
    }

    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        match *self {
            Value::Dictionary(ref v) => Some(v),
            _ => None
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        match *self {
            Value::Dictionary(ref v) => v.get(name),
            _ => None
        }
    }

}

#[cfg(test)]
mod tests {
    use { Ion, Value };

    #[test]
    fn as_string() {
        let v = Value::String("foo".into());
        assert_eq!(Some(&"foo".into()), v.as_string());
        let v = Value::Integer(1);
        assert_eq!(None, v.as_string());
    }

    #[test]
    fn as_boolean() {
        let v = Value::Boolean(true);
        assert_eq!(Some(true), v.as_boolean());
        let v = Value::Integer(1);
        assert_eq!(None, v.as_boolean());
    }

    #[test]
    fn as_integer() {
        let v = Value::Integer(1);
        assert_eq!(Some(1), v.as_integer());
        let v = Value::String("foo".into());
        assert_eq!(None, v.as_integer());
    }

    #[test]
    fn as_str() {
        let v = Value::String("foo".into());
        assert_eq!(Some("foo"), v.as_str());
        let v = Value::Integer(1);
        assert_eq!(None, v.as_str());
    }

    #[test]
    fn row_without_header() {
        let raw = r#"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
        "#;

        let ion : Ion = raw.parse().unwrap();
        let rows = ion.get("FOO").unwrap().rows_without_header();
        assert!(rows.len() == 3);
    }

    #[test]
    fn row_with_header() {
        let raw = r#"
            [FOO]
            | 1 | 2 | 3 |
            |---|---|---|
            |1||2|
            |1|   |2|
        "#;

        let ion : Ion = raw.parse().unwrap();
        let rows = ion.get("FOO").unwrap().rows_without_header();
        assert!(rows.len() == 2);
    }
}
