// Copyright 2017 Zephyr "zv" Pellerin. See the COPYRIGHT
// file at the top-level directory of this distribution
//
// Licensed under the MIT License, <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! S-expression parsing and serialization
//!
//! # What are S-expressions?
//! # What are S-Expressions?
//!
//! S-expressions are data structures for representing complex data.  They
//! are either byte-strings ("octet-strings") or lists of simpler
//! S-expressions.  Here is a sample S-expression:
//!
//! Data types that can be encoded are varied, and can be expressed in multiple
//! different ways, depending on the parser's configuration.
//!
//! * `Symbol`: An identifier
//! * `I64`: equivalent to rust's `i64`
//! * `U64`: equivalent to rust's `u64`
//! * `F64`: equivalent to rust's `f64`
//! * `Boolean`: equivalent to rust's `bool`
//! * `String`: equivalent to rust's `String`
//! * `Cons`: A structure with two pointers 'car' and 'cdr'
//! * `Nil`: A special object, indicating an end of list or the lack of value.
//!
//! Composite structures are made through combination of these components
//!
//! # Simple Lists
//! ```ignore
//! (computer clock calendar)
//! ```
//!
//! ## Associated Lists
//!
//! ```ignore
//! (doctors  . ("Dr. Hargrove" "Dr. Steve" "Dr. Mischief" "Dr. Lizzie")
//!  teachers . ("Ms. Taya" "Mr. Smith" "Principle Weedle")
//!  sailors  . ("Ole Skippy" "Ravin' Dave" "Popeye"))
//! ```
//!
//! # Examples of use
//!
//! ```ignore
//! use std::collections::BTreeMap;
//!
//! let values = "((New York . Albany)
//!                (Oregon   . Salem)
//!                (Florida  . Miami)
//!                (California . Sacramento)
//!                (Colorado . Denver))";
//!
//! let sexp = Sexp::from_str(values);
//!
//! // Deserialize using 'sexpattern::decode'
//! let decoded: BTreeMap = sexpattern::decode(&sexp).unwrap();
//!

//! println!("Colorado's Capital is: {}", decoded.get("Colorado"))
//! ```
use std::fmt::Display;
use std::i64;
use std::str;
use std::string::String;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use std::rc::Rc;

use error::{Error, ErrorCode};
pub use number::Number;

mod index;
pub use self::index::Index;

use self::ser::Serializer;

// Rather than having a specialized 'nil' atom, we save space by letting `None`
// here indicates 'nil'
type SexpPtr = Box<Sexp>;
type ConsCell = Option<SexpPtr>;

/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(PartialEq, Clone, Debug)]
pub enum Sexp {
    /// A special nil symbol
    Nil,
    /// A symbol or alist key
    Symbol(String),
    /// A UTF-8 String
    String(String),
    /// A keyword consists of a `:` (colon) followed by valid symbol characters.
    Keyword(String),
    /// Represents a Sexp number
    Number(Number),
    Boolean(bool),
    /// A classic 'cons cell' structure whose elts are themselves cons-cells.
    Pair(ConsCell, ConsCell),
    List(Vec<Sexp>),
}

mod ser;
mod de;

impl Sexp {
    /// Return a new Sexp::Pair with a symbol key
    ///
    /// # Examples
    /// ```rust
    /// # extern crate sexpr;
    /// # fn main() {
    /// use sexpr::Sexp;
    /// let alist_1 = Sexp::new_entry("a", 1)
    /// # }
    /// ```
    pub fn new_entry<S: ToString, I: Into<Sexp>> (key: S, value: I) -> Sexp {
        Sexp::Pair(Some(Box::new(Sexp::Symbol(key.to_string()))),
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
    /// ```rust
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let object = sexp!(((A . 65) (B . 66) (C . 67)));
    /// assert_eq!(*object.get("A").unwrap(), sexp!(65));
    ///
    /// let array = json!((A B C));
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
    /// ```rust
    /// # #[macro_use]
    /// # extern crate sexpr;
    /// #
    /// # fn main() {
    /// let object = sexp!((
    ///     (A . ("a" "á" "à"))
    ///     (B . ("b" "b́"))
    ///     (C . ("c" "ć" "ć̣" "ḉ"))
    /// ));
    /// assert_eq!(object["B"][0], json!("b"));
    ///
    /// assert_eq!(object["D"], json!(null));
    /// assert_eq!(object[0]["x"]["y"]["z"], json!(null));
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
/// ```rust
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
/// ```rust
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
/// ```rust
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

