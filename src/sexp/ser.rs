use serde::{self, Serialize};

use error::{Error, ErrorCode};
use map::Map;

use sexp::{Sexp, to_value};

impl Serialize for Sexp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ::serde::Serializer,
    {
        match *self {
            // Sexp::Null => serializer.serialize_unit(),
            Sexp::Boolean(b) => serializer.serialize_bool(b),
            Sexp::U64(num) => num.serialize(serializer),
            Sexp::I64(num) => num.serialize(serializer),
            Sexp::F64(num) => num.serialize(serializer),
            Sexp::Symbol(ref sym) => serializer.serialize_str(sym),
            Sexp::Keyword(ref sym) => serializer.serialize_str(sym),
            Sexp::String(ref s) => serializer.serialize_str(s),
            Sexp::List(ref v) => v.serialize(serializer),
            Sexp::Pair(Some(ref car), Some(ref cdr)) => {
                unimplemented!()
            },
            Sexp::Pair(Some(ref car), None) => unimplemented!(),
            Sexp::Pair(None, Some(ref cdr)) => unimplemented!(),
            Sexp::Pair(None, None)  => unimplemented!(),
        }
    }
}

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Sexp;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Sexp, Error> {
        Ok(Sexp::Boolean(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Sexp, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Sexp, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Sexp, Error> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Sexp, Error> {
        Ok(Sexp::I64(value.into()))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Sexp, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Sexp, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Sexp, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Sexp, Error> {
        Ok(Sexp::U64(value.into()))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Sexp, Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Sexp, Error> {
        Ok(Number::from_f64(value).map_or(List([]), Sexp::F64))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Sexp, Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Sexp, Error> {
        Ok(Sexp::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Sexp, Error> {
        let vec = value.iter().map(|&b| Sexp::U64(b.into())).collect();
        Ok(Sexp::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Sexp, Error> {
        Ok(Sexp::List([]))
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Sexp, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Sexp, Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Sexp, Error>
        where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Sexp, Error>
        where
        T: Serialize,
    {
        let mut values = Map::new();
        values.insert(String::from(variant), try!(to_value(&value)));
        Ok(Sexp::Object(values))
    }

    #[inline]
        fn serialize_none(self) -> Result<Sexp, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Sexp, Error>
        where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(SerializeVec { vec: Vec::with_capacity(len.unwrap_or(0)) })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(
            SerializeTupleVariant {
                name: String::from(variant),
                vec: Vec::with_capacity(len),
            },
        )
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(
            SerializeMap {
                map: Map::new(),
                next_key: None,
            },
        )
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(
            SerializeStructVariant {
                name: String::from(variant),
                map: Map::new(),
            },
        )
    }
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Vec<Sexp>,
}

#[doc(hidden)]
pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Sexp>,
}

#[doc(hidden)]
pub struct SerializeMap {
    map: Map<String, Sexp>,
    next_key: Option<String>,
}

#[doc(hidden)]
pub struct SerializeStructVariant {
    name: String,
    map: Map<String, Sexp>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        self.vec.push(try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Sexp, Error> {
        Ok(Sexp::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Sexp, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Sexp, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        self.vec.push(try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Sexp, Error> {
        let mut object = Map::new();
        object.insert(self.name, Sexp::Array(self.vec));
        Ok(Sexp::Object(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        match try!(to_value(&key)) {
            Sexp::String(s) => self.next_key = Some(s),
            Sexp::Number(n) => {
                if n.is_u64() || n.is_i64() {
                    self.next_key = Some(n.to_string())
                } else {
                    return Err(Error::syntax(ErrorCode::KeyMustBeAString, 0, 0));
                }
            }
            _ => return Err(Error::syntax(ErrorCode::KeyMustBeAString, 0, 0)),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Sexp, Error> {
        Ok(Sexp::Object(self.map))
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        try!(serde::ser::SerializeMap::serialize_key(self, key));
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Sexp, Error> {
        serde::ser::SerializeMap::end(self)
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        self.map
            .insert(String::from(key), try!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Sexp, Error> {
        let mut object = Map::new();

        object.insert(self.name, Sexp::Object(self.map));

        Ok(Sexp::Object(object))
    }
}
