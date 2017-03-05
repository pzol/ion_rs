use Row;
use ion::Value;

pub trait FromRow: Sized {
    type Err;
    fn from_str_iter<'a, I: Iterator<Item = &'a Value>>(row: I) -> Result<Self, Self::Err>;

    // fn from_row(row: &Row) -> Result<Self, Self::Err>;
}

pub trait ParseRow: Sized {
    type Err;
    fn parse<F: FromRow>(&self) -> Result<F, F::Err>;
}

impl ParseRow for Row {
    type Err = ();
    fn parse<F: FromRow>(&self) -> Result<F, F::Err> {
        F::from_str_iter(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use ion::{FromRow, Value};

    #[derive(Debug, PartialEq)]
    struct Foo {
        foo: u32,
        bar: String,
    }

    impl FromRow for Foo {
        type Err = &'static str;
        fn from_str_iter<'a, I: Iterator<Item = &'a Value>>(mut row: I) -> Result<Self, Self::Err> {
            Ok(Foo {
                foo: parse_next!(row, "foo"),
                bar: parse_next!(row, "bar"),
            })
        }
    }

    #[test]
    fn from_row() {
        let row: Vec<_> = "1|foo".split("|").map(|s| Value::String(s.to_owned())).collect();
        let foo = Foo::from_str_iter(row.iter()).unwrap();
        assert_eq!(Foo {
                       foo: 1,
                       bar: "foo".to_owned(),
                   },
                   foo);
    }
}
