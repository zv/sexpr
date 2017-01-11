/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Sexp {
    Nil,
    Atom(String),
    String(String),
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    Cons { car: Box<Sexp>, cdr: Box<Sexp> }
}

mod parse;

use parse::Parser;

impl Sexp {
    pub fn from_str(s: &str) -> Sexp {
        let mut p = Parser::new(s.chars());
        p.parse_value().unwrap()
    }
}



#[cfg(test)]
mod tests {
    use ::Sexp;

    /// Recursively expand an abbreviated s-expression format to it's full Rust
    /// struct representation.
    macro_rules! expand_sexp {
        () => {{ Sexp::Nil }};
        (atom[$string:expr]) => {{ Sexp::Atom(String::from($string)) }};
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
