use std::collections::BTreeMap;
use std::{ error, fmt, str };
use { Section, Value };

macro_rules! try {
    ($e:expr) => (match $e { Some(s) => s, None => return None })
}

#[derive(Debug, PartialEq)]
pub enum Element {
    Section(String),
    Row(Vec<Value>),
    Entry(String, Value),
    Comment(String)
}

pub struct Parser<'a> {
    input: &'a str,
    cur: str::CharIndices<'a>,
    pub errors: Vec<ParserError>,
    accepted_sections: Option<Vec<&'a str>>,
    section_capacity: usize,
    row_capacity: usize,
    array_capacity: usize,
}

macro_rules! some {
    ($expr:expr) => ({
        let ret = $expr;
        if let Some(value) = ret {
            value
        } else {
            return None;
        }
    })
}

impl<'a> Iterator for Parser<'a> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        let mut is_section_accepted = true;
        loop {
            self.ws();
            if self.newline() { continue }

            let c = match self.peek(0) {
                Some((_, c)) => c,
                None => return None,
            };

            if c == '[' {
                let name = self.section_name();
                match self.is_section_accepted(&name) {
                    Some(true) => return Some(Element::Section(name)),
                    Some(false) => is_section_accepted = false,
                    None => return None,
                };
            }
            if !is_section_accepted {
                self.skip_line();
                continue;
            }
            return match c {
                '|' => self.row(),
                '#' => self.comment(),
                _   => self.entry()
            };
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Self::new_filtered_opt(s, None)
    }

    pub fn new_filtered(s: &'a str, accepted_sections: Vec<&'a str>) -> Parser<'a> {
        Self::new_filtered_opt(s, Some(accepted_sections))
    }

    pub fn with_section_capacity(mut self, section_capacity: usize) -> Self {
        self.section_capacity = section_capacity;
        self
    }

    pub fn with_row_capacity(mut self, row_capacity: usize) -> Self {
        self.row_capacity = row_capacity;
        self
    }

    pub fn with_array_capacity(mut self, array_capacity: usize) -> Self {
        self.array_capacity = array_capacity;
        self
    }

    fn new_filtered_opt(s: &'a str, accepted_sections: Option<Vec<&'a str>>) -> Parser<'a> {
        Parser {
            input: s,
            cur: s.char_indices(),
            errors: Vec::new(),
            accepted_sections,
            section_capacity: 16,
            row_capacity: 8,
            array_capacity: 2,
        }

    }

    fn peek(&self, n: usize) -> Option<(usize, char)> {
        self.cur.clone().skip(n).next()
    }

    fn ws(&mut self) -> bool {
        let mut ret = false;
        loop {
            match self.peek(0) {
                Some((_, '\t')) | Some((_, ' ')) => { self.cur.next(); ret = true ;},
                _ => break
            }
        }
        ret
    }

    fn newline(&mut self) -> bool {
        match self.peek(0) {
            Some((_, '\n')) => { self.cur.next(); true },
            Some((_, '\r')) if self.peek(1).map(|c| c.1) == Some('\n') => { self.cur.next(); self.cur.next(); true },
            _ => false
        }
    }

    fn skip_line(&mut self) {
        self.cur.by_ref()
            .skip_while(|&(_, c)| c != '\n')
            .next();
    }

    fn comment(&mut self) -> Option<Element> {
        if !self.eat('#') { return None }

        Some(Element::Comment(self.slice_to_inc('\n').unwrap_or("").to_string()))
    }

    fn eat(&mut self, ch: char) -> bool {
        match self.peek(0) {
            Some((_, c)) if c == ch => { self.cur.next(); true }
            Some(_) | None => false
        }
    }

    fn section_name(&mut self) -> String {
        self.eat('[');
        self.ws();
        self.cur.by_ref()
            .map(|(_, c)| c)
            .take_while(|c| *c != ']')
            .collect()
    }

    fn entry(&mut self) -> Option<Element> {
        let key = some!(self.key_name());
        if !self.keyval_sep() { return None }
        let val = some!(self.value());

        Some(Element::Entry(key, val))
    }

    fn key_name(&mut self) -> Option<String> {
        self.slice_while(|ch| match ch {
                'a' ... 'z' |
                'A' ... 'Z' |
                '0' ... '9' |
                '_' | '-' => true,
                _ => false,
        }).map(str::to_owned)
    }

    fn value(&mut self) -> Option<Value> {
        self.ws();
        self.newline();
        self.ws();

        match self.cur.clone().next() {
            Some((_, '"')) => return self.finish_string(),
            Some((_, '[')) => return self.finish_array(),
            Some((_, '{')) => return self.finish_dictionary(),
            Some((_, ch)) if is_digit(ch) => self.number(),
            Some((pos, 't')) |
            Some((pos, 'f')) => self.boolean(pos),
            _ => {
                let mut it = self.cur.clone();
                let lo = it.next().map(|p| p.0).unwrap_or(self.input.len());
                let hi = it.next().map(|p| p.0).unwrap_or(self.input.len());
                self.errors.push(ParserError{
                    lo: lo, hi: hi,
                    desc: format!("expected a value")
                });

                None
            }
        }
    }

    fn finish_array(&mut self) -> Option<Value> {
        self.cur.next();
        let mut row = Vec::with_capacity(self.array_capacity);

        loop {
            self.ws();
            if let Some((_, ch)) = self.peek(0) {
                match ch {
                    ']' => { self.cur.next(); return Some(Value::Array(row)) },
                    ',' => { self.cur.next(); continue },
                    _ => {
                        match self.value() {
                            Some(v) => row.push(v),
                            None    => break
                        }
                    }
                }
            } else {
                break;
            }
        }
        None
    }

    fn finish_dictionary(&mut self) -> Option<Value> {
        self.cur.next();
        let mut map = BTreeMap::new();

        loop {
            self.ws();
            if let Some((_, ch)) = self.peek(0) {
                match ch {
                    '}' => { self.cur.next(); return Some(Value::Dictionary(map)) },
                    ',' => { self.cur.next(); continue },
                    '\n' => { self.cur.next(); continue },
                    _ => {
                        match self.entry() {
                            Some(Element::Entry(k, v)) => map.insert(k, v),
                            None    => break,
                            _ => panic!("Element::Entry expected")
                        };
                    }
                }
            } else {
                break;
            }
        }
        None
    }

    fn number(&mut self) -> Option<Value> {
        let mut is_float = false;
        let prefix = try!(self.integer());
        let decimal = if self.eat('.') {
            is_float = true;
            Some(try!(self.integer()))
        } else {
            None
        };

        let input = match decimal {
            Some(ref decimal) => prefix + "." + decimal,
            None          => prefix
        };

        if is_float {
            input.parse().ok().map(Value::Float)
        } else {
            input.parse().ok().map(Value::Integer)
        }
    }

    fn integer(&mut self) -> Option<String> {
        self.slice_while(|ch| match ch {
            '0' ... '9' => true,
            _ => false
        }).map(str::to_owned)
    }

    fn boolean(&mut self, start: usize) -> Option<Value> {
        let rest = &self.input[start..];

        if rest.starts_with("true") {
            for _ in 0..4 {
                self.cur.next();
            }
            Some(Value::Boolean(true))
        } else if rest.starts_with("false") {
            for _ in 0..5 {
                self.cur.next();
            }
            Some(Value::Boolean(false))
        } else {
            None
        }
    }

    fn finish_string(&mut self) -> Option<Value> {
        self.cur.next();
        self.slice_to_exc('"').map(|s| Value::String(s.to_string()))
    }

    fn keyval_sep(&mut self) -> bool {
        self.ws();
        if !self.expect('=') { return false }
        self.ws();
        true
    }

    fn expect(&mut self, ch: char) -> bool {
        self.eat(ch)
    }

    fn row(&mut self) -> Option<Element> {
        let mut row  = Vec::with_capacity(self.row_capacity);
        self.eat('|');

        loop {
            self.ws();
            if self.comment().is_some() { break } // this will eat and NOT return comments within tables
            if self.newline() { break }
            if self.peek(0).is_none() { break }

            row.push(Value::String(self.cell()));
        }

        Some(Element::Row(row))
    }

    fn cell(&mut self) -> String {
        self.ws();
        self.slice_to_exc('|').map(str::trim_end).unwrap_or("").to_owned()
    }

    pub fn read(&mut self) -> Option<BTreeMap<String, Section>> {
        let mut map = BTreeMap::new();

        let mut section = Section::with_capacity(self.section_capacity);
        let mut name    = None;

        while let Some(el) = self.next() {
            match el {
                Element::Section(ref n) => {
                    if let Some(name) = name {
                        map.insert(name, section);
                    }
                    name = Some(n.to_owned());
                    section = Section::with_capacity(self.section_capacity);
                },
                Element::Row(row) => section.rows.push(row),
                Element::Entry(ref key, ref value) => { section.dictionary.insert(key.clone(), value.clone()); },
                _ => continue
            };
        }

        match name {
            Some(name) => map.insert(name, section),
            None if self.accepted_sections.is_none() => map.insert("root".to_string(), section),
            _ => None,
        };

        if self.errors.len() > 0 {
            None
        } else {
            Some(map)
        }
    }

    fn is_section_accepted(&mut self, name: &str) -> Option<bool> {
        let sections = match self.accepted_sections {
            Some(ref mut sections) => sections,
            None => return Some(true),
        };
        if sections.is_empty() {
            return None
        }
        match sections.iter().position(|s| *s == name) {
            Some(idx) => {
                sections.swap_remove(idx);
                Some(true)
            },
            None => Some(false)
        }
    }

    // returns slice from the next character to `ch`, inclusive
    // after this function, self.cur.next() returns the next character after `ch`
    // None is only returned if the input is empty
    // Examples:
    // Parser::new("foObar").slice_to_inc('b') == Some("foOb"), self.cur.next() == (4, 'a')
    // Parser::new("foObar").slice_to_inc('f') == Some("f"),    self.cur.next() == (1, 'o')
    fn slice_to_inc(&mut self, ch: char) -> Option<&str> {
        self.cur
            .next()
            .and_then(|(start, c)|
                if c == ch {
                    Some(&self.input[start..=start])
                }
                else {
                    Some(
                        self.cur
                            .find(|(_, c)| *c == ch)
                            .map_or(
                                &self.input[start..],
                                |(end, _)| &self.input[start..=end]
                            )
                    )
                }
            )
    }

    // returns slice from the next character to `ch`
    // the result is exclusive (does not contain `ch`), but the functions consumes `ch`
    // None is returned when the input is empty or when `ch` is the next character
    // Examples:
    // Parser::new("foObar").slice_to_exc('b') == Some("foO"), self.cur.next() == (4, 'a')
    // Parser::new("foObar").slice_to_exc('f') == None,        self.cur.next() == (1, 'o')
    fn slice_to_exc(&mut self, ch: char) -> Option<&str> {
        self.cur
            .next()
            .and_then(|(start, c)|
                if c == ch {
                    Some("")
                } else {
                    Some(
                        self.cur
                            .find(|(_, c)| *c == ch)
                            .map_or(
                                &self.input[start..],
                                |(end, _)| &self.input[start..end]
                            )
                    )
                }
            )
    }

    // returns slice from the next character to the last consecutive character matching the predicate
    // the result is exclusive (does not contain `ch`) and does not consume `ch`
    // None is returned when the input is empty or when `ch` is the next character
    // Examples:
    // Parser::new("foObar").slice_while(|c| c != 'b') == Some("foO"), self.cur.next() == (3, 'b')
    // Parser::new("foObar").slice_while(|c| c != 'f') == None,        self.cur.next() == (0, 'f')
    fn slice_while(&mut self, predicate: impl Fn(char) -> bool) -> Option<&str> {
        self
            .peek(0)
            .and_then(|(start, c)|
                if !predicate(c) {
                    None
                }
                else {
                    self.cur.next();

                    while let Some((end, c)) = self.peek(0) {
                        if !predicate(c) {
                            return Some(&self.input[start..end]);
                        }

                        self.cur.next();
                    }

                    Some(&self.input[start..])
                }
            )
    }
}

fn is_digit(c: char) -> bool {
    match c { '0' ... '9' => true, _ => false }
}

#[derive(Clone, Debug)]
pub struct ParserError {
    /// The low byte at which this error is pointing at.
    pub lo: usize,
    /// One byte beyond the last character at which this error is pointing at.
    pub hi: usize,
    /// A human-readable description explaining what the error is.
    pub desc: String,
}

impl error::Error for ParserError {
    fn description(&self) -> &str {
        "error parsing Ion"
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::Element::{self, Row, Entry, Comment};
    use {Parser, Value, Section};
    use std::collections::BTreeMap;

    #[test]
    fn finish_string() {
        let mut p = Parser::new("\"foObar\"");
        assert_eq!(Some("foObar"), p.finish_string().unwrap().as_str());

        let mut p = Parser::new("\"foObar");
        assert_eq!(Some("foObar"), p.finish_string().unwrap().as_str());

        let mut p = Parser::new("");
        assert_eq!(None, p.finish_string());
    }

    #[test]
    fn finish_array() {
        let mut p = Parser::new("[\"a\"");
        assert_eq!(None, p.finish_array());

        let mut p = Parser::new("[\"a\"]");
        assert_eq!(Some(Value::new_string_array("a")), p.finish_array());
    }

    #[test]
    fn finish_dictionary() {
        let mut p = Parser::new("{ foo = \"bar\"");
        assert_eq!(None, p.finish_dictionary());

        let mut p = Parser::new("{ foo = [\"bar\"");
        assert_eq!(None, p.finish_dictionary());

        let mut p = Parser::new("{ foo = [\"bar\"]");
        assert_eq!(None, p.finish_dictionary());

        let mut p = Parser::new("{ foo = [\"bar\"] }");
        assert_eq!("{ foo = [ \"bar\" ] }", p.finish_dictionary().map(|d| d.to_string()).unwrap());
    }

    #[test]
    fn slice_to_inc() {
        let mut p = Parser::new("foObar");
        assert_eq!(Some("foOb"), p.slice_to_inc('b'));
        assert_eq!(Some((4, 'a')), p.cur.next());

        let mut p = Parser::new("foObar");
        assert_eq!(Some("f"), p.slice_to_inc('f'));
        assert_eq!(Some((1, 'o')), p.cur.next());
    }

    #[test]
    fn slice_to_exc() {
        let mut p = Parser::new("foObar");
        assert_eq!(Some("foO"), p.slice_to_exc('b'));
        assert_eq!(Some((4, 'a')), p.cur.next());

        let mut p = Parser::new("foObar");
        assert_eq!(Some(""), p.slice_to_exc('f'));
        assert_eq!(Some((1, 'o')), p.cur.next());
    }

    #[test]
    fn slice_while() {
        let mut p = Parser::new("foObar");
        assert_eq!(Some("foO"), p.slice_while(|c| c != 'b'));
        assert_eq!(Some((3, 'b')), p.cur.next());

        let mut p = Parser::new("foObar");
        assert_eq!(None, p.slice_while(|c| c != 'f'));
        assert_eq!(Some((0, 'f')), p.cur.next());
    }

    #[test]
    fn parse() {
        let raw = r#"
                [dict]
                first = "first"
                # comment
                second ="another"
                whitespace = "  "
                empty = ""
                some_bool = true

                ary = [ "col1", 2,"col3", false]

                [table]

                |abc|def|
                |---|---|
                |one|two|
                # comment
                |  1| 2 |
                |  2| 3 |

                [three]
                a=1
                B=2
                | this |
            "#;

        let mut p = Parser::new(raw);

        assert_eq!(Some(Element::Section("dict".to_owned())), p.next());
        assert_eq!(Some(Entry("first".to_owned(), Value::String("first".to_owned()))), p.next());
        assert_eq!(Some(Comment(" comment\n".to_owned())), p.next());
        assert_eq!(Some(Entry("second".to_owned(), Value::String("another".to_owned()))), p.next());
        assert_eq!(Some(Entry("whitespace".to_owned(), Value::String("  ".to_owned()))), p.next());
        assert_eq!(Some(Entry("empty".to_owned(), Value::String("".to_owned()))), p.next());
        assert_eq!(Some(Entry("some_bool".to_owned(), Value::Boolean(true))), p.next());
        assert_eq!(Some(
            Entry("ary".to_owned(),
                  Value::Array(vec![
                      Value::String("col1".to_owned()),
                      Value::Integer(2),
                      Value::String("col3".to_owned()),
                      Value::Boolean(false)
                  ]))), p.next());

        assert_eq!(Some(Element::Section("table".to_owned())), p.next());
        assert_eq!(Some(Row(vec![Value::String("abc".to_owned()), Value::String("def".to_owned())])), p.next());
        assert_eq!(Some(Row(vec![Value::String("---".to_owned()), Value::String("---".to_owned())])), p.next());
        assert_eq!(Some(Row(vec![Value::String("one".to_owned()), Value::String("two".to_owned())])), p.next());
        assert_eq!(Some(Comment(" comment\n".to_owned())), p.next());
        assert_eq!(Some(Row(vec![Value::String("1".to_owned()), Value::String("2".to_owned())])), p.next());
        assert_eq!(Some(Row(vec![Value::String("2".to_owned()), Value::String("3".to_owned())])), p.next());
        assert_eq!(Some(Element::Section("three".to_owned())), p.next());
        assert_eq!(Some(Entry("a".to_owned(), Value::Integer(1))), p.next());
        assert_eq!(Some(Entry("B".to_owned(), Value::Integer(2))), p.next());
        assert_eq!(Some(Row(vec![Value::String("this".to_owned())])), p.next());
        assert_eq!(None, p.next());
        assert_eq!(None, p.next());
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Value::String("foo".to_owned())), "foo");
        assert_eq!(format!("{}", Value::Integer(1)), "1");
        assert_eq!(format!("{}", Value::Boolean(true)), "true");
        let ary = Value::Array(vec![Value::Integer(1), Value::String("foo".to_owned())]);
        assert_eq!(format!("{}", ary), "[ 1, \"foo\" ]");
    }

    mod read {
        use super::*;

        mod when_parsing_without_filtering {
            use super::*;

            mod and_ion_has_root_section {
                use super::*;

                mod and_root_section_has_dictionary_with_string {
                    use super::*;

                    #[test]
                    fn then_returns_dictionary() {
                        let raw = r#"
                            foo = "bar"
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        section.dictionary.insert("foo".to_owned(), Value::String("bar".to_owned()));
                        expected.insert("root".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_root_section_has_dictionary_with_array {
                    use super::*;

                    #[test]
                    fn then_returns_dictionary() {
                        let raw = r#"
                            arr = ["WAW", "WRO"]
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        let array = vec![Value::String("WAW".to_owned()), Value::String("WRO".to_owned())];
                        section.dictionary.insert("arr".to_owned(), Value::Array(array));
                        expected.insert("root".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_root_section_has_dictionary_with_dictionary {
                    use super::*;

                    #[test]
                    fn then_returns_dictionary() {
                        let raw = r#"
                            ndict = { foo = "bar" }
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        let mut dict = BTreeMap::new();
                        dict.insert("foo".to_owned(), Value::String("bar".to_owned()));
                        section.dictionary.insert("ndict".to_owned(), Value::Dictionary(dict));
                        expected.insert("root".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_root_section_has_dictionary_with_dictionary_with_new_lines {
                    use super::*;

                    #[test]
                    fn then_returns_dictionary() {
                        let raw = r#"
                            R75042 = {
                            view = "SV"
                            loc  = ["M", "B"]
                            dist = { beach_km = 4.1 }
                        }"#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut sect = Section::new();
                        let mut dict = BTreeMap::new();
                        dict.insert("view".to_owned(), Value::String("SV".to_owned()));
                        let array = vec![Value::String("M".to_owned()), Value::String("B".to_owned())];
                        dict.insert("loc".to_owned(), Value::Array(array));
                        let mut dict_dict = BTreeMap::new();
                        dict_dict.insert("beach_km".to_owned(), Value::Float(4.1));
                        dict.insert("dist".to_owned(), Value::Dictionary(dict_dict));
                        sect.dictionary.insert("R75042".to_owned(), Value::Dictionary(dict));
                        expected.insert("root".to_owned(), sect);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_root_section_has_dictionary_with_dictionary_with_no_value {
                    use super::*;

                    #[test]
                    fn then_returns_error() {
                        let raw = r#"
                            key =
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read();

                        assert_eq!(None, actual);
                    }
                }

                mod and_root_section_has_array {
                    use super::*;

                    #[test]
                    fn then_returns_array() {
                        let raw = r#"
                            |1|2|
                            |3|
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut sect = Section::new();
                        sect.rows.push(vec![Value::String("1".to_owned()), Value::String("2".to_owned())]);
                        sect.rows.push(vec![Value::String("3".to_owned())]);
                        expected.insert("root".to_owned(), sect);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_root_section_has_array_with_empty_cell {
                    use super::*;

                    #[test]
                    fn then_returns_array_with_empty_strings_on_empty_cells() {
                        let raw = r#"
                            |1||2|
                            |3|   |
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut sect = Section::new();
                        sect.rows.push(vec![Value::String("1".to_owned()), Value::String("".to_owned()), Value::String("2".to_owned())]);
                        sect.rows.push(vec![Value::String("3".to_owned()), Value::String("".to_owned())]);
                        expected.insert("root".to_owned(), sect);
                        assert_eq!(expected, actual);
                    }
                }
            }

            mod and_ion_has_section {
                use super::*;

                mod and_section_occurs_once {
                    use super::*;

                    #[test]
                    fn then_returns_section() {
                        let raw = r#"
                            [SECTION]

                            key = "value"
                            # now a table
                            | col1 | col2|
                            | col1 | col2| # comment
                            | col1 | col2|
                        "#;

                        let expected = {
                            let mut map = BTreeMap::new();
                            let mut section = Section::new();
                            section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
                            let mut row = Vec::new();
                            row.push(Value::String("col1".to_owned()));
                            row.push(Value::String("col2".to_owned()));
                            section.rows.push(row.clone());
                            section.rows.push(row.clone());
                            section.rows.push(row);
                            map.insert("SECTION".to_owned(), section);
                            map
                        };

                        let mut p = Parser::new(raw);
                        assert_eq!(expected, p.read().unwrap());
                    }
                }

                mod and_section_is_duplicated {
                    use super::*;

                    #[test]
                    fn then_returns_last_occurance_of_section() {
                        let raw = r#"
                            [SECTION]
                            1key = "1value"
                            | 1col1 | 1col2|
                            [SECTION]
                            2key = "2value"
                            | 2col1 | 2col2|
                        "#;
                        let mut p = Parser::new(raw);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        section.dictionary.insert("2key".to_owned(), Value::String("2value".to_owned()));
                        section.rows.push(vec![Value::String("2col1".to_string()), Value::String("2col2".to_string())]);
                        expected.insert("SECTION".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }
            }
        }

        mod when_parsing_with_filtering {
            use super::*;

            mod and_ion_has_root_section {
                use super::*;

                mod and_no_other_sections {
                    use super::*;

                    #[test]
                    fn then_returns_nothing() {
                        let raw = r#"
                            nkey = "nvalue"
                            | ncol1 | ncol2 |
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let expected = BTreeMap::new();
                        assert_eq!(expected, actual);
                    }
                }

                mod and_then_accepted_section {
                    use super::*;

                    #[test]
                    fn then_returns_accepted_section() {
                        let raw = r#"
                            nkey = "nvalue"
                            | ncol1 | ncol2 |
                            [ACCEPTED]
                            key = "value"
                            | col1 | col2|
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
                        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
                        expected.insert("ACCEPTED".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_then_filtered_section {
                    use super::*;

                    #[test]
                    fn then_returns_nothing() {
                        let raw = r#"
                            nkey = "nvalue"
                            | ncol1 | ncol2 |
                            [FILTERED]
                            key = "value"
                            | col1 | col2|
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let expected = BTreeMap::new();
                        assert_eq!(expected, actual);
                    }
                }
            }

            mod and_ion_has_accepted_section {
                use super::*;

                mod and_no_other_sections {
                    use super::*;

                    #[test]
                    fn then_returns_accepted_section() {
                        let raw = r#"
                            [ACCEPTED]
                            key = "value"
                            | col1 | col2|
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
                        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
                        expected.insert("ACCEPTED".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_then_filtered_section {
                    use super::*;

                    #[test]
                    fn then_returns_accepted_section() {
                        let raw = r#"
                            [ACCEPTED]
                            key = "value"
                            | col1 | col2|
                            [FILTERED]
                            fkey = "fvalue"
                            | fcol1 | fcol2|
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
                        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
                        expected.insert("ACCEPTED".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }

                mod and_then_duplicated_allowed_section {
                    use super::*;

                    mod and_it_is_the_only_accepted_section {
                        use super::*;

                        #[test]
                        fn then_returns_first_occurance_of_accepted_section() {
                            let raw = r#"
                                [ACCEPTED]
                                1key = "1value"
                                | 1col1 | 1col2|
                                [ACCEPTED]
                                2key = "2value"
                                | 2col1 | 2col2|
                            "#;
                            let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                            let actual = p.read().expect("Read failed");

                            let mut expected = BTreeMap::new();
                            let mut section = Section::new();
                            section.dictionary.insert("1key".to_owned(), Value::String("1value".to_owned()));
                            section.rows.push(vec![Value::String("1col1".to_string()), Value::String("1col2".to_string())]);
                            expected.insert("ACCEPTED".to_owned(), section);
                            assert_eq!(expected, actual);
                        }
                    }

                    mod and_it_is_not_the_only_accepted_section {
                        use super::*;

                        #[test]
                        fn then_returns_first_occurance_of_accepted_section() {
                            let raw = r#"
                                [ACCEPTED]
                                1key = "1value"
                                | 1col1 | 1col2|
                                [ACCEPTED]
                                2key = "2value"
                                | 2col1 | 2col2|
                            "#;
                            let mut p = Parser::new_filtered(raw, vec!["ACCEPTED", "ANOTHER"]);

                            let actual = p.read().expect("Read failed");

                            let mut expected = BTreeMap::new();
                            let mut section = Section::new();
                            section.dictionary.insert("1key".to_owned(), Value::String("1value".to_owned()));
                            section.rows.push(vec![Value::String("1col1".to_string()), Value::String("1col2".to_string())]);
                            expected.insert("ACCEPTED".to_owned(), section);
                            assert_eq!(expected, actual);
                        }
                    }
                }
            }

            mod and_ion_has_filtered_section {
                use super::*;

                mod and_no_other_sections {
                    use super::*;

                    #[test]
                    fn then_returns_nothing() {
                        let raw = r#"
                            [FILTERED]
                            key = "value"
                            | col1 | col2|
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let expected = BTreeMap::new();
                        assert_eq!(expected, actual);
                    }
                }


                mod and_then_accepted_section {
                    use super::*;

                    #[test]
                    fn then_returns_accepted_section() {
                        let raw = r#"
                            [FILTERED]
                            fkey = "fvalue"
                            | fcol1 | fcol2|
                            [ACCEPTED]
                            key = "value"
                            | col1 | col2|
                        "#;
                        let mut p = Parser::new_filtered(raw, vec!["ACCEPTED"]);

                        let actual = p.read().expect("Read failed");

                        let mut expected = BTreeMap::new();
                        let mut section = Section::new();
                        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
                        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
                        expected.insert("ACCEPTED".to_owned(), section);
                        assert_eq!(expected, actual);
                    }
                }
            }
        }
    }
}
