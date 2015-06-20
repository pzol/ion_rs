use std::str;
use std::fmt;
use std::collections::BTreeMap;

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

#[derive(Debug)]
pub enum Section {
    Dictionary(BTreeMap<String, String>),
    Table(Vec<Vec<String>>)
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Parser {
            input: s,
            cur: s.char_indices(),
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Option<BTreeMap<String, Section>> {
        let mut ion = BTreeMap::new();

        while self.peek(0).is_some() {
            self.ws();
            if self.newline() { continue }
            if self.comment() { continue }

            let mut name = String::new();
            if self.eat('[') {
                self.ws();
                while let Some((_, ch)) = self.cur.next() {
                    if ch == ']' { break }
                    name.push(ch)
                }
            }

            let body = self.body();
            if body.is_none() { return None }
            ion.insert(name, body.unwrap());
        }

        Some(ion)
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

    fn comment(&mut self) -> bool {
        if !self.eat('#') { return false }
        for (_, ch) in self.cur.by_ref() {
            if ch == '\n' { break }
        }
        true
    }

    fn eat(&mut self, ch: char) -> bool {
        match self.peek(0) {
            Some((_, c)) if c == ch => { self.cur.next(); true }
            Some(_) | None => false
        }
    }

    fn body(&mut self) -> Option<Section> {
        loop {
            self.ws();
            if self.newline() { continue }
            if self.comment() { continue }

            match self.peek(0) {
                Some((_, '|'))  => return self.table(),
                Some((_, '[')) => return None,
                Some(..) => return return self.dictionary(),
                None => return None,
            }
        }
    }

    // fn expect(&self, expected: char) -> bool {
    //     if let Some((_, ch)) = self.peek(0) {
    //         expected == ch
    //     } else {
    //         false
    //     }
    // }

    fn dictionary(&mut self) -> Option<Section> {
        let mut dictionary = BTreeMap::new();
        loop {
            self.ws();
            if self.newline() { continue }
            if self.comment() { continue }

            match self.peek(0) {
                Some((_, '[')) | None => break,
                Some(..) => {}
            }

            let mut key = String::new();
            while let Some((_, ch)) = self.cur.next() {
                if ch == '=' { break }
                key.push(ch);
            }

            let mut value = String::new();
            while let Some((_, ch)) = self.cur.next() {
                if ch == '\n' { break }
                value.push(ch);
            }

            dictionary.insert(key, value);
        }

        Some(Section::Dictionary(dictionary))
    }

    fn table(&mut self) -> Option<Section> {
        let mut rows = Vec::new();
        let mut row  = Vec::new();

        loop {
            self.ws();
            if self.newline() { continue }
            if self.comment() { continue }

            match self.peek(0) {
                None => break,
                Some((_, '[')) => break,
                _ => {}
            }

            row.push(self.cell());

            if self.newline() { rows.push(row); row = Vec::new();  }
        }

        Some(Section::Table(rows))
    }

    fn cell(&mut self) -> String {
        let mut ret = String::new();
        self.eat('|');
        self.ws();

        while let Some((_, ch)) = self.cur.next() {
            if ch == '|' { break }
            ret.push(ch)
        }

        ret
    }
}

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
    use super::Parser;
    #[test]
    fn it_works() {
        let raw = r#"
        [dict]
        first="first";

        second="another"

        [table]

        |abc|def|
        |---|---|
        |one|two|
        # comment
        |  1| 2 |

        [three]
        a=1
        "#;

        let mut parser = Parser::new(raw);
        let ion = parser.parse();
        panic!("{:?}", ion);
    }
}
