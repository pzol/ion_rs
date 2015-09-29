mod display;
mod section;
mod value;

use std::{ error, str };
use std::collections::BTreeMap;
use { Parser, ParserError };
pub use self::section::Section;
pub use self::value::Value;

#[macro_export]
macro_rules! ion {
    ($raw:expr) => ({
        use ::ion::Ion;
        let ion : Ion = $raw.parse().unwrap();
        ion
    })
}

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

#[derive(Debug)]
pub struct Error {
    errors: Vec<ParserError>
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Error parsing ion"
    }
}

impl str::FromStr for Ion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Ion, Error> {
        let mut p = Parser::new(s);
        match p.read() {
            Some(ion) => Ok(Ion::new(ion)),
            None      => Err(Error { errors: p.errors })
        }
    }
}

#[cfg(test)]
mod tests {
    use { Value };

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
        let ion = ion!(r#"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
        "#);

        let rows = ion.get("FOO").unwrap().rows_without_header();
        assert!(rows.len() == 3);
    }

    #[test]
    fn row_with_header() {
        let ion = ion!(r#"
            [FOO]
            | 1 | 2 | 3 |
            |---|---|---|
            |1||2|
            |1|   |2|
        "#);

        let rows = ion.get("FOO").unwrap().rows_without_header();
        assert!(rows.len() == 2);
    }
}
