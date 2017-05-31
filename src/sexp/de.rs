// Copyright 2017 Zephyr Pellerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::i64;
use std::io;
use std::slice;
use std::str;
use std::vec;

use serde;
use serde::de::{
    Deserialize,
    DeserializeSeed,
    Visitor,
    SeqAccess,
    MapAccess,
};

use error::Error;
use number::Number;
use atom::Atom;
use sexp::Sexp;

impl<'de> Deserialize<'de> for Sexp {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Sexp, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Sexp;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid Sexp value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Sexp, E> {
                Ok(Sexp::Boolean(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Sexp, E> {
                Ok(Sexp::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Sexp, E> {
                Ok(Sexp::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Sexp, E> {
                Ok(Number::from_f64(value).map_or(Sexp::Nil, Sexp::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Sexp, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Sexp, E> {
                Ok(Sexp::Atom(Atom::into_string(value)))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Sexp, E> {
                Ok(Sexp::Nil)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Sexp, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Sexp, E> {
                Ok(Sexp::Nil)
            }

            #[inline]
            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Sexp, D::Error>
                where
                D: serde::Deserializer<'de>,
            {
                /// XXX something about this feels wrong
                let result: String = try!(Deserialize::deserialize(deserializer));
                Ok(Sexp::Atom(Atom::into_symbol(String::from(result))))
            }


            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Sexp, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = try!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Sexp::List(vec))
            }

            fn visit_map<V>(self, _visitor: V) -> Result<Sexp, V::Error>
            where
                V: MapAccess<'de>,
            {
                unimplemented!()
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

struct WriterFormatter<'a, 'b: 'a> {
    inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> io::Write for WriterFormatter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        fn io_error<E>(_: E) -> io::Error {
            // Sexp does not matter because fmt::Debug and fmt::Display impls
            // below just map it to fmt::Error
            io::Error::new(io::ErrorKind::Other, "fmt error")
        }
        let s = try!(str::from_utf8(buf).map_err(io_error));
        try!(self.inner.write_str(s).map_err(io_error));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}


impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alternate = f.alternate();
        let mut wr = WriterFormatter { inner: f };
        if alternate {
            // {:#}
            super::super::ser::to_writer_pretty(&mut wr, self).map_err(|_| fmt::Error)
        } else {
            // {}
            super::super::ser::to_writer(&mut wr, self).map_err(|_| fmt::Error)
        }
    }
}


impl str::FromStr for Sexp {
    type Err = Error;
    fn from_str(s: &str) -> Result<Sexp, Error> {
        super::super::de::from_str(s)
    }
}

impl<'de> serde::Deserializer<'de> for Sexp {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Sexp::Nil => visitor.visit_unit(),
            Sexp::Boolean(v) => visitor.visit_bool(v),
            Sexp::Number(n) => n.deserialize_any(visitor),
            Sexp::Atom(a) => visitor.visit_string(a.as_string()),
            Sexp::Pair(_, _) => {
                unimplemented!()
            },
            Sexp::List(v) => {
                let len = v.len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = try!(visitor.visit_seq(&mut deserializer));
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Sexp::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Sexp>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Sexp>) -> Self {
        SeqDeserializer { iter: vec.into_iter() }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = try!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}


impl<'de> serde::Deserializer<'de> for &'de Sexp {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Sexp::Nil => visitor.visit_unit(),
            Sexp::Boolean(v) => visitor.visit_bool(v),
            Sexp::Number(ref n) => n.deserialize_any(visitor),
            Sexp::Atom(ref a) => visitor.visit_borrowed_str(a.as_str()),
            Sexp::Pair(_, _) => {
                unimplemented!()
            },
            Sexp::List(ref v) => {
                let len = v.len();
                let mut deserializer = SeqRefDeserializer::new(v);
                let seq = try!(visitor.visit_seq(&mut deserializer));
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Sexp::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}


struct SeqRefDeserializer<'de> {
    iter: slice::Iter<'de, Sexp>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [Sexp]) -> Self {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> serde::Deserializer<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = try!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

