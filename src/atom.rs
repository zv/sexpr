// Copyright 2017 Zephyr Pellerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use error::Error;
use serde::de::{self, Visitor};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::fmt::{self, Debug, Display};

use std::borrow::Cow;

/// Represents a Sexp atom, whether symbol, keyword or string.
#[derive(Clone, PartialEq)]
pub struct Atom {
    a: A
}

#[cfg_attr(feature = "cargo-clippy", allow(enum_variant_names))]
#[derive(Clone, Debug, PartialEq)]
enum A {
    Symbol(String),
    Keyword(String),
    String(String)
}

impl Atom {
    pub fn is_symbol(&self) -> bool {
        match self.a {
            A::Symbol(_) => true,
            A::Keyword(_) => false,
            A::String(_) => false,
        }
    }

    pub fn is_keyword(&self) -> bool {
        match self.a {
            A::Symbol(_) => false,
            A::Keyword(_) => true,
            A::String(_) => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self.a {
            A::Symbol(_) => false,
            A::Keyword(_) => false,
            A::String(_) => true,
        }
    }

    /// Returns an Atom appropriate for it's contents.
    ///
    /// Criteria for discriminating variants can be configured as appropriate.
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern_crate sexpr;
    /// #
    /// # use sexpr::Atom;
    /// # fn main() {
    /// assert!(Atom::Keyword("keyword"), Atom::discriminate("#:keyword"))
    /// assert!(Atom::Symbol("symbol"), Atom::discriminate("symbol"))
    /// assert!(Atom::String("\"string\""), Atom::discriminate("\"string\""))
    /// ```
    pub fn discriminate(s: String) -> Self {
        if s.starts_with("#:") {
            let (_, keyword) = s.split_at(2);
            Atom { a: A::Keyword(String::from(keyword)) }
        } else if (s.starts_with('"') && s.ends_with('"'))
               || (s.starts_with("'") && s.ends_with("'")) {
            Atom { a: A::String(s)}
        } else {
            Atom { a: A::Symbol(s) }
        }
    }

    #[inline]
    pub fn from_str(s: &str) -> Self {
        Atom::discriminate(String::from(s))
    }

    #[inline]
    pub fn from_string(s: String) -> Self {
        Atom::discriminate(s)
    }

    #[inline]
    pub fn as_str<'a>(&'a self) -> &'a str {
        match self.a {
            A::Symbol(ref s) => s,
            A::Keyword(ref s) => s,
            A::String(ref s) => s,
        }
    }

    #[inline]
    pub fn as_string(&self) -> String {
        let s = match self.a {
            A::Symbol(ref s)  => s,
            A::Keyword(ref s) => s,
            A::String(ref s)  => s,
        };

        s.clone()
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.a {
            A::Symbol(ref s) => Display::fmt(&s, formatter),
            A::Keyword(ref s) => Display::fmt(&s, formatter),
            A::String(ref s) => Display::fmt(&s, formatter),
        }
    }
}

impl Debug for Atom {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.a, formatter)
    }
}


impl Serialize for Atom {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
        S: Serializer,
    {
        match self.a {
            A::Symbol(ref s) => serializer.serialize_str(s),
            A::Keyword(ref s) => serializer.serialize_str(s),
            A::String(ref s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for Atom {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Atom, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AtomVisitor;

        impl<'de> Visitor<'de> for AtomVisitor {
            type Value = Atom;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an atom")
            }

            // #[inline]
            // fn visit_str<E>(self, value: &str) -> Result<Atom, E>
            // {
            //     self.visit_string(String::from(value))
            // }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Atom, E>
            where
                E: de::Error,
            {
                Ok(Atom::from_string(value))
            }
        }

        deserializer.deserialize_any(AtomVisitor)
    }
}


impl<'de> Deserializer<'de> for Atom {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
        V: Visitor<'de>,
    {
        match self.a {
            A::Symbol(s) => visitor.visit_string(s),
            A::Keyword(s) => visitor.visit_string(s),
            A::String(s) => visitor.visit_string(s),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}


impl<'de, 'a> Deserializer<'de> for &'a Atom {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
        where
        V: Visitor<'de>,
    {
        match self.a {
            A::Symbol(ref s) => visitor.visit_string(s.clone()),
            A::Keyword(ref s) => visitor.visit_string(s.clone()),
            A::String(ref s) => visitor.visit_string(s.clone()),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}

impl From<String> for Atom {
    #[inline]
    fn from(s: String) -> Self {
        Atom::from_string(String::from(s))
    }
}


impl<'a> From<&'a str> for Atom {
    #[inline]
    fn from(s: &'a str) -> Self {
        Atom::from_str(s)
    }
}

impl<'a> From<Cow<'a, str>> for Atom {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        Atom::from_string(s.to_string())
    }
}
