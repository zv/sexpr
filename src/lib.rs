// Copyright 2017 Zephyr "zv" Pellerin. See the COPYRIGHT
// file at the top-level directory of this distribution
//
// Licensed under the MIT License, <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//! # Sexpr
//!
//! ## What are S-expressions?
//!
//! S-expressions are a lightweight format for representating and transmitting data
//! consisting of parenthesis pairs.
//!
//! ```scheme
//! (package
//!  (name "sexpr")
//!  (version "0.7.0")
//!  (authors ("Zephyr Pellerin <zv@nxvr.org>") :only)
//!  (license "MIT/Apache-2.0")
//!  (description "Multi-format S-expression serialization/deserialization support")
//!  (repository "https://github.com/zv/sexpr")
//!  (keywords ("sexp","s-exp","sexpr","smtlib"))
//!  (categories ("encoding"))
//!  (readme "README.org")
//!  (documentation  "https://zv.github.io/rust/sexpr"))
//! ```
//!
//! Sexpr also supports more complex types; including keywords with configurable
//! tokens for `true`, `false` and `nil`.
//!
//! ```scheme
//! (define-class rectangle ()
//!  (width
//!    #:init-value #nil ;; Nil value
//!    #:settable #t     ;; true
//!    #:guard (> width 10)
//!    )
//!   (height
//!    #:init-value 10
//!    #:writable #f ;; false
//!   )
//! )
//! ```
//!
//! Here are some common ways in which you might use sexpr.
//!
//! - **As text data**: An unprocessed string of s-expressions that you recieve from
//!   some HTTP endpoint, read from a file, or prepare to send to a remote server.
//! - **As untyped data**: Determining if some S-expression is valid before passing
//!   it on, or to do basic manipulations like inserting keys or items into a
//!   list.
//! - **As a strongly-typed Rust data structure**: When you expect all of your data
//!   to conform to a particular structure and you want to get work done without
//!   S-expressions loosely-typed struture.
//!
//! Sexpr provides efficient, flexible, safe ways of converting data between
//! each of these representations.
//!
//! # Operating on untyped JSON values
//!
//! Any valid s-exp can be manipulated in the following recursive enum
//! representation. This data structure is [`sexpr::Sexp`][sexp].
//!
//! ```rust
//!  # use sexpr::{Number, Map};
//!  #
//!  # #[allow(dead_code)]
//! enum Sexp {
//!     Nil,
//!     Symbol(String),
//!     String(String),
//!     Keyword(String),
//!     Number(Number),
//!     Boolean(bool),
//!     Pair(ConsCell, ConsCell),
//!     List(Vec<Sexp>),
//! }
//! ```
//!
//! A string of S-expressions may be parsed into a `sexpr::Sexp` by the
//! [`sexpr::from_str`][from_str] function. There is also
//! [`from_slice`][from_slice] for parsing from a byte slice &[u8] and
//! [`from_reader`][from_reader] for parsing from any `io::Read` like a File or
//! a TCP stream.
//!
//! ```rust
//!  extern crate sexpr;
//!
//!  use sexpr::{Sexp, Error};
//!
//!  fn untyped_example() -> Result<(), Error> {
//!      // Some s-expressions a &str.
//!      let data = r#"(
//!                      (name . "John Doe")
//!                      (age . 43)
//!                      (phones . (
//!                        "+44 1234567"
//!                        "+44 2345678"))
//!                    )"#;
//!
//!      // Parse the string of data into sexpr::Sexp.
//!      let v: Sexp = sexpr::from_str(data)?;
//!
//!      // Access parts of the data by indexing with square brackets.
//!      println!("Please call {} at the number {}", v["name"], v["phones"][0]);
//!
//!      Ok(())
//!  }
//!  #
//!  # fn main() {
//!  #     untyped_example().unwrap();
//!  # }
//! ```
//!
//!
//! # Parsing S-expressions as strongly typed data structures
//!
//! Serde provides a powerful way of mapping S-expression data into Rust data
//! structures automatically.
//!
//! ```rust
//! extern crate serde;
//! extern crate sexpr;
//!
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use sexpr::Error;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Person {
//!     name: String,
//!     age: u8,
//!     phones: Vec<String>,
//! }
//!
//! fn typed_example() -> Result<(), Error> {
//!     // Some SEXP input data as a &str. Maybe this comes from the user.
//!     let data = r#"(
//!                     ("name" . "John Doe")
//!                     ("age" . 43)
//!                     ("phones" . (
//!                       "+44 1234567"
//!                       "+44 2345678"
//!                     ))
//!                   )"#;
//!
//!     // Parse the string of data into a Person object. This is exactly the
//!     // same function as the one that produced sexpr::Sexp above, but
//!     // now we are asking it for a Person as output.
//!     let p: Person = sexpr::from_str(data)?;
//!
//!     // Do things just like with any other Rust data structure.
//!     println!("Please call {} at the number {}", p.name, p.phones[0]);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     typed_example().unwrap();
//! # }
//! ```
//!
//! This is the same `sexpr::from_str` function as before, but this time we
//! assign the return value to a variable of type `Person` so Serde will
//! automatically interpret the input data as a `Person` and produce informative
//! error messages if the layout does not conform to what a `Person` is expected
//! to look like.
//!
//! Any type that implements Serde's `Deserialize` trait can be deserialized
//! this way. This includes built-in Rust standard library types like `Vec<T>`
//! and `HashMap<K, V>`, as well as any structs or enums annotated with
//! `#[derive(Deserialize)]`.
//!
//! Once we have `p` of type `Person`, our IDE and the Rust compiler can help us
//! use it correctly like they do for any other Rust code. The IDE can
//! autocomplete field names to prevent typos, which was impossible in the
//! `sexpr::Sexp` representation. And the Rust compiler can check that
//! when we write `p.phones[0]`, then `p.phones` is guaranteed to be a
//! `Vec<String>` so indexing into it makes sense and produces a `String`.
//!
//! # Creating S-expressions by serializing data structures
//!
//! A data structure can be converted to a sexp string by
//! [`sexpr::to_string`][to_string]. There is also
//! [`sexpr::to_vec`][to_vec] which serializes to a `Vec<u8>` and
//! [`sexpr::to_writer`][to_writer] which serializes to any `io::Write`
//! such as a File or a TCP stream.
//!
//! ```rust
//! extern crate serde;
//! extern crate sexpr;
//!
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use sexpr::Error;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Address {
//!     street: String,
//!     city: String,
//! }
//!
//! fn print_an_address() -> Result<(), Error> {
//!     // Some data structure.
//!     let address = Address {
//!         street: "10 Downing Street".to_owned(),
//!         city: "London".to_owned(),
//!     };
//!
//!     // Serialize it to a SEXP string.
//!     let j = sexpr::to_string(&address)?;
//!
//!     // Print, write to a file, or send to an HTTP server.
//!     println!("{}", j);
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     print_an_address().unwrap();
//! # }
//! ```
extern crate num_traits;
// extern crate core;
#[macro_use]
extern crate serde;
extern crate itoa;
extern crate dtoa;

#[doc(inline)]
pub use self::de::{Deserializer, StreamDeserializer, from_reader, from_slice, from_str};
#[doc(inline)]
pub use self::error::{Error, Result};
#[doc(inline)]
pub use ser::{to_string, Serializer};
#[doc(inline)]
pub use self::sexp::{Sexp, Number, from_value, to_value};

#[macro_use]
mod macros;

pub mod de;
pub mod error;
pub mod ser;
pub mod sexp;

mod iter;
mod number;
mod atom;
mod read;
