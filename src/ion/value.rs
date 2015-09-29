use { Dictionary, Row };

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    // Float(f64),
    Boolean(bool),
    // Datetime(String),
    Array(Row),
    Dictionary(Dictionary)
}

impl Value {
    pub fn type_str(&self) -> &'static str {
        match *self {
            Value::String(..)     => "string",
            Value::Integer(..)    => "integer",
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

}
