use std::vec;
use {Dictionary, FromIon, IonError, Value, Row};

#[derive(Debug, PartialEq)]
pub struct Section {
    pub dictionary: Dictionary,
    pub rows: Vec<Row>,
}

impl Section {
    pub fn new() -> Section {
        Section {
            dictionary: Dictionary::new(),
            rows: Vec::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.dictionary.get(name)
    }

    /// like get, only returns a `Result`
    pub fn fetch(&self, key: &str) -> Result<&Value, IonError> {
        self.get(key).ok_or(IonError::MissingValue(key.to_owned()))
    }

    pub fn rows_without_header(&self) -> &[Row] {
        if self.rows.len() > 1 {
            let row = &self.rows[1];
            if let &[Value::String(ref s), _..] = &row[..1] {
                if s.starts_with("-") {
                    return &self.rows[2..];
                }
            }
        }

        &self.rows
    }

    pub fn parse<F: FromIon<Section>>(&self) -> Result<F, F::Err> {
        F::from_ion(self)
    }
}

pub struct IntoIter<T> {
    iter: vec::IntoIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> IntoIterator for &'a Section {
    type Item = Row;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.rows_without_header()
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}

impl IntoIterator for Section {
    type Item = Row;
    type IntoIter = IntoIter<Row>;
    fn into_iter(self) -> Self::IntoIter {
        let has_header = self.rows
            .iter()
            .skip(1)
            .take(1)
            .take_while(|&v| if let &[Value::String(ref s), _..] = &v[1..] {
                s.starts_with("-")
            } else {
                false
            })
            .next()
            .is_some();

        if has_header {
            IntoIter {
                iter: self.rows
                    .into_iter()
                    .skip(2)
                    .collect::<Vec<_>>()
                    .into_iter(),
            }
        } else {
            IntoIter {
                iter: self.rows
                    .into_iter(),
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use Section;

    #[test]
    fn into_iter_ref_section() {
        let ion = ion!(r#"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
        "#);

        let section: &Section = ion.get("FOO").unwrap();
        let rows: Vec<_> = section.into_iter().collect();
        assert!(rows.len() == 3);
    }

    #[test]
    fn into_iter_section() {
        let mut ion = ion!(r#"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
        "#);

        let section: Section = ion.remove("FOO").unwrap();
        let rows: Vec<_> = section.into_iter().collect();
        assert_eq!(3, rows.len());
    }


    #[test]
    fn into_iter_section_loop() {
        let mut ion = ion!(r#"
            [FOO]
            |1||2|
            |1|   |2|
            |1|2|3|
        "#);

        let section: Section = ion.remove("FOO").unwrap();
        let mut rows = Vec::new();
        for row in section {
            rows.push(row)
        }
        assert_eq!(3, rows.len());
    }

    #[test]
    fn into_iter_section_with_header() {
        let mut ion = ion!(r#"
            [FOO]
            | 1 | 2 | 3 |
            |---|---|---|
            |1||2|
            |1|   |2|
            |1|2|3|
        "#);

        let section: Section = ion.remove("FOO").unwrap();
        let rows: Vec<_> = section.into_iter().collect();
        assert_eq!(3, rows.len());
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