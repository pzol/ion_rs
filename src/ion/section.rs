use std::vec;
use {Dictionary, FromIon, IonError, Row, Value};

#[derive(Debug, PartialEq)]
pub struct Section {
    pub dictionary: Dictionary,
    pub rows: Vec<Row>,
}

impl Section {
    pub fn new() -> Section {
        Self::with_capacity(1)
    }

    pub fn with_capacity(n: usize) -> Section {
        Section {
            dictionary: Dictionary::new(),
            rows: Vec::with_capacity(n),
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
            if row.first().map_or(false, |v| match v {
                Value::String(s) => !s.is_empty() && s.chars().all(|c| c == '-' ),
                _                => false
            }) {
                return &self.rows[2..];
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
            iter: self
                .rows_without_header()
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
        let has_header = self
            .rows
            .iter()
            .skip(1)
            .take(1)
            .take_while(|&v| {
                if let Some(Value::String(ref s)) = v.iter().skip(1).next() {
                    s.starts_with("-")
                } else {
                    false
                }
            })
            .next()
            .is_some();

        if has_header {
            IntoIter {
                iter: self
                    .rows
                    .into_iter()
                    .skip(2)
                    .collect::<Vec<_>>()
                    .into_iter(),
            }
        } else {
            IntoIter {
                iter: self.rows.into_iter(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use regex::Regex;
    use Ion;

    fn is_input_string_invalid(s: &str) -> bool {
        // ion cell is invalid if it contains any of [\n \t|\r] or is entirely made out of hyphens
        let disallowed_cell_contents: Regex = Regex::new("[\n \t\r|]|^-+$").expect("regex");

        disallowed_cell_contents.is_match(s)
    }

    #[quickcheck]
    fn when_headers_absent(item: String) -> TestResult {
        if is_input_string_invalid(item.as_str()) {
            return TestResult::discard();
        }

        let ion_str = format!(
            r#"
            [FOO]
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
        "#,
            item = item
        );

        let ion = ion_str.parse::<Ion>().expect("Format ion");

        let section = ion.get("FOO").expect("Get section");

        TestResult::from_bool(3 == section.rows_without_header().len())
    }

    #[quickcheck]
    fn when_headers_present(item: String) -> TestResult {
        if is_input_string_invalid(item.as_str()) {
            return TestResult::discard();
        }

        let ion_str = format!(
            r#"
            [FOO]
            |head1|head2|head3|
            |-----|-----|-----|
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
            |{item}|{item}|{item}|
        "#,
            item = item
        );

        let ion = ion_str.parse::<Ion>().expect("Format ion");

        let section = ion.get("FOO").expect("Get section");

        TestResult::from_bool(3 == section.rows_without_header().len())
    }
}
