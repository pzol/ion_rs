#[macro_export]
macro_rules! ion {
    ($raw:expr) => ({
        use ::ion::Ion;
        let ion : Ion = $raw.parse().unwrap();
        ion
    })
}

mod decoder;
mod display;
mod from_ion;
mod from_row;
mod ion_error;
mod section;
mod value;

use std::str;
use std::collections::BTreeMap;
use Parser;
pub use self::decoder::{ decode, decode_from_vec, Decoder };
pub use self::ion_error::IonError;
pub use self::section::Section;
pub use self::value::Value;
pub use ion::from_row::FromRow;
pub use ion::from_ion::FromIon;

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

    pub fn fetch(&self, key: &str) -> Result<&Section, IonError> {
        self.get(key).ok_or(IonError::MissingSection(key.to_owned()))
    }

    /// Removes a `Section` from the ion structure and returning it
    pub fn remove(&mut self, key: &str) -> Option<Section> {
        self.sections.remove(key)
    }

    pub fn iter(&self) -> ::std::collections::btree_map::Iter<String, Section> {
        self.sections.iter()
    }
}

impl str::FromStr for Ion {
    type Err = IonError;

    fn from_str(s: &str) -> Result<Ion, IonError> {
        let mut p = Parser::new(s);
        match p.read() {
            Some(ion) => Ok(Ion::new(ion)),
            None      => Err(IonError::ParserErrors(p.errors))
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

    #[test]
    fn no_rows_with_header() {
        let ion = ion!(r#"
            [FOO]
            | 1 | 2 | 3 |
            |---|---|---|
        "#);

        let rows = ion.get("FOO").unwrap().rows_without_header();
        assert_eq!(0, rows.len());
    }
}
