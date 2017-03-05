use std::{ convert  };
use std::io::{ self, Write };
use Value;

pub type Result = io::Result<()>;

pub struct Writer<W: Write> {
    writer: Box<W>
}

impl<W: Write> Writer<W> {
    pub fn new(w: Box<W>) -> Writer<W> {
        Writer { writer: w }
    }

    pub fn write(&mut self, text: &str) -> Result {
        try!(self.writer.write(text.as_bytes()));
        Ok(())
    }

    pub fn section(&mut self, name: &str) -> Result {
        try!(self.write("["));
        try!(self.write(name));
        self.write("]\n")
    }

    pub fn key_value<'a, I: Into<String>>(&mut self, name: &str, value: I) -> Result {
        try!(self.write(name));
        try!(self.write(" = "));
        try!(self.write(&value.into()));
        self.write("\n")
    }
}

impl<'a> convert::From<&'a Value> for String {
    fn from(v: &Value) ->  String {
        match v {
            &Value::String(ref s) => format!("\"{}\"", s),
            &Value::Array(ref ary) => {
                let mut out = String::new();
                let mut first = true;
                out.push_str("[ ");
                for v in ary {
                    if first { first = false } else { out.push_str(", ")}
                    let s : String = v.into();
                    out.push_str(&s)
                }
                out.push_str(" ]");
                out
            }
            _ => v.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use { Value, Writer };

    #[test]
    #[allow(unused_must_use)]
    fn writer() {
        use std::collections::BTreeMap;
        let mut s = String::new();
        {
            let mut w = Writer::new(Box::new(unsafe { s.as_mut_vec() }));
            w.section("TEST");
            w.key_value("string", &Value::String("bar".to_string()));
            w.key_value("integer", &Value::Integer(1));
            w.key_value("boolean", &Value::Boolean(true));
            w.key_value("array", &Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::String("foobar".to_string())]));
            let mut dict = BTreeMap::new();
            dict.insert("foo".to_string(), Value::String("bar".to_string()));
            w.key_value("dict", &Value::Dictionary(dict));
        }

        assert_eq!(r#"[TEST]
string = "bar"
integer = 1
boolean = true
array = [ 1, 2, "foobar" ]
dict = { foo = "bar" }
"#, s);

    }
}