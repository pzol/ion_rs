use { Dictionary, Value, Row };

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
        if self.rows.len() > 1 {
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
