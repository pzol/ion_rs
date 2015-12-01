use { Ion, Section, Value };

use std::fmt;

impl fmt::Display for Ion {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (name, section) in &self.sections {
            try!(f.write_fmt(format_args!("[{}]\n", name)));
            try!(section.fmt(f));
            try!(f.write_str("\n"));
        }
        Ok(())
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (k, v) in &self.dictionary {
            if v.type_str() == "string" {
                try!(f.write_fmt(format_args!("{} = \"{}\"\n", k, v)));
            } else {
                try!(f.write_fmt(format_args!("{} = {}\n", k, v)));
            }
        }

        for row in &self.rows {
            for cell in row {
                try!(fmt::Display::fmt(&format!("| {} ", cell), f));
            }
            try!(f.write_str("|\n"));
        }
        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Value::String(ref v)     => v.fmt(f),
            Value::Integer(ref v)    => v.fmt(f),
            Value::Float(ref v)      => v.fmt(f),
            Value::Boolean(ref v)    => v.fmt(f),
            Value::Array(ref v)      => {
                try!(f.write_str("[ "));

                let mut first = true;
                for i in v {
                    if first { first = false } else { try!(f.write_str(", ")) }
                    try!(i.fmt(f));
                }
                f.write_str(" ]")
            },
            Value::Dictionary(ref d) => {
                try!(f.write_str("{ "));

                let mut first = true;
                for (k, v) in d {
                    if first { first = false } else { try!(f.write_str(", ")) }
                    try!(k.fmt(f));
                    try!(f.write_str(" = "));
                    if v.type_str() == "string" {
                        try!(f.write_str("\""));
                        try!(v.fmt(f));
                        try!(f.write_str("\""));
                    } else {
                        try!(v.fmt(f));
                    }
                }
                f.write_str(" }")
            }
        }
    }
}

impl fmt::Display for super::Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
