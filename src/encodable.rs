//! sexpr::encode is responsible for compile-time type reflection, codegen and
//! appropriate formatting.
//! EncodeSyntax is the (buggy) compiler plugin allowing derived traits to
//! encode into s-expressions.
//! Encodeable contains the s-expression formatting details necessary to
//! implement the `#[derive(Encodable)]` (and `Decodable`, in decodable.rs)
//! extension.
//!
//! For example, a type like:
//!
//! ```ignore
//! #[derive(RustcEncodable, RustcDecodable)]
//! struct Node { id: usize }
//! ```
//!
//! would generate two implementations like:
//!
//! ```ignore
//! impl<S: Encoder<E>, E> Encodable<S, E> for Node {
//!     fn encode(&self, s: &mut S) -> Result<(), E> {
//!         s.emit_struct("Node", 1, |this| {
//!             this.emit_struct_field("id", 0, |this| {
//!                 Encodable::encode(&self.id, this)
//!                 /* this.emit_usize(self.id) can also be used */
//!             })
//!         })
//!     }
//! }
//!
//! impl<D: Decoder<E>, E> Decodable<D, E> for Node {
//!     fn decode(d: &mut D) -> Result<Node, E> {
//!         d.read_struct("Node", 1, |this| {
//!             match this.read_struct_field("id", 0, |this| Decodable::decode(this)) {
//!                 Ok(id) => Ok(Node { id: id }),
//!                 Err(e) => Err(e),
//!             }
//!         })
//!     }
//! }
//! ```
//!
//! Other interesting scenarios are when the item has type parameters or
//! references other non-built-in types.  A type definition like:
//!
//! ```ignore
//! #[derive(RustcEncodable, RustcDecodable)]
//! struct Spanned<T> { node: T, span: Span }
//! ```
//!
//! would yield functions like:
//!
//! ```ignore
//! impl<
//!     S: Encoder<E>,
//!     E,
//!     T: Encodable<S, E>
//! > Encodable<S, E> for Spanned<T> {
//!     fn encode(&self, s: &mut S) -> Result<(), E> {
//!         s.emit_struct("Spanned", 2, |this| {
//!             this.emit_struct_field("node", 0, |this| self.node.encode(this))
//!                 .unwrap();
//!             this.emit_struct_field("span", 1, |this| self.span.encode(this))
//!         })
//!     }
//! }
//!
//! impl<
//!     D: Decoder<E>,
//!     E,
//!     T: Decodable<D, E>
//! > Decodable<D, E> for Spanned<T> {
//!     fn decode(d: &mut D) -> Result<Spanned<T>, E> {
//!         d.read_struct("Spanned", 2, |this| {
//!             Ok(Spanned {
//!                 node: this.read_struct_field("node", 0, |this| Decodable::decode(this))
//!                     .unwrap(),
//!                 span: this.read_struct_field("span", 1, |this| Decodable::decode(this))
//!                     .unwrap(),
//!             })
//!         })
//!     }
//! }
//! ```
extern crate rustc_serialize;
use self::rustc_serialize::Encodable;
use std::{char, f64, fmt, str};
use std::io::Write;
use error::EncoderError;
use Sexp;

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
    try!(wr.write_str("\""));

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

    try!(wr.write_str("\""));
    Ok(())
}

fn escape_char(writer: &mut fmt::Write, v: char) -> EncodeResult<()> {
    let mut buf = [0; 4];
    let _ = write!(&mut &mut buf[..], "{}", v);
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

    /// Emit a nil value.
    fn emit_nil(&mut self) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        try!(write!(self.writer, "()"));
        Ok(())
    }

    // Various numeric types
    /// Emit a usize value
    fn emit_usize(&mut self, v: usize) -> EncodeResult<()>  { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a u64 value
    fn emit_u64(&mut self, v: u64) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a u32 value
    fn emit_u32(&mut self, v: u32) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a u16 value
    fn emit_u16(&mut self, v: u16) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a u8 value
    fn emit_u8(&mut self, v: u8) -> EncodeResult<()>        { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a isize value
    fn emit_isize(&mut self, v: isize) -> EncodeResult<()>  { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a i64 value
    fn emit_i64(&mut self, v: i64) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a i32 value
    fn emit_i32(&mut self, v: i32) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a i16 value
    fn emit_i16(&mut self, v: i16) -> EncodeResult<()>      { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a i8 value
    fn emit_i8(&mut self, v: i8) -> EncodeResult<()>        { emit_enquoted_if_mapkey!(self, v) }
    /// Emit a f64 value
    fn emit_f64(&mut self, v: f64) -> EncodeResult<()> {
        emit_enquoted_if_mapkey!(self, fmt_number_or_null(v))
    }
    /// Emit a f32 value
    fn emit_f32(&mut self, v: f32) -> EncodeResult<()> {
        self.emit_f64(v as f64)
    }

    /// Emit a boolean (#t / #f)
    fn emit_bool(&mut self, v: bool) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if v {
            try!(write!(self.writer, "#t"));
        } else {
            try!(write!(self.writer, "#f"));
        }
        Ok(())
    }

    /// Emit a char value.
    fn emit_char(&mut self, v: char) -> EncodeResult<()> { escape_char(self.writer, v) }

    /// Emit a string value.
    fn emit_str(&mut self, v: &str) -> EncodeResult<()> { escape_str(self.writer, v) }

    fn emit_enum<F>(&mut self, _name: &str, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        f(self)
    }

    /// Emit a enumeration variant value with no or unnamed data.
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
        try!(f(self));
        try!(write!(self.writer, ")"));
        Ok(())
    }

    /// Serialize a tuple
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(sexpr::encode(&(1, 2, 3)).unwrap(), "(1 2 3)");
    /// ```
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

    /// Serialize a tuple struct
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(RustcEncodable, Debug)]
    /// struct TupleStruct(i32, i32, i32);
    /// let ts = TupleStruct(1, 2, 3);
    /// assert_eq!(sexpr::encode(&ts).unwrap(), "((_field0 1) (_field1 2) (_field2 3))");
    /// ```
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

    /// Serialize a nullable type.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(RustcEncodable)]
    /// struct Opt {
    ///     value: Option<i32>
    /// };
    /// let some = Opt { value: Some(1) };
    /// let none = Opt { value: None };
    /// assert_eq!(sexpr::encode(&some).unwrap(), "((value 1))")
    /// assert_eq!(sexpr::encode(&none).unwrap(), "((value ()))")
    /// ```
    fn emit_option<F>(&mut self, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        f(self)
    }
    /// Serialize `None`
    fn emit_option_none(&mut self) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_nil()
    }
    /// Serialize `Some(VALUE)`
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

    /// Serialize map-like structures
    ///
    /// # Examples
    ///
    /// ```
    /// let ht = HashMap::new();
    /// ht.insert('a', 1);
    /// ht.insert('b', 2);
    /// assert_eq!(sexpr::encode(&ht).unwrap(), "((a . 1) (b . 2))")
    /// ```
    ///
    /// ## Warning
    ///
    /// There is no native way to represent maps in S-expression notation
    /// without introducing another level of abstraction. We use the very common
    /// S-expression 'pair syntax' extension to accommodate this. If your
    /// S-expression variant cannot handle pairs, please convert to proplists
    /// ahead of time.
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

    /// Serialize map key
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

    /// Serialize map value
    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }

        try!(write!(self.writer, " . "));
        try!(f(self));
        try!(write!(self.writer, ")"));
        Ok(())
    }

}
