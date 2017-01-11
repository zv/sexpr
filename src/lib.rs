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
//! ```rust
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
use std::str::FromStr;

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

impl Sexp {
    //  
    pub fn car(self) -> Option<Sexp> {
        match self {
            Sexp::Cons { car: box car, cdr: _ } => Some(car),
            _ => None
        }
    }

    pub fn cdr(self) -> Option<Sexp> {
        match self {
            Sexp::Cons { car: _, cdr: box cdr } => Some(cdr),
            _ => None
        }
    }

    pub fn cadr(self) -> Option<Sexp> {
        match self.cdr() {
            Some(cdr @ Sexp::Cons {car: _, cdr: _}) => cdr.car(),
            _ => None
        }
    }

    pub fn cddr(self) -> Option<Sexp> {
        match self.cdr() {
            Some(cdr @ Sexp::Cons {car: _, cdr: _}) => cdr.cdr(),
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

    // #[test]
    // fn test_sexp_reader() {
    //     let result = Sexp::from_str("(a b (c (d)))").unwrap();
    //     assert_eq!(result,
    //                expand_sexp!(
    //                    cons[
    //                        car[cons[car[atom["a"]],
    //                                 cdr[atom["b"]]]],
    //                        cdr[cons[car[atom["c"]],
    //                                 cdr[cons[
    //                                     car[atom["d"]],
    //                                     cdr[]]]]]]))
    // }

    // #[test]
    // fn test_simple_sexp_reader() {
    //     let result = Sexp::read("(a b c)").unwrap();
    //     assert_eq!(result,
    //                expand_sexp!(
    //                    cons[
    //                        car[atom["a"]],
    //                        cdr[cons[
    //                            car[atom["b"]],
    //                            cdr[cons[
    //                                car[atom["c"]],
    //                                cdr[]]]]]]
    //                ))
    // }
}
