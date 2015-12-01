use rustc_serialize;

use super::{Decoder, DecodeError};
use super::DecodeErrorKind::*;
use Value;


impl rustc_serialize::Decoder for Decoder {
    type Error = DecodeError;

   fn read_nil(&mut self) -> Result<(), DecodeError> {
        match self.value {
            Some(Value::String(ref s)) if s.len() == 0 => {}
            Some(Value::String(..)) => return Err(self.err(NilTooLong)),
            ref found => return Err(self.mismatch("string", found)),
        }
        Ok(())
    }

    fn read_usize(&mut self) -> Result<usize, DecodeError> {
        self.read_i64().map(|i| i as usize)
    }

    fn read_u64(&mut self) -> Result<u64, DecodeError> {
        self.read_i64().map(|i| i as u64)
    }

    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        self.read_i64().map(|i| i as u32)
    }

    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        self.read_i64().map(|i| i as u16)
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        self.read_i64().map(|i| i as u8)
    }

    fn read_isize(&mut self) -> Result<isize, DecodeError> {
        self.read_i64().map(|i| i as isize)
    }

    fn read_i64(&mut self) -> Result<i64, DecodeError> {
        match self.value {
            Some(Value::Integer(i)) => { Ok(i) }
            Some(Value::String(ref s))  => {
                match s.parse() {
                    Ok(i) => Ok(i),
                    _ => Err(self.err(ExpectedType("integer", "string")))
                }
            }
            ref found => Err(self.mismatch("integer", found)),
        }
    }

    fn read_i32(&mut self) -> Result<i32, DecodeError> {
        self.read_i64().map(|i| i as i32)
    }

    fn read_i16(&mut self) -> Result<i16, DecodeError> {
        self.read_i64().map(|i| i as i16)
    }

    fn read_i8(&mut self) -> Result<i8, DecodeError> {
        self.read_i64().map(|i| i as i8)
    }

    fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.value {
            Some(Value::Boolean(b)) => { Ok(b) }
            Some(Value::String(ref s)) => match s.parse() {
                Ok(b) => Ok(b),
                _     => Err(self.err(ExpectedType("bool", "string")))
            },
            ref found => Err(self.mismatch("bool", found)),
        }
    }

    fn read_f64(&mut self) -> Result<f64, DecodeError> {
        unimplemented!()
        // match self.next() {
        //     Some(Value::Float(f)) => Ok(f),
        //     ref found => Err(self.mismatch("float", found)),
        // }
    }

    fn read_f32(&mut self) -> Result<f32, DecodeError> {
        self.read_f64().map(|f| f as f32)
    }

    fn read_char(&mut self) -> Result<char, DecodeError> {
        let ch = match self.value {
            Some(Value::String(ref s)) if s.chars().count() == 1 =>
                s.chars().next().unwrap(),
            ref found => return Err(self.mismatch("string", found)),
        };
        Ok(ch)
    }

    fn read_str(&mut self) -> Result<String, DecodeError> {
        match self.value.take() {
            Some(Value::String(s)) => Ok(s),
            found => return Err(self.mismatch("string", &found))
        }
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, _names: &[&str], mut _f: F)
        -> Result<T, DecodeError>
        where F: FnMut(&mut Decoder, usize) -> Result<T, DecodeError>
    {
        // // When decoding enums, this crate takes the strategy of trying to
        // // decode the current TOML as all of the possible variants, returning
        // // success on the first one that succeeds.
        // //
        // // Note that fidelity of the errors returned here is a little nebulous,
        // // but we try to return the error that had the relevant field as the
        // // longest field. This way we hopefully match an error against what was
        // // most likely being written down without losing too much info.
        // let mut first_error = None::<DecodeError>;
        // for i in 0..names.len() {
        //     let mut d = self.sub_decoder(self.next().clone(), "");
        //     match f(&mut d, i) {
        //         Ok(t) => {
        //             self.toml = d.toml;
        //             return Ok(t)
        //         }
        //         Err(e) => {
        //             if let Some(ref first) = first_error {
        //                 let my_len = e.field.as_ref().map(|s| s.len());
        //                 let first_len = first.field.as_ref().map(|s| s.len());
        //                 if my_len <= first_len {
        //                     continue
        //                 }
        //             }
        //             first_error = Some(e);
        //         }
        //     }
        // }
        // Err(first_error.unwrap_or_else(|| self.err(NoEnumVariants)))
        unimplemented!()
    }

    fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F)
        -> Result<T, DecodeError>
        where F: FnMut(&mut Decoder, usize) -> Result<T, DecodeError>
    {
        unimplemented!()
    }

    fn read_enum_struct_variant_field<T, F>(&mut self,
                                            _f_name: &str,
                                            _f_idx: usize,
                                            _f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        unimplemented!()
    }

    fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        match self.value {
            Some(Value::Array(..)) => {
                let ret = try!(f(self));
                match self.value {
                    Some(Value::Array(ref t)) if t.len() == 0 => {}
                    _ => return Ok(ret)
                }
                self.value.take();
                Ok(ret)
            },
            ref found => Err(self.mismatch("Array", found)),
        }
    }

    fn read_struct_field<T, F>(&mut self, f_name: &str, f_idx: usize, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        let value = match self.value {
            Some(Value::Array(ref ary)) => {
                match ary.get(f_idx) {
                    Some(v) => v,
                    None    => return Err(self.err(ExpectedField(Some("field"))))
                }
            },
            ref found => return Err(self.mismatch("Array", found)),
        };
        let mut d = self.sub_decoder(Some(value.clone()), f_name);
        let ret = try!(f(&mut d));

        Ok(ret)
    }

    fn read_tuple<T, F>(&mut self, tuple_len: usize, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        self.read_seq(move |d, len| {
            assert!(len == tuple_len,
                    "expected tuple of length `{}`, found tuple \
                         of length `{}`", tuple_len, len);
            f(d)
        })
    }
    fn read_tuple_arg<T, F>(&mut self, a_idx: usize, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        self.read_seq_elt(a_idx, f)
    }

    fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        unimplemented!()
    }
    fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        unimplemented!()
    }

    // Specialized types:
    fn read_option<T, F>(&mut self, mut f: F)
        -> Result<T, DecodeError>
        where F: FnMut(&mut Decoder, bool) -> Result<T, DecodeError>
    {
        let ret = match self.value {
            Some(Value::String(ref s)) if s.len() == 0 => false,
            Some(..) => true,
            None => false,
        };

        f(self, ret)
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder, usize) -> Result<T, DecodeError>
    {
        if let Some(Value::String(ref s)) = self.value {
            let ary : Vec<_> = s.split(|c| c == ',' || c == ';').map(|v| Value::String(v.to_owned())).collect();
            let len = ary.len();
            let mut d = self.sub_decoder(Some(Value::Array(ary)), "");
            // let ret = try!(f(&mut d));
            return f(&mut d, len);
        }

        let len = match self.value {
            Some(Value::Array(ref arr)) => arr.len(),
            None => 0,
            ref found => return Err(self.mismatch("array", found)),
        };
        let ret = try!(f(self, len));
        match self.value {
            Some(Value::Array(ref mut arr)) => {
                arr.retain(|slot| slot.as_integer() != Some(0));
                if arr.len() != 0 { return Ok(ret) }
            }
            _ => return Ok(ret)
        }
        self.value.take();
        Ok(ret)
    }

    fn read_seq_elt<T, F>(&mut self, idx: usize, f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        use ::std::mem;

        let value = match self.value {
            Some(Value::Array(ref mut arr)) => {
                mem::replace(&mut arr[idx], Value::Integer(0))
            }
            ref found => return Err(self.mismatch("array", found)),
        };
        let mut d = self.sub_decoder(Some(value), "");
        let ret = try!(f(&mut d));
        match d.value {
            Some(value) => match self.value {
                Some(Value::Array(ref mut arr)) => arr[idx] = value,
                _ => {}
            },
            _ => {}
        }
        Ok(ret)
    }

    fn read_map<T, F>(&mut self, _f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder, usize) -> Result<T, DecodeError>
    {
        unimplemented!()
        // let len = match self.toml {
        //     Some(Value::Table(ref table)) => table.len(),
        //     ref found => return Err(self.mismatch("table", found)),
        // };
        // let ret = try!(f(self, len));
        // self.toml.take();
        // Ok(ret)
    }
    fn read_map_elt_key<T, F>(&mut self, _idx: usize, _f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        unimplemented!()
        // match self.toml {
        //     Some(Value::Table(ref table)) => {
        //         match table.iter().skip(idx).next() {
        //             Some((key, _)) => {
        //                 let val = Value::String(key.to_string());
        //                 f(&mut self.sub_decoder(Some(val), key))
        //             }
        //             None => Err(self.err(ExpectedMapKey(idx))),
        //         }
        //     }
        //     ref found => Err(self.mismatch("table", found)),
        // }
    }
    fn read_map_elt_val<T, F>(&mut self, _idx: usize, _f: F)
        -> Result<T, DecodeError>
        where F: FnOnce(&mut Decoder) -> Result<T, DecodeError>
    {
        unimplemented!()
        // match self.toml {
        //     Some(Value::Table(ref table)) => {
        //         match table.iter().skip(idx).next() {
        //             Some((key, value)) => {
        //                 // XXX: this shouldn't clone
        //                 f(&mut self.sub_decoder(Some(value.clone()), key))
        //             }
        //             None => Err(self.err(ExpectedMapElement(idx))),
        //         }
        //     }
        //     ref found => Err(self.mismatch("table", found)),
        // }
    }

    fn error(&mut self, err: &str) -> DecodeError {
        DecodeError {
            field: self.cur_field.clone(),
            kind: ApplicationError(format!("{}", err))
        }
    }
}
