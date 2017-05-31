// Copyright 2017 Zephyr Pellerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{self, Serialize};
use error::{Error, ErrorCode};
use number::Number;
use sexp::{Sexp, to_value};


impl Serialize for Sexp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ::serde::Serializer,
    {
        match *self {
            Sexp::Nil => serializer.serialize_unit(),
            Sexp::Boolean(b) => serializer.serialize_bool(b),
            Sexp::Number(ref n) => n.serialize(serializer),
            Sexp::Atom(ref atom) => serializer.serialize_str(&atom.as_string()),
            Sexp::List(ref v) => v.serialize(serializer),
            Sexp::Pair(_, _) => {
                unimplemented!()
            },
            // Sexp::Pair(Some(_), None) => unimplemented!(),
            // Sexp::Pair(None, Some(_)) => unimplemented!(),
            // Sexp::Pair(None, None)  => unimplemented!(),
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
    // XXX TODO
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
        Ok(Sexp::Number(value.into()))
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
        Ok(Sexp::Number(value.into()))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Sexp, Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Sexp, Error> {
        Ok(Number::from_f64(value).map_or(Sexp::Nil, Sexp::Number))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Sexp, Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Sexp, Error> {
        Ok(Sexp::Atom(value.to_owned().into()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Sexp, Error> {
        let vec = value.iter().map(|&b| Sexp::Number(b.into())).collect();
        Ok(Sexp::List(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Sexp, Error> {
        Ok(Sexp::Nil)
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
        _variant: &'static str,
        _value: &T,
    ) -> Result<Sexp, Error>
        where
        T: Serialize,
    {
        unimplemented!()
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
        unimplemented!()
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
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        unimplemented!()
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
        Ok(Sexp::List(self.vec))
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
        unimplemented!()
    }
}

#[doc(hidden)]
pub struct SerializeMap {
    next_key: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        match try!(to_value(&key)) {
            Sexp::Atom(a) => self.next_key = Some(a.as_string()),
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
        unimplemented!()
    }

    fn end(self) -> Result<Sexp, Error> {
        unimplemented!()
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

#[doc(hidden)]
pub struct SerializeStructVariant {
    name: String,
    values: Vec<Sexp>,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Sexp;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
        where
        T: Serialize,
    {
        self.values.push(
            Sexp::new_entry(key, to_value(&value).ok().unwrap_or(Sexp::Nil))
        );
        Ok(())
    }

    fn end(self) -> Result<Sexp, Error> {
        Ok(Sexp::new_entry(self.name, Sexp::List(self.values)))
    }
}
