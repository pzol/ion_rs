use std::ascii::AsciiExt;
use std::char;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::str;

pub struct Parser<'a> {
    input: &'a str,
    cur: str::CharIndices<'a>,

    /// A list of all errors which have occurred during parsing.
    ///
    /// Not all parse errors are fatal, so this list is added to as much as
    /// possible without aborting parsing. If `None` is returned by `parse`, it
    /// is guaranteed that this list is not empty.
    pub errors: Vec<ParserError>,
}

pub struct ParserError {
    /// The low byte at which this error is pointing at.
    pub lo: usize,
    /// One byte beyond the last character at which this error is pointing at.
    pub hi: usize,
    /// A human-readable description explaining what the error is.
    pub desc: String,
}


impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Parser {
            input: s,
            cur: s.char_indices(),
            errors: Vec::new(),
        }
    }

    // Returns true and consumes the next character if it matches `ch`,
    // otherwise do nothing and return false
    fn eat(&mut self, ch: char) -> bool {
        match self.peek(0) {
            Some((_, c)) if c == ch => { self.cur.next(); true }
            Some(_) | None => false,
        }
    }

    // Consumes whitespace ('\t' and ' ') until another character (or EOF) is
    // reached. Returns if any whitespace was consumed
    fn ws(&mut self) -> bool {
        let mut ret = false;
        loop {
            match self.peek(0) {
                Some((_, '\t')) |
                Some((_, ' ')) => { self.cur.next(); ret = true; }
                _ => break,
            }
        }
        ret
    }

    // Consumes a newline if one is next
    fn newline(&mut self) -> bool {
        match self.peek(0) {
            Some((_, '\n')) => { self.cur.next(); true }
            Some((_, '\r')) if self.peek(1).map(|c| c.1) == Some('\n') => {
                self.cur.next(); self.cur.next(); true
            }
            _ => false
        }
    }

    fn expect(&mut self, ch: char) -> bool {
        if self.eat(ch) { return true }
        let mut it = self.cur.clone();
        let lo = it.next().map(|p| p.0).unwrap_or(self.input.len());
        let hi = it.next().map(|p| p.0).unwrap_or(self.input.len());
        self.errors.push(ParserError {
            lo: lo,
            hi: hi,
            desc: match self.cur.clone().next() {
                Some((_, c)) => format!("expected `{}`, but found `{}`", ch, c),
                None => format!("expected `{}`, but found eof", ch)
            }
        });
        false
    }

    /// Executes the parser, parsing the string contained within.
    ///
    /// This function will return the `TomlTable` instance if parsing is
    /// successful, or it will return `None` if any parse error or invalid TOML
    /// error occurs.
    ///
    /// If an error occurs, the `errors` field of this parser can be consulted
    /// to determine the cause of the parse failure.
    pub fn parse(&mut self) -> Option<TomlTable> {
        let mut ret = BTreeMap::new();
        while self.peek(0).is_some() {
            self.ws();
            if self.newline() { continue }
            if self.comment() { continue }
            if self.eat('[') {
                let array = self.eat('[');
                let start = self.next_pos();

                // Parse the name of the section
                let mut keys = Vec::new();
                loop {
                    self.ws();
                    match self.key_name() {
                        Some(s) => keys.push(s),
                        None => {}
                    }
                    self.ws();
                    if self.eat(']') {
                        if array && !self.expect(']') { return None }
                        break
                    }
                    if !self.expect('.') { return None }
                }
                if keys.len() == 0 { return None }

                // Build the section table
                let mut table = BTreeMap::new();
                if !self.values(&mut table) { return None }
                if array {
                    self.insert_array(&mut ret, &*keys, Table(table), start)
                } else {
                    self.insert_table(&mut ret, &*keys, table, start)
                }
            } else {
                if !self.values(&mut ret) { return None }
            }
        }
        if self.errors.len() > 0 {
            None
        } else {
            Some(ret)
        }
    }

}
