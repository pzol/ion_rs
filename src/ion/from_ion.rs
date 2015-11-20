use ion::Value;

pub trait FromIon : Sized {
    type Err;
    fn from_ion(&Value) -> Result<Self, Self::Err>;
}

impl FromIon for String {
    type Err = ();
    fn from_ion(value: &Value) -> Result<Self, Self::Err> {
        value.as_string().map(|s| s.to_owned()).ok_or(())
    }
}

impl FromIon for Option<String> {
    type Err = ();
    fn from_ion(value: &Value) -> Result<Self, Self::Err> {
        value.as_string().map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_owned())
            }
        }).ok_or(())
    }
}

macro_rules! from_ion_value_int_impl {
     ($($t:ty)*) => {$(
         impl FromIon for $t {
             type Err = ::std::num::ParseIntError;
             fn from_ion(value: &Value) -> Result<Self, Self::Err> {
                match value.as_string() {
                    Some(s) => Ok(try!(s.parse())),
                    None => "".parse()
                }
             }
         }
     )*}
 }

from_ion_value_int_impl!{ isize i8 i16 i32 i64 usize u8 u16 u32 u64 }

impl FromIon for bool {
    type Err = ::std::str::ParseBoolError;
    fn from_ion(value: &Value) -> Result<Self, Self::Err> {
        match value.as_string() {
            Some(s) => Ok(try!(s.parse())),
            None => "".parse()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use ion::{ FromIon, Value };

    #[test]
    fn string() {
        let v = Value::String("foo".to_owned());
        let s = String::from_ion(&v).unwrap();
        assert_eq!("foo", s);
        let s : String = v.from_ion().unwrap();
        assert_eq!("foo", s);
    }

    #[test]
    fn option_string() {
        let v = Value::from_str("foo").unwrap();
        let os : Option<String> = v.from_ion().unwrap();
        assert_eq!(Some("foo".to_owned()), os);

        let v = Value::from_str("").unwrap();
        let os : Option<String> = v.from_ion().unwrap();
        assert_eq!(None, os);
    }

    #[test]
    fn u32() {
        let v = Value::from_str("16").unwrap();
        let u : u32 = v.from_ion().unwrap();
        assert_eq!(16, u);
    }

    #[test]
    fn bool() {
        let v = Value::from_str("true").unwrap();
        let u : bool = v.from_ion().unwrap();
        assert_eq!(true, u);

        let v = Value::from_str("false").unwrap();
        let u : bool = v.from_ion().unwrap();
        assert_eq!(false, u);

        let v = Value::from_str("").unwrap();
        let u : Result<bool, _> = v.from_ion();
        assert!(u.is_err());

    }
}
