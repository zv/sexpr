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
#![feature(box_patterns)]
extern crate rustc_serialize;
use self::rustc_serialize::Encodable;

use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;
use std::string::String;

mod config;

pub mod encodable;
use encodable::{Encoder, EncodeResult};

mod error;
use error::{ErrorCode, ParserError, IntoAlistError, SexpError};
use error::SexpError::*;
use error::InvalidType::*;

mod parse;
use parse::Parser;

// Rather than having a specialized 'nil' atom, we save space by letting `None`
// here indicates 'nil'
type SexpPtr = Rc<Sexp>;
type ConsCell = Option<SexpPtr>;

/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Sexp {
    /// A symbol or alist key
    Symbol(String),
    /// A UTF-8 String
    String(String),
    /// A keyword consists of a `:` (colon) followed by valid symbol characters.
    Keyword(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    /// A classic 'cons cell' structure whose elts are themselves cons-cells.
    Pair(ConsCell, ConsCell),
    List(Vec<Sexp>)
}

use ::Sexp::*;


/// Shortcut function to encode a `T` into a JSON `String`
pub fn encode<T: Encodable>(object: &T) -> EncodeResult<String> {
    let mut s = String::new();
    {
        let mut encoder = Encoder::new(&mut s);
        try!(object.encode(&mut encoder));
    }
    Ok(s)
}

impl Sexp {
    /// Makes a new pair
    fn new_pair(car: &Sexp, cdr: &Sexp) -> Sexp {
        // If we've encountered an empty list, replace our cdr with `None`
        let r_car: ConsCell;
        let r_cdr: ConsCell;
        r_car = Some(Rc::new(car.clone()));
        match cdr {
            &Sexp::List(ref elt) if elt.len() == 0 => r_cdr = None,
            _ => r_cdr = Some(Rc::new(cdr.clone()))
        }

        Sexp::Pair(r_car, r_cdr)
    }

    /// Returns a string if the s-expression is a primitive value.
    pub fn primitive_string(&self) -> Result<String, ErrorCode> {
        match *self {
            Keyword(ref s) | String(ref s) | Symbol(ref s) => Ok(s.to_owned()),
            U64(n) => Ok(format!("{}", n)),
            F64(n) => Ok(format!("{}", n)),
            I64(n) => Ok(format!("{}", n)),
            _ => Err(ErrorCode::InvalidAtom)
        }
    }
    /// Return a newly-created copy of lst with elements `PartialEq` to item
    /// removed. This procedure mirrors `memq`: `delq` compares elements of lst
    /// against item with eq?.
    pub fn delq(&self, other: Sexp) -> Result<Sexp, SexpError> {
        match *self {
            List(ref elts) => {
                // Build up a list of cloned elts, sans `other`
                let lst = elts.iter()
                    .filter(|x| **x != other)
                    .map(|x| x.clone())
                    .collect::<Vec<Sexp>>();

                Ok(Sexp::List(lst))
            }
            _ => Err(InvalidType(ExpectingList))
        }
    }

    /// Return the first `Sexp` of self who is `eq` with `other`.
    /// If x does not occur in lst, then `SexpError::NotFound` is returned.
    pub fn memq(&self, other: Sexp) -> Result<usize, SexpError> {
        match *self {
            List(ref elts) => {
                // Build up a list of cloned elts, sans `other`
                match elts.iter().position(|x| *x == other) {
                    Some(idx) => Ok(idx),
                    None => Err(NotFound)
                }
            },
            _ => Err(InvalidType(ExpectingList))
        }
    }

    /// Converts an alist structured S-expressions into a map.
    ///
    /// # Warning
    /// This generates string keys, which means that keys that share the same
    /// string representation will nessasarily be replaced, rather than
    /// duplicated.
    ///
    /// # Example
    ///
    /// Simple alist structure
    /// ```rust
    /// let sexp = r#(
    ///  (doctors  . ("Dr. Hargrove" "Dr. Steve" "Dr. Mischief" "Dr. Lizzie"))
    ///  (teachers . ("Ms. Taya" "Mr. Smith" "Principle Weedle"))
    ///  (sailors  . ("Ole Skippy" "Ravin' Dave" "Popeye")))#
    /// let alist = Sexp::from_str(sexp);
    /// let map = alist.into_map()
    /// ```
    ///
    /// Non-pair structured key-value list into map
    ///
    /// ```rust
    /// // Noticed that these are list, rather than pair structured S-expressions
    /// use sexpr::Sexp;
    /// use std::str::FromStr;
    /// let sexp = "(
    ///  (\"New York\" \"Albany\")
    ///  (\"Oregon\"   \"Salem\")
    ///  (\"Florida\"  \"Miami\"))";
    /// let alist = Sexp::from_str(sexp).unwrap().into_map();
    /// ```
    pub fn into_map(self) -> Result<BTreeMap<String, ConsCell>, IntoAlistError> {
        use error::IntoAlistError::*;
        let mut map = BTreeMap::new();
        match self {
            List(ref items) => for elt in items {
                match elt {
                    &Sexp::Pair(ref car, ref cdr) => {
                        match car {
                            &Some(ref value) => {
                                let key = format!("{}", *value);
                                if key.is_empty() {
                                    return Err(KeyValueMustBePair);
                                } else {
                                    if map.insert(key, cdr.clone()).is_some() {
                                        return Err(DuplicateKey);
                                    }
                                }
                            }
                            _ => return Err(KeyValueMustBePair)
                        }
                    }
                    _ => return Err(KeyValueMustBePair),
                }
            },
            _ => return Err(ContainerSexpNotList)
        }


        Ok(map)
    }
}

impl FromStr for Sexp {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Sexp, Self::Err> {
        let mut p = Parser::new(s.chars());
        p.parse()
    }
}

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Symbol(ref sym) | Keyword(ref sym)  =>
                write!(f, "{}", sym),
            String(ref string) => write!(f, "\"{}\"", string),
            F64(num)           => write!(f, "{}", num),
            I64(num)           => write!(f, "{}", num),
            U64(num)           => write!(f, "{}", num),
            Boolean(true)      => write!(f, "#t"),
            Boolean(false)     => write!(f, "#f"),
            List(ref elts)     => {
                write!(f, "({})",
                       elts // The following code joins the elements with a space separator
                       .iter()
                       .fold("".to_string(),
                             |a,b| if a.len() > 0 { a + " "}
                             else { a } + &b.to_string()))
            },
            Pair(Some(ref car), Some(ref cdr)) => write!(f, "({} . {})", car, cdr),
            Pair(Some(ref car), None)      => write!(f, "({})", car),
            Pair(None, Some(ref cdr))      => write!(f, "(() . {})", cdr),
            Pair(None, None)           => write!(f, "(())"),
        }
    }
}



#[cfg(test)]
mod tests {
    use ::Sexp;
    use error::SexpError;
    use std::str::FromStr;

    fn roundtrip(sexp: &str) -> String {
        format!("{}", Sexp::from_str(sexp).unwrap())
    }

    fn assert_decoded(expected: &str, sexp: &str) {
        assert_eq!(expected, roundtrip(sexp))
    }

    fn assert_roundtrip(sexp: &str) {
        assert_eq!(sexp, roundtrip(sexp))
    }

    #[test]
    fn test_sexp_parser_simple() { assert_roundtrip("(1 (2 (3 4 a (b) a)))") }

    #[test]
    fn test_escape_string() { assert_decoded("(a \"ab\"c\")", "(a \"ab\\\"c\")") }

    #[test]
    fn test_simple_pair() { assert_roundtrip("(a . b)") }

    #[test]
    fn test_long_pair() { assert_roundtrip("((a . b) (c . d) (e . 1))") }

    #[test]
    fn test_decode_hex_radix() { assert_decoded("(10 11 12)", "(#xa #xb #xc)") }

    #[test]
    fn test_skip_comment() {
        assert_decoded(
            "(a b c)",
            "(; this is a comment
              a
              ;; another comment
              b
              ;;; third comment
              c
              ;;; final comment
            )"
        )
    }

    #[test]
    fn test_square_brackets() { assert_decoded("(a (b c (d)))", "(a [b c (d)])") }

    #[test]
    #[should_panic]
    fn test_square_bracket_balance() { assert_roundtrip("(a b (a [b c) d] e)") }


    #[test]
    fn test_remove_1() {
        let lst = Sexp::from_str("(a b c d)").unwrap();
        let lst_delq = Sexp::from_str("(b c d)").unwrap();
        assert_eq!(
            lst.delq(Sexp::Symbol(String::from("a"))).unwrap(),
            lst_delq
        )
    }

    #[test]
    fn test_memq_1() {
        let lst = Sexp::from_str("(a b c d)").unwrap();
        assert_eq!(
            lst.memq(Sexp::Symbol(String::from("a"))).unwrap(),
            0
        )
    }

    #[test]
    fn test_memq_2() {
        let lst = Sexp::from_str("(a b c d)").unwrap();
        assert_eq!(
            lst.memq(Sexp::Symbol(String::from("x"))),
            Err(SexpError::NotFound)
        )
    }

    #[test]
    fn test_memq_3() {
        let lst = Sexp::from_str("(a b c d)").unwrap();
        assert_eq!(
            lst.memq(Sexp::Symbol(String::from("d"))).unwrap(),
            3
        )
    }
}
