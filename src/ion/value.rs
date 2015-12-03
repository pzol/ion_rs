use std::str::FromStr;
use { Dictionary, FromIon, IonError, Row };

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Row),
    Dictionary(Dictionary)
}

impl Value {
    pub fn type_str(&self) -> &'static str {
        match *self {
            Value::String(..)     => "string",
            Value::Integer(..)    => "integer",
            Value::Float(..)      => "float",
            Value::Boolean(..)    => "boolean",
            Value::Array(..)      => "array",
            Value::Dictionary(..) => "dictionary"
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match *self {
            Value::String(ref v) => Some(v),
            _ => None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref v) => Some(v.as_str()),
            _ => None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match *self {
            Value::Integer(v) => Some(v),
            _ => None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match *self {
            Value::Float(v) => Some(v),
            _ => None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            Value::Boolean(v) => Some(v),
            _ => None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Array(ref v) => Some(v),
            _ => None
        }
    }

    pub fn as_dictionary(&self) -> Option<&Dictionary> {
        match *self {
            Value::Dictionary(ref v) => Some(v),
            _ => None
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        match *self {
            Value::Dictionary(ref v) => v.get(name),
            _ => None
        }
    }

    /// convert to type `F` using the `FromIon` trait
    pub fn from_ion<F : FromIon>(&self) -> Result<F, F::Err> {
        F::from_ion(self)
    }

    /// parse to the resulting type, if the inner value is not a string, convert to string first
    pub fn parse<F : FromStr>(&self) -> Result<F, F::Err> {
        match self.as_string() {
            Some(s) => s.parse(),
            None    => self.to_string().parse()
        }
    }
}

impl FromStr for Value {
    type Err = IonError;

    fn from_str(s: &str) -> Result<Value, IonError> {
        Ok(Value::String(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use Value;

    #[test]
    fn parse() {
        let v : Value = "1".parse().unwrap();
        assert_eq!(1, v.parse().unwrap());
    }
}
