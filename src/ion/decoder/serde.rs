use serde::de;
use Value;
use super::{Decoder, DecodeError, DecodeErrorKind};

fn error(err: de::value::Error, ty: &'static str) -> DecodeError {
    match err {
        de::value::Error::SyntaxError => de::Error::syntax(ty),
        de::value::Error::EndOfStreamError => de::Error::end_of_stream(),
        de::value::Error::MissingFieldError(s) => {
            DecodeError {
                field: Some(s.to_string()),
                kind: DecodeErrorKind::ExpectedField(Some(ty)),
            }
        },
        de::value::Error::UnknownFieldError(s) => {
            DecodeError {
                field: Some(s.to_string()),
                kind: DecodeErrorKind::UnknownField,
            }
        },
    }
}

impl de::Deserializer for Decoder {
    type Error = DecodeError;

 fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, DecodeError>
        where V: de::Visitor
    {
        match self.value.take() {
            Some(Value::String(s))  => visitor.visit_string(s).map_err(|e| error(e, "string")),
            Some(Value::Integer(i)) => visitor.visit_i64(i).map_err(|e| error(e, "integer")),
            Some(Value::Float(f))   => visitor.visit_f64(f).map_err(|e| error(e, "float")),
            Some(Value::Boolean(b)) => visitor.visit_bool(b).map_err(|e| error(e, "boolean")),
            // Some(VAlue::Array(a))   => //TODO
            None => Err(de::Error::end_of_stream()),
        }
    }
}

impl de::Error for DecodeError {
    fn syntax(_: &str) -> DecodeError {
        DecodeError { field: None, kind: DecodeErrorKind::SyntaxError }
    }
    fn end_of_stream() -> DecodeError {
        DecodeError { field: None, kind: DecodeErrorKind::EndOfStream }
    }
    fn missing_field(name: &'static str) -> DecodeError {
        DecodeError {
            field: Some(name.to_string()),
            kind: DecodeErrorKind::ExpectedField(None),
        }
    }
    fn unknown_field(name: &str) -> DecodeError {
        DecodeError {
            field: Some(name.to_string()),
            kind: DecodeErrorKind::UnknownField,
        }
    }
}
