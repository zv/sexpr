use super::Sexp;
use super::Sexp::*;

use std::str::FromStr;

use std::fmt;
use std::fmt::{Formatter};

impl fmt::Display for Sexp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Nil => write!(f, "nil"),
            Symbol(ref sym) | Keyword(ref sym)  =>
                write!(f, "{}", sym),
            String(ref string) => write!(f, "\"{}\"", string),
            Number(ref num)    => write!(f, "{}", num),
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
