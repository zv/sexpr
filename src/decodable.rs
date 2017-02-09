extern crate rustc_serialize;
use self::rustc_serialize::Decodable;
use std::{char, f64, fmt, str};
use std::io::Write;
use error::DecoderError;
use error::DecoderError::*;
use Sexp;

pub type DecodeResult<T> = Result<T, DecoderError>;

/// A structure to decode Sexp to values in rust.
pub struct Decoder {
    stack: Vec<Sexp>,
}

impl Decoder {
    /// Creates a new decoder instance for decoding the specified Sexp value.
    pub fn new(sexp: Sexp) -> Decoder {
        Decoder { stack: vec![sexp] }
    }
}


impl Decoder {
    fn pop(&mut self) -> DecodeResult<Sexp> {
        match self.stack.pop() {
            Some(s) => Ok(s),
            None => Err(EOF),
        }
    }
}


macro_rules! expect {
    ($e:expr, Null) => ({
        match try!($e) {
            Sexp::List(elts) => Ok(()),
            other => Err(ExpectedError("Null".to_string(),
                                       format!("{}", other)))
        }
    });
    ($e:expr, $t:ident) => ({
        match try!($e) {
            Sexp::$t(v) => Ok(v),
            other => {
                Err(ExpectedError(stringify!($t).to_string(),
                                  format!("{}", other)))
            }
        }
    })
}

macro_rules! read_primitive {
    ($name:ident, $ty:ident) => {
        #[allow(unused_comparisons)]
        fn $name(&mut self) -> DecodeResult<$ty> {
            match try!(self.pop()) {
                Sexp::I64(i) => {
                    let other = i as $ty;
                    if i == other as i64 && (other > 0) == (i > 0) {
                        Ok(other)
                    } else {
                        Err(ExpectedError("Number".to_string(), i.to_string()))
                    }
                }
                Sexp::U64(u) => {
                    let other = u as $ty;
                    if u == other as u64 && other >= 0 {
                        Ok(other)
                    } else {
                        Err(ExpectedError("Number".to_string(), u.to_string()))
                    }
                }
                Sexp::F64(f) => {
                    Err(ExpectedError("Integer".to_string(), f.to_string()))
                }
                // re: #12967.. a type w/ numeric keys (ie HashMap<usize, V> etc)
                Sexp::String(s) => match s.parse() {
                    Ok(f)  => Ok(f),
                    Err(_) => Err(ExpectedError("Number".to_string(), s)),
                },
                value => {
                    Err(ExpectedError("Number".to_string(), value.to_string()))
                }
            }
        }
    }
}

impl rustc_serialize::Decoder for Decoder {
    type Error = DecoderError;

    fn read_nil(&mut self) -> DecodeResult<()> {
        expect!(self.pop(), Null)
    }

    read_primitive! { read_usize, usize }
    read_primitive! { read_u8, u8 }
    read_primitive! { read_u16, u16 }
    read_primitive! { read_u32, u32 }
    read_primitive! { read_u64, u64 }
    read_primitive! { read_isize, isize }
    read_primitive! { read_i8, i8 }
    read_primitive! { read_i16, i16 }
    read_primitive! { read_i32, i32 }
    read_primitive! { read_i64, i64 }

    fn read_f32(&mut self) -> DecodeResult<f32> {
        self.read_f64().map(|x| x as f32)
    }

    fn read_f64(&mut self) -> DecodeResult<f64> {
        match try!(self.pop()) {
            Sexp::I64(f) => Ok(f as f64),
            Sexp::U64(f) => Ok(f as f64),
            Sexp::F64(f) => Ok(f),
            Sexp::String(s) => {
                match s.parse() {
                    Ok(f)  => Ok(f),
                    Err(_) => Err(ExpectedError("Number".to_string(), s)),
                }
            },
            // We could match NAN for f64, but there's no strong mapping with
            // existing s-expression implementations.
            // Sexp::List([]) => Ok(f64::NAN),
            value => Err(ExpectedError("Number".to_string(), format!("{}", value)))
        }
    }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        expect!(self.pop(), Boolean)
    }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(ExpectedError("single character string".to_string(), format!("{}", s)))
    }

    fn read_str(&mut self) -> DecodeResult<String> {
        expect!(self.pop(), String)
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str],
                               mut f: F) -> DecodeResult<T>
        where F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        // find 'variant' name
        let name = match try!(self.pop()) {
            Sexp::String(s) => s,
            elts => {
                match elts.member(|sexp| **sexp == Sexp::symbol_from("variant")) {
                    Ok(Sexp::List(ref result)) => {
                        result[1].to_string()
                    },
                    sexp => {
                        return Err(ExpectedError("Must be list".to_string(), format!("{}", sexp.unwrap())))
                    }
                }
            }
        };
        // Lookup the index of the variant name.
        let idx = match names.iter().position(|n| *n == name) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        // We're good to go
        f(self, idx)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T, F>(&mut self,
                                         _name: &str,
                                         idx: usize,
                                         f: F)
                                         -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T, F>(&mut self, _name: &str, _len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        let value = try!(f(self));
        try!(self.pop());
        Ok(value)
    }

    fn read_struct_field<T, F>(&mut self,
                               name: &str,
                               _idx: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        let mut obj = self.pop().unwrap();
        println!("{:?}", obj);
        let value = match obj.remove_key(name.to_string()) {
            Err(err) => {
                // Add a Null and try to parse it as an Option<_>
                // to get None as a default value.
                self.stack.push(Sexp::List(vec![]));
                match f(self) {
                    Ok(x) => x,
                    Err(_) => return Err(MissingFieldError(name.to_string())),
                }
            },
            Ok(sexp) => {
                self.stack.push(sexp);
                try!(f(self))
            }
        };
        self.stack.push(obj);
        Ok(value)
    }

    fn read_tuple<T, F>(&mut self, tuple_len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq(move |d, len| {
            if len == tuple_len {
                f(d)
            } else {
                Err(ExpectedError(format!("Tuple{}", tuple_len), format!("Tuple{}", len)))
            }
        })
    }

    fn read_tuple_arg<T, F>(&mut self, idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T, F>(&mut self,
                               _name: &str,
                               len: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self,
                                   idx: usize,
                                   f: F)
                                   -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T, F>(&mut self, mut f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, bool) -> DecodeResult<T>,
    {
        match try!(self.pop()) {
            // check for 'nil'
            Sexp::List(ref elts) if elts.len() == 0  => f(self, false),
            value => { self.stack.push(value); f(self, true) }
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let array = self.pop();
        let mut len = 0;
        for v in array.into_iter().rev() {
            self.stack.push(v);
            len += 1;
        }
        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        f(self, 0)
        // let obj = self.pop();
        // // this is probably fqd
        // // let len = obj.len();
        // let mut len = 0;
        // for sexp in obj.into_iter() {
        //     self.stack.push(sexp.0);
        //     self.stack.push(Sexp::String(sexp[1]));
        //     len += 1;
        // }
        // f(self, len)

    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> DecoderError {
        ApplicationError(err.to_string())
    }
}
