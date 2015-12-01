use std::error;
use std::fmt;
use Value;
use self::DecodeErrorKind::*;

mod rustc_serialize;

pub struct Decoder {
    value: Option<Value>,
    cur_field: Option<String>,
}

#[derive(Debug)]
pub struct DecodeError {
    pub field: Option<String>,
    pub kind: DecodeErrorKind,
}

/// Enumeration of possible errors which can occur while decoding a structure.
#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum DecodeErrorKind {
    /// An error flagged by the application, e.g. value out of range
    ApplicationError(String),
    /// A field was expected, but none was found.
    ExpectedField(/* type */ Option<&'static str>),
    /// A field was found, but it was not an expected one.
    UnknownField,
    /// A field was found, but it had the wrong type.
    ExpectedType(/* expected */ &'static str, /* found */ &'static str),
    //// The nth map key was expected, but none was found.
    ExpectedMapKey(usize),
    /// The nth map element was expected, but none was found.
    ExpectedMapElement(usize),
    /// An enum decoding was requested, but no variants were supplied
    NoEnumVariants,
    /// The unit type was being decoded, but a non-zero length string was found
    NilTooLong,
    /// There was an error with the syntactical structure of the TOML.
    SyntaxError,
    /// The end of the TOML input was reached too soon
    EndOfStream,
}

impl error::Error for DecodeError {
    fn description(&self) -> &str {
        match self.kind {
            ApplicationError(ref s) => &**s,
            ExpectedField(..) => "expected a field",
            UnknownField => "found an unknown field",
            ExpectedType(..) => "expected a type",
            ExpectedMapKey(..) => "expected a map key",
            ExpectedMapElement(..) => "expected a map element",
            NoEnumVariants => "no enum variants to decode to",
            NilTooLong => "nonzero length string representing nil",
            SyntaxError => "syntax error",
            EndOfStream => "end of stream",
        }
    }
}

impl fmt::Display for DecodeError {
    #[allow(dead_code)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(match self.kind {
            ApplicationError(ref err) => {
                write!(f, "{}", err)
            }
            ExpectedField(expected_type) => {
                match expected_type {
                    Some("table") => write!(f, "expected a section"),
                    Some("field") => write!(f, "expected the field"),
                    Some(e) => write!(f, "expected a value of type `{}`", e),
                    None => write!(f, "expected a value"),
                }
            }
            UnknownField => write!(f, "unknown field"),
            ExpectedType(expected, found) => {
                fn humanize(s: &str) -> String {
                    if s == "section" {
                        format!("a section")
                    } else {
                        format!("a value of type `{}`", s)
                    }
                }
                write!(f, "expected {}, but found {}",
                       humanize(expected),
                       humanize(found))
            }
            ExpectedMapKey(idx) => {
                write!(f, "expected at least {} keys", idx + 1)
            }
            ExpectedMapElement(idx) => {
                write!(f, "expected at least {} elements", idx + 1)
            }
            NoEnumVariants => {
                write!(f, "expected an enum variant to decode to")
            }
            NilTooLong => {
                write!(f, "expected 0-length string")
            }
            SyntaxError => {
                write!(f, "syntax error")
            }
            EndOfStream => {
                write!(f, "end of stream")
            }
        });
        match self.field {
            Some(ref s) => {
                write!(f, " for the key `{}`", s)
            }
            None => Ok(())
        }
    }
}

impl Decoder {
    pub fn new(value: Value) -> Decoder {
        Decoder {
            value: Some(value),
            cur_field: None
        }
    }

    fn err(&self, kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            field: self.cur_field.clone(),
            kind: kind,
        }
    }

    fn mismatch(&self, expected: &'static str,
                found: &Option<Value>) -> DecodeError{
        match *found {
            Some(ref val) => self.err(ExpectedType(expected, val.type_str())),
            None => self.err(ExpectedField(Some(expected))),
        }
    }

    fn sub_decoder(&self, value: Option<Value>, field: &str) -> Decoder {
        Decoder {
            value: value,
            cur_field: if field.len() == 0 {
                self.cur_field.clone()
            } else {
                match self.cur_field {
                    None => Some(format!("{}", field)),
                    Some(ref s) => Some(format!("{}.{}", s, field))
                }
            }
        }
    }
}

pub fn decode<T: ::rustc_serialize::Decodable + fmt::Debug>(value: Value) -> Result<T, DecodeError> {
    ::rustc_serialize::Decodable::decode(&mut Decoder::new(value))
}

#[cfg(test)]
mod tests {
    use ion::decode;
    use ::rustc_serialize as rs;
    use Value;

    #[derive(Debug, PartialEq)]
    struct Bar {
        foo: u32
    }

    impl rs::Decodable for Bar {
        fn decode<D: rs::Decoder>(d: &mut D) -> Result<Bar, D::Error> {
            let foo = try!(d.read_u32());
            Ok(Bar { foo: foo})
        }
    }

    #[derive(Debug, PartialEq, RustcDecodable)]
    struct Foo {
        int: u32,
        string: String,
        opt_string: Option<String>,
        yesno: bool,
        nested: Bar,
        ary: Vec<u32>
    }

    #[test]
    fn test_decode() {
        let row : Vec<_> = "1|foo|some|true|1|1,2".split("|").map(|s| Value::String(s.to_owned())).collect();
        let foo : Foo = decode(Value::Array(row)).expect("decode");

        assert_eq!(1, foo.int);
        assert_eq!("foo", foo.string);
        assert_eq!(Some("some".to_owned()), foo.opt_string);
        assert_eq!(true, foo.yesno);
        assert_eq!(Bar { foo: 1 }, foo.nested);
        assert_eq!(vec![1, 2], foo.ary);
    }
}
