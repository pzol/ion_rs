use std::collections::BTreeMap;
use std::str;

#[derive(Debug, PartialEq)]
pub enum Element {
    Section(String),
    Row(Vec<Value>),
    Entry(String, Value),
    Comment(String)
}

#[derive(Debug)]
pub enum Section {
    Dictionary(BTreeMap<String, Value>),
    Table(Vec<Vec<Value>>)
}


#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(String),
    Array(Vec<Value>)
}

#[derive(Debug)]
enum State {
    None,
    Section,
    Table,
    Dictionary
}

pub struct Parser<'a> {
    input: &'a str,
    cur: str::CharIndices<'a>,
    state: State
}

macro_rules! some {
    ($expr:expr) => ({
        let ret = $expr;
        if ret.is_some() { return ret; }
    })
}

macro_rules! try {
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
        while self.peek(0).is_some() {
            self.ws();
            if self.newline() { continue }
            some!(self.comment());

            return match self.state {
                State::None       => self.section(),
                State::Section    => self.body(),
                State::Dictionary => self.dictionary(),
                State::Table      => self.table()
            };
        }

        None
    }
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Parser {
            input: s,
            cur: s.char_indices(),
            state: State::None
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

        self.state = State::Section;
        Some(Element::Section(name))
    }

    fn body(&mut self) -> Option<Element> {
        loop {
            self.ws();
            if self.newline() { continue }
            some!(self.comment());

            return match self.peek(0) {
                Some((_, '|'))  => self.table(),
                Some((_, '['))  => None,
                Some(..)        => self.dictionary(),
                None => None,
            }
        }
    }

    fn dictionary(&mut self) -> Option<Element> {
        self.state = State::Dictionary;

        loop {
            self.ws();
            if self.newline() { continue }
            some!(self.comment());

            match self.peek(0) {
                None => return None,
                Some((_, '['))  => break,
                Some(..) => {}
            }

            let key = try!(self.key_name());
            if !self.keyval_sep() { return None }
            let val = try!(self.value());
            if let Some((_, ch)) = self.peek(0) {
                if ch != '\n' { return None }
            }

            return Some(Element::Entry(key, val))
        }

        self.state = State::None;
        self.section()
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
        // if self.eat('"') { return self.finish_string(); }

        match self.cur.clone().next() {
            Some((_, '"')) => return self.finish_string(),
            Some((_, ch)) if is_digit(ch) => self.integer(),
            Some((pos, 't')) |
            Some((pos, 'f')) => self.boolean(pos),
            _ => None
        }
    }

    fn integer(&mut self) -> Option<Value> {
        let mut ret = String::new();
        while let Some((_, ch)) = self.cur.clone().next() {
            match ch {
                '0' ... '9' => { self.cur.next(); ret.push(ch) },
                '\n' => break,
                _ => return None
            }
        }

        let num : i64 = match ret.parse() {
            Ok(num) => num,
            Err(_)  => return None
        };

        Some(Value::Integer(num))
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

    fn table(&mut self) -> Option<Element> {
        self.state = State::Table;
        let mut row  = Vec::new();

        loop {
            self.ws();
            if self.newline() { continue }
            some!(self.comment());

            match self.peek(0) {
                None => break,
                Some((_, '[')) => break,
                _ => {}
            }

            row.push(Value::String(self.cell()));

            if self.newline() { return Some(Element::Row(row)) }
        }

        self.state = State::None;
        self.section()
    }

    fn cell(&mut self) -> String {
        let mut ret = String::new();
        self.eat('|');
        self.ws();

        while let Some((_, ch)) = self.cur.next() {
            if ch == '|' { break }
            ret.push(ch)
        }

        ret.trim_right().to_owned()
    }

    pub fn read(&mut self) -> BTreeMap<String, Section> {
        let mut map = BTreeMap::new();

        let mut section : Option<Section> = None;
        let mut name    = None;

        while let Some(el) = self.next() {
            match el {
                Element::Section(ref n) if section.is_none() => name = Some(n.clone()),
                Element::Section(n) => {
                    map.insert(name.unwrap(), section.unwrap());
                    section = None;
                    name = Some(n)
                },

                Element::Row(ref row) if section.is_none() => section = Some(Section::Table(vec![row.clone()])),
                Element::Row(row) => {
                    match section {
                        Some(Section::Table(ref mut rows)) => {
                            rows.push(row)
                        }
                        _ => panic!("adding a row to a NON-table")
                    }
                },
                Element::Entry(ref key, ref value) if section.is_none() => section = {
                    let mut dict = BTreeMap::new();
                    dict.insert(key.clone(), value.clone());
                    Some(Section::Dictionary(dict))
                },
                Element::Entry(ref key, ref value) => {
                    match section {
                        Some(Section::Dictionary(ref mut dict)) => {
                            dict.insert(key.clone(), value.clone());
                        },
                        _ => panic!("adding a dictionary entry to a NON-dictionary")
                    }
                }
                _ => ()
            }
        }

        if let Some(s) = section {
            map.insert(name.expect("name must not be empty"), s);
        }

        map
    }

}

fn is_digit(c: char) -> bool {
    match c { '0' ... '9' => true, _ => false }
}

#[allow(dead_code)]
pub struct ParserError {
    /// The low byte at which this error is pointing at.
    pub lo: usize,
    /// One byte beyond the last character at which this error is pointing at.
    pub hi: usize,
    /// A human-readable description explaining what the error is.
    pub desc: String,
}

#[cfg(test)]
mod tests {
    macro_rules! next {
        ($parser:ident) => ({
            $parser.next().unwrap()
        })
    }

    use super::{ Parser, Value };
    use super::Element::*;

    #[test]
    fn parse() {
        let raw = r#"
        [dict]
        first = "first"
        # comment
        second ="another"
        some_bool = true

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
        "#;

        let mut p = Parser::new(raw);
        assert_eq!(Section("dict".to_owned()), next!(p));
        assert_eq!(Entry("first".to_owned(), Value::String("first".to_owned())), next!(p));
        assert_eq!(Comment(" comment\n".to_owned()), next!(p));
        assert_eq!(Entry("second".to_owned(), Value::String("another".to_owned())), next!(p));
        assert_eq!(Entry("some_bool".to_owned(), Value::Boolean(true)), next!(p));
        assert_eq!(Section("table".to_owned()), next!(p));
        assert_eq!(Row(vec![Value::String("abc".to_owned()), Value::String("def".to_owned())]), next!(p));
        assert_eq!(Row(vec![Value::String("---".to_owned()), Value::String("---".to_owned())]), next!(p));
        assert_eq!(Row(vec![Value::String("one".to_owned()), Value::String("two".to_owned())]), next!(p));
        assert_eq!(Comment(" comment\n".to_owned()), next!(p));
        assert_eq!(Row(vec![Value::String("1".to_owned()), Value::String("2".to_owned())]), next!(p));
        assert_eq!(Row(vec![Value::String("2".to_owned()), Value::String("3".to_owned())]), next!(p));
        assert_eq!(Section("three".to_owned()), next!(p));
        assert_eq!(Entry("a".to_owned(), Value::Integer(1)), next!(p));
        assert_eq!(Entry("B".to_owned(), Value::Integer(2)), next!(p));
        assert_eq!(None, p.next());
        assert_eq!(None, p.next());
    }

    #[test]
    fn mapping() {
        let raw = r#"
        [MAPPING]
        "#;
    }
}
