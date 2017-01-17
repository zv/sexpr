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
use std::fmt;
use std::str::FromStr;
use std::string::String;
use std::collections::BTreeMap;

use std::rc::Rc;

type ConsCell = Option<Rc<Sexp>>;

/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Sexp {
    Symbol(String),
    String(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    Pair(ConsCell, ConsCell),
    List(Vec<Sexp>)
}

mod parse;
mod error;

use parse::Parser;
use error::{ParserError, IntoAlistError};
use error::IntoAlistError::*;

impl FromStr for Sexp {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Sexp, Self::Err> {
        let mut p = Parser::new(s.chars());
        p.parse_value()
    }
}

use self::Sexp::*;

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Symbol(ref sym)    => write!(f, "{}", sym),
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
                             else {a} + &b.to_string()))
            },
            Pair(Some(ref car), Some(ref cdr)) => write!(f, "({} . {})", car, cdr),
            Pair(Some(ref car), None)      => write!(f, "({})", car),
            Pair(None, Some(ref cdr))      => write!(f, "(() . {})", cdr),
            Pair(None, None)           => write!(f, "(())"),
        }
    }
}

impl Sexp {
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
    /// let sexp = r#(
    ///  ("New York" "Albany")
    ///  ("Oregon"   "Salem")
    ///  ("Florida"  "Miami"))#
    /// let alist = Sexp::from_str(sexp);
    /// let map = alist.map(|s| Sexp::Pair(s[0], s[1]))
    ///                .collect::<Sexp::List>()
    ///                .into_map()
    /// ```
    pub fn into_map(self) -> Result<BTreeMap<String, Option<Rc<Sexp>>>, IntoAlistError> {
        let mut map = BTreeMap::new();
        match self {
            List(ref items) => for elt in items {
                match elt {
                    &Sexp::Pair(ref car, ref cdr) => {
                        match car {
                            &Some(ref value) => {
                                let key = format!("{}", *value);
                                if key.is_empty() {
                                    return Err(IntoAlistError::KeyValueMustBePair);
                                } else {
                                    if map.insert(key, cdr.clone()).is_some() {
                                        return Err(IntoAlistError::DuplicateKey);
                                    }
                                }
                            }
                            _ => return Err(IntoAlistError::KeyValueMustBePair)
                        }
                    }
                    _ => return Err(IntoAlistError::KeyValueMustBePair),
                }
            },
            _ => return Err(IntoAlistError::ContainerSexpNotList)
        }


        Ok(map)
    }

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
}

/*
#[cfg(test)]
mod tests {
use ::Sexp;
use std::str::FromStr;

/// Recursively expand an abbreviated s-expression format to it's full Rust
/// struct representation.
macro_rules! expand_sexp {
() => {{ Sexp::Nil }};
(atom[$string:expr]) => {{ Sexp::Symbol(String::from($string)) }};
(cons [ car[ $($car:tt)* ], cdr[ $($cdr:tt)* ] ]) => {{
Sexp::Cons { car: Box::new(expand_sexp!($($car)*)),
cdr: Box::new(expand_sexp!($($cdr)*))}
        }};
    }

    #[test]
    fn test_sexp_parser_simple() {
        let result = Sexp::from_str("(a b c)").unwrap();
        assert_eq!(result,
                   expand_sexp!(
                       cons[
                           car[atom["a"]],
                           cdr[cons[car[atom["b"]],
                                    cdr[cons[car[atom["c"]],
                                             cdr[]]]
                           ]]
                       ]
                   ))
    }

    #[test]
    fn test_sexp_parser_pair() {
        let result = Sexp::from_str("(a . b)").unwrap();
        assert_eq!(result,
                   expand_sexp!(
                       cons[car[atom["a"]],
                            cdr[atom["b"]]]))
    }

    #[test]
    fn test_sexp_display_numeric() {
        let src = "(1 (2 3 4))";
        assert_eq!(src, format!("{}", Sexp::from_str(src).unwrap()));
    }

    #[test]
    fn test_sexp_display_mixed_numeric() {
        let src = "(1 2.1 3 4)";
        assert_eq!(src, format!("{}", Sexp::from_str(src).unwrap()));
    }
}
 */
