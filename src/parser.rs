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
    accepted_sections: &'a[&'a str],
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
        loop {
            self.ws();
            if self.newline() { continue }

            if let Some((_, c)) = self.peek(0) {
                return match c {
                    '[' => self.section(),
                    '|' => self.row(),
                    '#' => self.comment(),
                    _   => self.entry()
                };
            } else {
                return None;
            }
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Self::new_filtered(s, &[])
    }

    pub fn new_filtered(s: &'a str, accepted_sections: &'a[&'a str]) -> Parser<'a> {
        Parser {
            input: s,
            cur: s.char_indices(),
            errors: Vec::new(),
            accepted_sections,
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

    fn comment(&mut self) -> Option<Element> {
        if !self.eat('#') { return None }
        let mut ret = String::new();
        for (_, ch) in self.cur.by_ref() {
            ret.push(ch);
            if ch == '\n' { break }
        }
        Some(Element::Comment(ret))
    }

    fn eat(&mut self, ch: char) -> bool {
        match self.peek(0) {
            Some((_, c)) if c == ch => { self.cur.next(); true }
            Some(_) | None => false
        }
    }

    fn section(&mut self) -> Option<Element> {
        let mut name = String::new();

        if self.eat('[') {
            self.ws();
            while let Some((_, ch)) = self.cur.next() {
                if ch == ']' { break }
                name.push(ch)
            }
        }

        Some(Element::Section(name))
    }

    fn entry(&mut self) -> Option<Element> {
        let key = some!(self.key_name());
        if !self.keyval_sep() { return None }
        let val = some!(self.value());

        Some(Element::Entry(key, val))
    }

    fn key_name(&mut self) -> Option<String> {
        let mut ret = String::new();
        while let Some((_, ch)) = self.cur.clone().next() {
            match ch {
                'a' ... 'z' |
                'A' ... 'Z' |
                '0' ... '9' |
                '_' | '-' => { self.cur.next(); ret.push(ch) }
                _ => break,
            }
        }
        Some(ret)
    }

    fn value(&mut self) -> Option<Value> {
        self.ws();
        self.newline();
        self.ws();
        // if self.eat('"') { return self.finish_string(); }
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
        let mut row = Vec::new();

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

        println!("{:?}.{:?}", prefix, decimal);

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
        let mut ret = String::new();
        while let Some((_, ch)) = self.cur.clone().next() {
            match ch {
                '0' ... '9' => { self.cur.next(); ret.push(ch) },
                _ => break
            }
        }

        Some(ret)
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
        let mut val = String::new();
        while let Some((_, ch)) = self.cur.next() {
            if ch == '"' { return Some(Value::String(val)); }
            val.push(ch);
        }
        None
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
        let mut row  = Vec::new();
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
        let mut ret = String::new();
        self.ws();

        while let Some((_, ch)) = self.cur.next() {
            if ch == '|' { break }
            ret.push(ch)
        }

        ret.trim_right().to_owned()
    }

    pub fn read(&mut self) -> Option<BTreeMap<String, Section>> {
        let mut map = BTreeMap::new();

        let mut name    = None;
        let mut section = Some(Section::new());

        while let Some(el) = self.next() {
            match el {
                Element::Section(ref n) => {
                    if let (Some(name), Some(section)) = (name.take(), section.take()) {
                        map.insert(name, section);
                    }
                    if self.is_section_accepted(n) {
                        name = Some(n.to_string());
                        section = Some(Section::new());
                    }
                },
                Element::Row(row) =>
                    if let Some(ref mut s) = section {
                        s.rows.push(row);
                    },
                Element::Entry(ref key, ref value) =>
                    if let Some(ref mut s) = section {
                        s.dictionary.insert(key.clone(), value.clone());
                    },
                _ => (),
            };
        }

        if let Some(section) = section {
            map.insert(name.unwrap_or_else(|| "root".to_string()), section);
        }

        if self.errors.len() > 0 {
            None
        } else {
            Some(map)
        }
    }

    fn is_section_accepted(&self, name: &str) -> bool {
        self.accepted_sections.is_empty()
            || self.accepted_sections.contains(&name)
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

    use { Ion, Parser, Value, Section };
    use super::Element::{ self, Row, Entry, Comment };
    use std::collections::BTreeMap;

    #[test]
    fn empty_cell() {
        let raw = r#"
            [EMPTY]
            |1||2|
            |1|   |2|
        "#;

        let mut p = Parser::new(raw);
        assert_eq!(Some(Element::Section("EMPTY".to_owned())), p.next());
        assert_eq!(Some(Row(vec![Value::String("1".to_owned()), Value::String("".to_owned()), Value::String("2".to_owned())])), p.next());
        assert_eq!(Some(Row(vec![Value::String("1".to_owned()), Value::String("".to_owned()), Value::String("2".to_owned())])), p.next());
        assert_eq!(None, p.next());
    }

    #[test]
    fn err_entry() {
        let raw = r#"
            [err]
            key = 
        "#;

        let res : Result<Ion, _> = raw.parse();
        assert!(res.is_err());
    }

    #[test]
    fn parse() {
        let raw = r#"
        [dict]
        first = "first"
        # comment
        second ="another"
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
    fn no_section() {
        let raw = r#"
        foo = "bar"
        "#;

        let ion = Parser::new(raw).read().unwrap();
        let s   = ion.get("root").unwrap();
        assert_eq!(format!("{}", s), "foo = \"bar\"\n");
    }

    #[test]
    fn string_ary() {
        let raw = r#"
        ary = ["WAW", "WRO"]"#;

        let ion = Parser::new(raw).read().unwrap();
        let s   = ion.get("root").unwrap();
        assert_eq!(s.to_string(), "ary = [ \"WAW\", \"WRO\" ]\n");
    }

    #[test]
    fn nested_dictionary() {
        let raw = r#"
        [dict]
        ndict = { foo = "bar" }
        "#;

        let expected = {
            let mut map = BTreeMap::new();
            let mut sect = Section::new();
            let mut dict = BTreeMap::new();
            dict.insert("foo".to_owned(), Value::String("bar".to_owned()));
            sect.dictionary.insert("ndict".to_owned(), Value::Dictionary(dict));
            map.insert("dict".to_owned(), sect);
            map
        };

        let mut p = Parser::new(raw);
        assert_eq!(expected, p.read().unwrap());
    }

    #[test]
    fn read() {
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

    #[test]
    fn dict_with_newline() {
        let ion = ion!(r#"[TEST]
    R75042 = {
    view = "SV"
    loc  = ["M", "B"]
    dist = { beach_km = 4.1 }
}"#);
        assert_eq!("[TEST]\nR75042 = { dist = { beach_km = 4.1 }, loc = [ \"M\", \"B\" ], view = \"SV\" }\n\n", ion.to_string());
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Value::String("foo".to_owned())), "foo");
        assert_eq!(format!("{}", Value::Integer(1)), "1");
        assert_eq!(format!("{}", Value::Boolean(true)), "true");
        let ary = Value::Array(vec![Value::Integer(1), Value::String("foo".to_owned())]);
        assert_eq!(format!("{}", ary), "[ 1, \"foo\" ]");
    }

    #[test]
    fn accepted_section() {
        let raw = r#"
        [ACCEPTED]
        key = "value"
        | col1 | col2|
        "#;
        let mut p = Parser::new_filtered(raw, &["ACCEPTED"]);

        let actual = p.read().expect("Read failed");

        let mut expected = BTreeMap::new();
        let mut section = Section::new();
        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
        expected.insert("ACCEPTED".to_owned(), section);
        assert_eq!(expected, actual);
    }

    #[test]
    fn filtered_section() {
        let raw = r#"
        [FILTERED]
        key = "value"
        | col1 | col2|
        "#;
        let mut p = Parser::new_filtered(raw, &["ACCEPTED"]);

        let actual = p.read().expect("Read failed");

        let expected = BTreeMap::new();
        assert_eq!(expected, actual);
    }

    #[test]
    fn no_section_then_accepted_section() {
        let raw = r#"
        nkey = "nvalue"
        | ncol1 | ncol2 |
        [ACCEPTED]
        key = "value"
        | col1 | col2|
        "#;
        let mut p = Parser::new_filtered(raw, &["ACCEPTED"]);

        let actual = p.read().expect("Read failed");

        let mut expected = BTreeMap::new();
        let mut section = Section::new();
        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
        expected.insert("ACCEPTED".to_owned(), section);
        assert_eq!(expected, actual);
    }

    #[test]
    fn no_section_then_filtered_section() {
        let raw = r#"
        nkey = "nvalue"
        | ncol1 | ncol2 |
        [FILTERED]
        key = "value"
        | col1 | col2|
        "#;
        let mut p = Parser::new_filtered(raw, &["ACCEPTED"]);

        let actual = p.read().expect("Read failed");

        let expected = BTreeMap::new();
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepted_section_then_filtered_section() {
        let raw = r#"
        [ACCEPTED]
        key = "value"
        | col1 | col2|
        [FILTERED]
        fkey = "fvalue"
        | fcol1 | fcol2|
        "#;
        let mut p = Parser::new_filtered(raw, &["ACCEPTED"]);

        let actual = p.read().expect("Read failed");

        let mut expected = BTreeMap::new();
        let mut section = Section::new();
        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
        expected.insert("ACCEPTED".to_owned(), section);
        assert_eq!(expected, actual);
    }

    #[test]
    fn filtered_section_then_accepted_section() {
        let raw = r#"
        [FILTERED]
        fkey = "fvalue"
        | fcol1 | fcol2|
        [ACCEPTED]
        key = "value"
        | col1 | col2|
        "#;
        let mut p = Parser::new_filtered(raw, &["ACCEPTED"]);

        let actual = p.read().expect("Read failed");

        let mut expected = BTreeMap::new();
        let mut section = Section::new();
        section.dictionary.insert("key".to_owned(), Value::String("value".to_owned()));
        section.rows.push(vec![Value::String("col1".to_string()), Value::String("col2".to_string())]);
        expected.insert("ACCEPTED".to_owned(), section);
        assert_eq!(expected, actual);
    }
}
