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

/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Sexp {
    Nil,
    Symbol(String),
    String(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    Cons { car: Box<Sexp>, cdr: Box<Sexp> }
}

mod parse;
mod error;

use parse::Parser;
use error::ParserError;

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


        #[allow(useless_format, unknown_lints)]
        fn fmt_driver(sexp: &Sexp) -> String {
            match *sexp {
                Nil => format!("#nil"),
                Symbol(ref sym) => format!("{}", sym),
                String(ref string) => format!("\"{}\"", string),
                F64(num) => format!("{}", num),
                I64(num) => format!("{}", num),
                U64(num) => format!("{}", num),
                Boolean(true) => format!("#t"),
                Boolean(false) => format!("#f"),
                // Due to the complex rules surrounding how a 'list' of
                // s-expressions is stringified, there is a lot of logic here
                // for determining the inclusion of surrounding parenthesis
                Cons { box ref car, box ref cdr } => {
                    match *car {
                        Sexp::Cons { car: ref caar, cdr: ref cdar } => {
                            format!("({} {})", fmt_driver(caar), fmt_driver(cdar))
                        },
                        _ => {
                            match *cdr {
                                Nil => format!("{}", fmt_driver(car)),
                                Cons { .. } =>
                                    format!("{} {}", fmt_driver(car), fmt_driver(cdr)),
                                _ => format!("{} . {}", fmt_driver(car), fmt_driver(cdr)),
                            }
                        }
                    }
                }
            }
        }

        write!(f, "({})", fmt_driver(self))
    }
}

impl Sexp {
    pub fn car(self) -> Option<Sexp> {
        match self {
            Sexp::Cons { car: box car, .. } => Some(car),
            _ => None
        }
    }

    pub fn cdr(self) -> Option<Sexp> {
        match self {
            Sexp::Cons { cdr: box cdr, .. } => Some(cdr),
            _ => None
        }
    }

    pub fn cadr(self) -> Option<Sexp> {
        match self.cdr() {
            Some(cdr @ Sexp::Cons { .. }) => cdr.car(),
            _ => None
        }
    }

    pub fn cddr(self) -> Option<Sexp> {
        match self.cdr() {
            Some(cdr @ Sexp::Cons { .. }) => cdr.cdr(),
            _ => None
        }
    }

    // pub fn cons(car: Sexp, cdr: Sexp) -> Sexp {
    //     Sexp::Cons {
    //         car: Box::new(car),
    //         cdr: Box::new(cdr)
    //     }
    // }
}


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
