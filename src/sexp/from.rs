// Copyright 2017 Zephyr Pellerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use super::Sexp;
use map::Map;
use number::Number;

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Sexp {
                fn from(n: $ty) -> Self {
                    Sexp::Number(n.into())
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
}

impl From<f32> for Sexp {
    /// Convert 32-bit floating point number to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let f: f32 = 13.37;
    /// let x: Sexp = f.into();
    /// # }
    /// ```
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Sexp {
    /// Convert 64-bit floating point number to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let f: f64 = 13.37;
    /// let x: Sexp = f.into();
    /// # }
    /// ```
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Sexp::Null, Sexp::Number)
    }
}

impl From<bool> for Sexp {
    /// Convert boolean to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let b = false;
    /// let x: Sexp = b.into();
    /// # }
    /// ```
    fn from(f: bool) -> Self {
        Sexp::Boolean(f)
    }
}

impl From<String> for Sexp {
    /// Convert `String` to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let s: String = "lorem".to_string();
    /// let x: Sexp = s.into();
    /// # }
    /// ```
    fn from(f: String) -> Self {
        Sexp::String(f)
    }
}

impl<'a> From<&'a str> for Sexp {
    /// Convert string slice to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let s: &str = "lorem";
    /// let x: Sexp = s.into();
    /// # }
    /// ```
    fn from(f: &str) -> Self {
        Sexp::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Sexp {
    /// Convert copy-on-write string to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Sexp = s.into();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_string());
    /// let x: Sexp = s.into();
    /// # }
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        Sexp::String(f.to_string())
    }
}

impl From<Map<String, Sexp>> for Sexp {
    /// Convert map (with string keys) to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::{Map, Sexp};
    ///
    /// let mut m = Map::new();
    /// m.insert("Lorem".to_string(), "ipsum".into());
    /// let x: Sexp = m.into();
    /// # }
    /// ```
    fn from(f: Map<String, Sexp>) -> Self {
        unimplemented!()
    }
}

impl<T: Into<Sexp>> From<Vec<T>> for Sexp {
    /// Convert a `Vec` to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Sexp = v.into();
    /// # }
    /// ```
    fn from(f: Vec<T>) -> Self {
        Sexp::List(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Sexp>> From<&'a [T]> for Sexp {
    /// Convert a slice to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Sexp = v.into();
    /// # }
    /// ```
    fn from(f: &'a [T]) -> Self {
        Sexp::List(f.into_iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Sexp>> ::std::iter::FromIterator<T> for Sexp {
    /// Convert an iteratable type to a `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Sexp = v.collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use sexpr::Sexp;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Sexp = v.into_iter().collect();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// use std::iter::FromIterator;
    /// use sexpr::Sexp;
    ///
    /// let x: Sexp = Sexp::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// # }
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<Sexp> = iter.into_iter().map(|x| x.into()).collect();

        Sexp::List(vec)
    }
}
