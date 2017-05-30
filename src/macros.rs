// Copyright 2017 Zephyr Pellerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Construct a `sexpr::Sexp` from a S-expression literal.
///
/// ```rust
/// # #[macro_use]
/// # extern crate sexpr;
/// #
/// # fn main() {
/// let value = sexp!((
///     ("code" . 200)
///     ("success" . true)
///     ("payload" .
///         ("features" . ("serde" "sexpr")))
/// ));
/// # }
/// ```
///
/// Variables or expressions can be interpolated into the S-exp literal by
/// prefixing it with a comma (`,`). Any type interpolated into an array element
/// or object value must implement Serde's `Serialize` trait, while any type
/// interpolated into a object key must implement `Into<String>`. If the
/// `Serialize` implementation of the interpolated type decides to fail, or if
/// the interpolated type contains a map with non-string keys, the `sexp!` macro
/// will panic.
///
/// ```rust
/// # #[macro_use]
/// # extern crate sexpr;
/// #
/// # fn main() {
/// let code = 200;
/// let features = vec!["serde", "sexpr"];
///
/// let value = sexp!((
///     (code . ,code)
///     (success . true)
///     (payload .
///         (,features[0] . ,features[1]))
/// ));
/// # }
/// ```
#[macro_export]
macro_rules! sexp {
    ($t:tt) => {
        $crate::from_str(stringify!($t)).unwrap();
    };
}
