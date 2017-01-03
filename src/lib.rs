/// An s-expression is either an atom or a list of s-expressions. This is
/// similar to the data format used by lisp.
#[derive(Clone, Debug)]
pub enum Sexp {
    Nil,
    Atom(String),
    Cons { car: Box<Sexp>, cdr: Box<Sexp> }
}

#[allow(unused_variables)]
fn dbg(msg: &str, pos: &usize) { println!("{} @ {}", msg, pos) }


#[derive(Clone, Debug)]
pub enum ReadSexpError {
    InvalidTerminal,
    UnknownSymbol,
}

impl Sexp {
    fn read(tokens: &str) -> Result<Sexp, ReadSexpError> {
        // returns the char it found, and the new size if you wish to consume that char
        fn peek(s: &str, pos: &usize) -> Option<(char, usize)> {
            dbg("peek", pos);
            if *pos == s.len() {
                None
            } else if s.is_char_boundary(*pos) {
                let ch = s[*pos..].chars().next().unwrap();
                let next = *pos + ch.len_utf8();
                Some((ch, next))
            } else {
                // strings must be composed of valid utf-8 chars.
                unreachable!()
            }
        }

        fn read_term(tokens: &str, pos: &mut usize) -> Result<Sexp, ReadSexpError> {
            dbg("read_term", pos);
            let mut nnn = String::new();
            while let Some((ch, _)) = peek(tokens, pos) {
                *pos = *pos + 1;
                match ch {
                    'A'...'Z' | 'a'...'z' => nnn.push(ch),
                    _ => break
                }
            }

            Ok(Sexp::Atom(nnn))
        }

        fn read_list(tokens: &str, pos: &mut usize) -> Result<Sexp, ReadSexpError> {
            let mut seek = || {
                let mut result: Sexp = Sexp::Nil;
                loop {
                    let (ch, sz) = match peek(tokens, pos) {
                        Some(i) => i,
                        None => return result
                    };
                    if ch == ' ' { *pos = sz; continue; }
                    if ch == ')' { *pos = sz; return result; }

                    result = match ch {
                        '('  => {
                            *pos = *pos + 1;
                            read_list(tokens, pos).unwrap()
                        },
                        // '0'...'9' => read_num(tokens, pos).unwrap(),
                        'A'...'z' => read_term(tokens, pos).unwrap(),
                        _ => unimplemented!(),
                    };

                    return result;
                }
            };

            Ok(Sexp::Cons { car: Box::new(seek()), cdr: Box::new(seek())})
        }

        let mut pos = 0;
        let (c, _) = peek(tokens, &pos).unwrap();
        let r =
            if c == '(' { read_list(tokens, &mut pos) }
            else { read_term(tokens, &mut pos) };

        r
    }
}

#[cfg(test)]
mod tests {
    use ::Sexp;
    macro_rules! expand_sexp {
        () => {{ Sexp::Nil }};
        (atom_s[$string:expr]) => {{ Sexp::Atom(String::from($string)) }};
        (cons [ car[ $car:tt ] cdr[ $cdr:tt ] ]) => {{
            Sexp::Cons { car: Box::new(expand_sexp!($car)), cdr: Box::new(expand_sexp!($cdr))}
        }};
        (cons [ car[ $($car:tt)* ] cdr[ $($cdr:tt)* ] ]) => {{
            Sexp::Cons { car: Box::new(expand_sexp!($($car)*)), cdr: Box::new(expand_sexp!($($cdr)*))}
        }};
    }

    #[test]
    fn test_sexp_reader() {
        assert_eq!(
            println!("{:?}", Sexp::read("((a b) (c (d)))").unwrap()),
            println!("{:?}", expand_sexp!(
                cons[
                    car[cons[car[atom_s["a"]]
                             cdr[atom_s["b"]]]]
                        cdr[cons[car[atom_s["c"]] cdr[cons[car[atom_s["d"]] cdr[]] ]]]]))
        )
    }
}
