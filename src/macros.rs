// Copyright 2017 Zephyr Pellerin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Construct a `sexpr::Sexp` from a S-expression literal.
///
/// ```rust,ignore
/// # #[macro_use]
/// # extern crate sexpr;
/// #
/// # fn main() {
/// let value: Sexp = sexp!((
///     ("code" . 200)
///     ("success" . true)
///     ("payload" .
///         ("features" . ("serde" "sexpr")))
/// ));
/// # }
/// ```
#[macro_export]
macro_rules! sexp {
    ($t:tt) => {
        $crate::from_str(stringify!($t)).unwrap();
    };
}
