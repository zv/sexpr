// Copyright 2017 Zephyr Pellerin <zv@nxvr.org>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_bytes;
extern crate sexpr;

use std::fmt::{Debug};
use std::{f32, f64};
use std::{u32, u64};
use std::{i8, i16, i32, i64};

//use serde::de::{self, Deserialize};
use serde::ser::{self};

use sexpr::{to_string, to_value};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },
    AntHive(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    a: (),
    b: usize,
    c: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    inner: Vec<Inner>,
}

fn test_encode_ok<T>(errors: &[(T, &str)])
    where
    T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = to_string(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value).unwrap();
        let s = to_string(&v).unwrap();
        assert_eq!(s, out);
    }
}

#[test]
fn test_write_u64() {
    let tests = &[(3u64, "3"), (u64::MAX, &u64::MAX.to_string())];
    test_encode_ok(tests);
}

#[test]
fn test_write_i64() {
    let tests = &[
        (3i64, "3"),
        (-2i64, "-2"),
        (-1234i64, "-1234"),
        (i64::MIN, &i64::MIN.to_string()),
    ];
    test_encode_ok(tests);
}

#[test]
fn test_write_f64() {
    let tests = &[
        (3.0, "3.0"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
    ];
    test_encode_ok(tests);
}


#[test]
fn test_write_str() {
    let tests = &[("", "\"\""), ("foo", "\"foo\"")];
    test_encode_ok(tests);
}

#[test]
fn test_write_bool() {
    let tests = &[(true, "#t"), (false, "#f")];
    test_encode_ok(tests);
}

#[test]
fn test_write_sym() {
    let tests = &[("sym", "sym"), ("Symbol", "Symbol")];
    test_encode_ok(tests);
}


// ///
// /// ```rust
// /// # #[macro_use]
// /// # extern crate sexpr;
// /// #
// /// # use sexpr::atom::Atom;
// /// # fn main() {
// /// assert!(Atom::Keyword("keyword"), Atom::discriminate("#:keyword"));
// /// assert!(Atom::Symbol("symbol"), Atom::discriminate("symbol"));
// /// assert!(Atom::String("string"), Atom::discriminate(r#""string""#));
// /// # }
// /// ```
