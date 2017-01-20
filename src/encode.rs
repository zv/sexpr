extern crate rustc_serialize;
use self::rustc_serialize::Encodable;

use std::error::Error as StdError;
//use std::str::FromStr;
//use std::string;
use std::{char, f64, fmt, io, str};
use Sexp;

/// Shortcut function to encode a `T` into a JSON `String`
pub fn encode<T: Encodable>(object: &T) -> EncodeResult<String> {
    let mut s = String::new();
    {
        let mut encoder = Encoder::new(&mut s);
        try!(object.encode(&mut encoder));
    }
    Ok(s)
}


impl Encodable for Sexp {
    fn encode<S: rustc_serialize::Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        match *self {
            Sexp::Symbol(ref v) => v.encode(e),
            Sexp::String(ref v) => v.encode(e),
            Sexp::Keyword(ref v) => v.encode(e),

            Sexp::I64(v) => v.encode(e),
            Sexp::U64(v) => v.encode(e),
            Sexp::F64(v) => v.encode(e),

            Sexp::Boolean(v) => v.encode(e),

            Sexp::Pair(ref car, _) => car.encode(e),
            Sexp::List(ref v) => v.encode(e)
        }
    }
}


#[derive(Copy, Debug)]
pub enum EncoderError {
    FmtError(fmt::Error),
    BadHashmapKey,
}

impl PartialEq for EncoderError {
    fn eq(&self, other: &EncoderError) -> bool {
        match (*self, *other) {
            (EncoderError::FmtError(_), EncoderError::FmtError(_)) => true,
            (EncoderError::BadHashmapKey, EncoderError::BadHashmapKey) => true,
            _ => false,
        }
    }
}

impl Clone for EncoderError {
    fn clone(&self) -> Self { *self }
}

impl StdError for EncoderError {
    fn description(&self) -> &str { "encoder error" }
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<fmt::Error> for EncoderError {
    fn from(err: fmt::Error) -> EncoderError { EncoderError::FmtError(err) }
}


pub type EncodeResult<T> = Result<T, EncoderError>;

macro_rules! emit_enquoted_if_mapkey {
    ($enc:ident,$e:expr) => {
        if $enc.is_emitting_map_key {
            try!(write!($enc.writer, "\"{}\"", $e));
            Ok(())
        } else {
            try!(write!($enc.writer, "{}", $e));
            Ok(())
        }
    }
}


fn escape_str(wr: &mut fmt::Write, v: &str) -> EncodeResult<()> {
    // try!(wr.write_str("\""));

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write_str(&v[start..i]));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != v.len() {
        try!(wr.write_str(&v[start..]));
    }

    // try!(wr.write_str("\""));
    Ok(())
}

fn escape_char(writer: &mut fmt::Write, v: char) -> EncodeResult<()> {
    let mut buf = [0; 4];
    // let _ = write!(&mut &mut buf[..], "{}", v);
    let buf = unsafe { str::from_utf8_unchecked(&buf[..v.len_utf8()]) };
    escape_str(writer, buf)
}

fn fmt_number_or_null(v: f64) -> String {
    use std::num::FpCategory::{Nan, Infinite};

    match v.classify() {
        Nan | Infinite => "null".to_string(),
        _ => {
            let s = v.to_string();
            if s.contains(".") {s} else {s + ".0"}
        }
    }
}


impl<'a> Encoder<'a> {
    /// Creates a new encoder whose output will be written in compact
    /// JSON to the specified writer
    pub fn new(writer: &'a mut fmt::Write) -> Encoder<'a> {
        Encoder {
            writer: writer,
            is_emitting_map_key: false,
        }
    }
}


/// A structure for implementing serialization to S-expressions.
pub struct Encoder<'a> {
    writer: &'a mut (fmt::Write+'a),
    is_emitting_map_key: bool,
}


impl<'a> rustc_serialize::Encoder for Encoder<'a> {
    type Error = EncoderError;

    fn emit_nil(&mut self) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        try!(write!(self.writer, "()"));
        Ok(())
    }

    fn emit_usize(&mut self, v: usize) -> EncodeResult<()>  { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult<()>        { emit_enquoted_if_mapkey!(self, v) }
    fn emit_isize(&mut self, v: isize) -> EncodeResult<()>  { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i64(&mut self, v: i64) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i32(&mut self, v: i32) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i16(&mut self, v: i16) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i8(&mut self, v: i8) -> EncodeResult<()>        { emit_enquoted_if_mapkey!(self, v) }


    fn emit_f64(&mut self, v: f64) -> EncodeResult<()> {
        emit_enquoted_if_mapkey!(self, fmt_number_or_null(v))
    }
    fn emit_f32(&mut self, v: f32) -> EncodeResult<()> {
        self.emit_f64(v as f64)
    }


    fn emit_bool(&mut self, v: bool) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if v {
            try!(write!(self.writer, "#t"));
        } else {
            try!(write!(self.writer, "#f"));
        }
        Ok(())
    }

    fn emit_char(&mut self, v: char) -> EncodeResult<()> { escape_char(self.writer, v) }
    fn emit_str(&mut self, v: &str) -> EncodeResult<()> { escape_str(self.writer, v) }

    fn emit_enum<F>(&mut self, _name: &str, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        f(self)
    }

    fn emit_enum_variant<F>(&mut self, name: &str, _id: usize, cnt: usize, f: F)
                            -> EncodeResult<()> where F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        // enums are encoded as strings or objects
        // Bunny => "Bunny"
        // Kangaroo(34,"William") => ((variant kangaroo) (fields (34 "William)))
        // Kangaroo(34,"William") => ((variant . kangaroo) (fields . (34 "William)))
        if cnt == 0 {
            escape_str(self.writer, name)
        } else {
            if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
            try!(write!(self.writer, "((variant "));
            // We could write a 'dot' to allow a more unambiguous s-expression.
            try!(escape_str(self.writer, name));
            try!(write!(self.writer, ") "));

            try!(f(self)); // Encode the sub-sexpression's fields

            try!(write!(self.writer, ")"));

            Ok(())
        }
    }

    fn emit_enum_variant_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, " "));
        }
        f(self)
    }


    fn emit_enum_struct_variant<F>(&mut self,
                                   name: &str,
                                   id: usize,
                                   cnt: usize,
                                   f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         idx: usize,
                                         f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_enum_variant_arg(idx, f)
    }


    fn emit_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if len == 0 {
            try!(write!(self.writer, "(())"));
        } else {
            try!(write!(self.writer, "("));
            try!(f(self));
            try!(write!(self.writer, ")"));
        }
        Ok(())
    }

    fn emit_struct_field<F>(&mut self, name: &str, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, " "));
        }
        try!(write!(self.writer, "("));
        try!(escape_str(self.writer, name));
        try!(write!(self.writer, " "));
        f(self);
        try!(write!(self.writer, ")"));
        Ok(())
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq_elt(idx, f)
    }

    fn emit_option<F>(&mut self, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_nil()
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        f(self)
    }


    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if len == 0 {
            try!(write!(self.writer, "()"));
        } else {
            try!(write!(self.writer, "( "));
            try!(f(self));
            try!(write!(self.writer, " )"));
        }
        Ok(())
    }

    fn emit_seq_elt<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, " "));
        }
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if len == 0 {
            try!(write!(self.writer, "(())"));
        } else {
            try!(write!(self.writer, "("));
            try!(f(self));
            try!(write!(self.writer, ")"));
        }
        Ok(())
    }

    fn emit_map_elt_key<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, " "));
        }
        self.is_emitting_map_key = true;
        try!(write!(self.writer, "("));
        try!(f(self));
        self.is_emitting_map_key = false;
        Ok(())
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }

        try!(write!(self.writer, " . "));
        f(self);
        try!(write!(self.writer, ")"));
        Ok(())
    }

}
