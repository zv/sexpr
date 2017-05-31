// Copyright 2017 Zephyr "zv" Pellerin. See the COPYRIGHT
// file at the top-level directory of this distribution
//
// Licensed under the MIT License, <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! The Sexp enum, a loosely typed way of representing any valid S-expression value.
//!
//! # Constructing S-expression
//!
//! Serde S-expression provides a [`sexp!` macro][macro] to build `sexpr::Sexp`
//! objects with very natural S-expression syntax. In order to use this macro,
//! `sexpr` needs to be imported with the `#[macro_use]` attribute.
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate sexpr;
//!
//! fn main() {
//!     // The type of `john` is `sexpr::Sexp`
//!     let john = sexp!((
//!       ("name" . "John Doe")
//!       ("age" . 43)
//!       ("phones" . (
//!         ("+44 1234567")
//!         ("+44 2345678")
//!       ))
//!     ));
//!
//!     println!("first phone number: {}", john["phones"][0]);
//!
//!     // Convert to a string of S-expression and print it out
//!     println!("{}", john.to_string());
//! }
//! ```
//!
//! The `Sexp::to_string()` function converts a `sexpr::Value` into a `String` of
//! S-expression text. A string of S-expression data can be parsed into a
//! `sexpr::Sexp` by the [`sexpr::from_str`][from_str] function. There is also
//! [`from_slice`][from_slice] for parsing from a byte slice &[u8] and
//! [`from_reader`][from_reader] for parsing from any `io::Read` like a File or a
//! TCP stream.
//!
//! ```rust,ignore
//! extern crate sexpr;
//!
//! use sexpr::{Sexp, Error};
//!
//! fn untyped_example() -> Result<(), Error> {
//!     // Some S-expression input data as a &str. Maybe this comes from the user.
//!     let data = r#"(
//!       ("name" . "John Doe")
//!       ("age" . 43)
//!       ("phones" . (
//!         ("+44 1234567")
//!         ("+44 2345678")
//!       ))
//!     )"#;
//!
//!     // Parse the string of data into sexpr::Sexp.
//!     let v: Sexp = sexpr::from_str(data)?;
//!
//!     // Access parts of the data by indexing with square brackets.
//!     println!("Please call {} at the number {}", v["name"], v["phones"][0]);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     untyped_example().unwrap();
//! # }
//! ```
//!
use std::str;
use std::string::String;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use error::Error;
pub use number::Number;
pub use atom::Atom;

mod index;
pub use self::index::Index;

use self::ser::Serializer;

// Rather than having a specialized 'nil' atom, we save space by letting `None`
// here indicates 'nil'
type SexpPtr = Box<Sexp>;
type ConsCell = Option<SexpPtr>;

/// Represents any valid S-expression value.
///
/// See the `sexpr::sexp` module documentation for usage examples.
#[derive(PartialEq, Clone, Debug)]
pub enum Sexp {
    /// Represents a S-expression nil value.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let v = sexpr!(#nil);
    /// # }
    /// ```
    Nil,

    /// Represents a S-expression string, symbol or keyword.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let s = sexp!("string");
    /// let y = sexp!(symbol);
    /// let k = sexp!(#:keyword);
    /// # }
    /// ```
    Atom(Atom),

    /// Represents a S-expression number, whether integer or floating point.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let v = sexp!(12.5);
    /// # }
    /// ```
    Number(Number),

    /// Represents a S-expression boolean.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let v = sexp!(#t);
    /// # }
    /// ```
    Boolean(bool),

    /// Represents a S-expression cons-pair.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let v = sexp!((a . 1));
    /// # }
    /// ```
    Pair(ConsCell, ConsCell),

    /// Represents a S-expression list.
    ///
    /// This enum type is 'multi-function' at this point, possibly representing either
    /// a list of items or an associative list.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let v = sexp!((a b c));
    /// # }
    /// ```
    List(Vec<Sexp>),
}

mod ser;
mod de;


impl From<String> for Sexp {
    /// Convert `String` to `Sexp`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
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
        Sexp::Atom(Atom::from_string(f))
    }
}

impl Sexp {
    /// Return a new Sexp::Pair with a symbol key
    ///
    /// # Examples
    /// ```rust,ignore
    /// # extern crate sexpr;
    /// # fn main() {
    /// use sexpr::Sexp;
    /// let alist_1 = Sexp::new_entry("a", 1)
    /// # }
    /// ```
    pub fn new_entry<A: Into<Atom>, I: Into<Sexp>> (key: A, value: I) -> Sexp {
        Sexp::Pair(Some(Box::new(Sexp::Atom(key.into()))),
                   Some(Box::new(Sexp::from(value.into()))))
    }

    /// Index into a Sexp alist or list. A string index can be used to access a
    /// value in an alist, and a usize index can be used to access an element of an
    /// list.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is an array or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the array.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let object = sexp!(((A . 65) (B . 66) (C . 67)));
    /// assert_eq!(*object.get("A").unwrap(), sexp!(65));
    ///
    /// let array = sexp!((A B C));
    /// assert_eq!(*array.get(2).unwrap(), sexp!("C"));
    ///
    /// assert_eq!(array.get("A"), None);
    /// # }
    /// ```
    ///
    /// Square brackets can also be used to index into a value in a more concise
    /// way. This returns `Value::Null` in cases where `get` would have returned
    /// `None`.
    ///
    /// ```rust,ignore
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let object = sexp!((
    ///     (A . ("a" "á" "à"))
    ///     (B . ("b" "b́"))
    ///     (C . ("c" "ć" "ć̣" "ḉ"))
    /// ));
    /// assert_eq!(object["B"][0], sexp!("b"));
    ///
    /// assert_eq!(object["D"], sexp!(null));
    /// assert_eq!(object[0]["x"]["y"]["z"], sexp!(null));
    /// # }
    /// ```
    pub fn get<I: Index>(&self, index: I) -> Option<&Sexp> {
        unimplemented!()
    }

    // fn search_alist<S: ToString>(&self, key: S) -> Option<Sexp>
    // {
    //     let key = key.to_string();
    //     match *self {
    //         Sexp::List(ref elts) => {
    //             for elt in elts {
    //                 match *elt {
    //                     Sexp::Pair(Some(car), cdr) => {
    //                         if (*car).to_string() == key {
    //                             return cdr.and_then(|x| Some(*x));
    //                         }
    //                     }
    //                     _ => return None
    //                 }
    //             }
    //         }
    //     }

}

/// Convert a `T` into `sexpr::Sexp` which is an enum that can represent
/// any valid S-expression data.
///
/// ```rust,ignore
/// extern crate serde;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate sexpr;
///
/// use std::error::Error;
///
/// #[derive(Serialize)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn compare_values() -> Result<(), Box<Error>> {
///     let u = User {
///         fingerprint: "0xF9BA143B95FF6D82".to_owned(),
///         location: "Menlo Park, CA".to_owned(),
///     };
///
///     // The type of `expected` is `sexpr::Sexp`
///     let expected = sexp!((
///                            (fingerprint . "0xF9BA143B95FF6D82")
///                            (location . "Menlo Park, CA")
///                          ));
///
///     let v = sexpr::to_value(u).unwrap();
///     assert_eq!(v, expected);
///
///     Ok(())
/// }
/// #
/// # fn main() {
/// #     compare_values().unwrap();
/// # }
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
///
/// ```rust,ignore
/// extern crate sexpr;
///
/// use std::collections::BTreeMap;
///
/// fn main() {
///     // The keys in this map are vectors, not strings.
///     let mut map = BTreeMap::new();
///     map.insert(vec![32, 64], "x86");
///
///     println!("{}", sexpr::to_value(map).unwrap_err());
/// }
/// ```
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
// Taking by value is more friendly to iterator adapters, option and result
// consumers, etc.
pub fn to_value<T>(value: T) -> Result<Sexp, Error>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

/// Interpret a `sexpr::Sexp` as an instance of type `T`.
///
/// This conversion can fail if the structure of the Sexp does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Sexp
/// contains something other than a S-expression map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the S-expression map or some number is too big to fit in the expected primitive
/// type.
///
/// ```rust,ignore
/// #[macro_use]
/// extern crate sexpr;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// extern crate serde;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn main() {
///     // The type of `s` is `sexpr::Sexp`
///     let s = sexp!((
///                     (fingerprint . "0xF9BA143B95FF6D82")
///                     (location . "Menlo Park, CA")
///                   ));
///
///     let u: User = sexpr::from_value(s).unwrap();
///     println!("{:#?}", u);
/// }
/// ```
pub fn from_value<T>(value: Sexp) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}
