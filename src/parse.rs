#![allow(dead_code)]

use Sexp;

use error::ErrorCode;
use error::ErrorCode::*;
use error::ParserError;
use error::ParserError::*;
use config::{STANDARD, ParseConfig};

type ParseResult = Result<Sexp, ParserError>;

/// A streaming S-Exp parser implemented as an iterator of `SexpEvent`, consuming
/// an iterator of char.
pub struct Parser<T> {
    reader: T,
    ch: Option<char>,
    line: usize,
    col: usize,
    config: ParseConfig,
}

fn debug(s: &str) { if false { println!("{}", s) } }

impl<T: Iterator<Item = char>> Parser<T> {
    pub fn new(reader: T) -> Parser<T> {
        let mut p = Parser {
            reader: reader,
            ch: Some('\x00'),
            line: 1,
            col: 0,
            config: STANDARD
        };
        p.bump();

        p
    }

    /// Advance the head to the next 'character', whatever that designation
    /// implies for a particular parser-configuration.
    fn bump(&mut self) {
        self.ch = self.reader.next();

        if self.ch_is('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        // If `semi_comments` is enabled, we should read the stream until
        // newline as a comment.
        if self.config.semi_comments && self.ch_is(';') {
            self.parse_comment();
        }
    }

    fn error(&mut self, reason: ErrorCode) -> ParseResult {
        Err(SyntaxError(reason, self.line, self.col))
    }

    fn next_char(&mut self) -> Option<char> { self.bump(); self.ch }
    fn ch_is(&self, c: char) -> bool { self.ch == Some(c) }
    fn eof(&self) -> bool { self.ch.is_none() }
    fn ch_or_null(&self) -> char { self.ch.unwrap_or('\x00') }

    /// Consume tokens until a newline or other comment terminator
    fn parse_comment(&mut self) {
        while !self.ch_is('\n') { self.bump(); }
    }

    /// Consume tokens until the head is no longer 'whitespace'
    fn parse_whitespace(&mut self) {
        while self.ch_is(' ') ||
            self.ch_is('\n') ||
            self.ch_is('\t') ||
            self.ch_is('\r') { self.bump(); }
    }

    // `parse_symbol` reads in any bare atom not otherwise handled by other
    // 'arms' of the recursive descent parser. In some s-expression variants,
    // this is technically invalid. If your use-case needs to be restrictive in
    // which s-expressions you accept, consider modifying the configuration to
    // bail on particular tokens.
    fn parse_symbol(&mut self) -> Option<String> {
        debug("Parsing symbol");
        let mut result = String::new();
        loop {
            // In cases with large number of or-cases, it's more convienent to
            // unwrap the character ahead of time and check for the null byte as
            // a sentinel for EOF, this of course prevents us from checking for
            // an actual null byte, which is valid in a s-expression however.
            match self.ch_or_null() {
                ch @ 'a' ... 'z' => result.push(ch),
                '\t' | ' ' | '\n' | ')' | '\x00' | ']' if self.config.square_brackets
                    => break,
                // This is a superset fallthrough of the earlier 'a'...'z'
                // pattern, this is a convienent stub for later changes to the
                // definition of a valid symbol
                ch => result.push(ch)
            };
            self.bump();
        }

        // A 0-length symbol isn't one at all -- Return none.
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    fn parse_atom(&mut self) -> ParseResult {
        debug("Parsing Atom");
        match self.parse_symbol() {
            Some(atom) => Ok(Sexp::Symbol(atom)),
            None => self.error(InvalidAtom)
        }
    }

    fn parse_keyword(&mut self) -> ParseResult {
        debug("Parsing Keyword");
        match self.parse_symbol() {
            Some(atom) => Ok(Sexp::Keyword(atom)),
            None => self.error(InvalidAtom)
        }
    }

    // `parse_string` reads in a string, which can contain a variety of control
    // characters, unicode escapes and other sub-languages.
    fn parse_string(&mut self) -> ParseResult {
        debug("Parsing String");
        let mut result = String::new();
        let mut escape = false;

        loop {
            self.bump();

            if escape {
                // This implements a subset of the valid escapes that many
                // Scheme's read implementation guarantees
                match self.ch {
                    Some('"')  => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/')  => result.push('/'),
                    Some('b')  => result.push('\x08'),
                    Some('f')  => result.push('\x0c'),
                    Some('n')  => result.push('\n'),
                    Some('r')  => result.push('\r'),
                    Some('t')  => result.push('\t'),
                    Some(_)    => return self.error(InvalidEscape),
                    None       => return self.error(UnexpectedEndOfHexEscape)
                }
                escape = false;
            } else if self.ch_is('\\') {
                escape = true;
            } else {
                match self.ch {
                    Some('"') => {
                        self.bump();
                        return Ok(Sexp::String(result));
                    },
                    Some(ch) if ch <= '\u{1F}' =>
                        return self.error(ControlCharacterInString),
                    Some(ch) => result.push(ch),
                    None => unreachable!()
                }
            }
        }
    }

    // `parse_numeric` is responsible for the variety of numbers that Sexpr can
    // handle. It implements a strait-forward algorithm of reading until a space
    // occurs, at which point any of the various modifiers (such as "negative"
    // or "decimal") are applied
    fn parse_numeric(&mut self) -> ParseResult {
        debug("Parsing Numeric");
        let mut result: String = self.ch.unwrap().to_string();
        let mut is_float = false;

        loop {
            if self.ch_is('.') { is_float = true }
            match self.next_char() {
                Some(ch @ '.') | Some(ch @ '0' ... '9') => result.push(ch),
                Some(_) => break,
                None => return self.error(EOFWhileParsingNumeric)
            };
        }

        if is_float {
            let n = result.parse::<f64>();
            match n {
                Ok(num) => Ok(Sexp::F64(num)),
                Err(_) => self.error(InvalidNumber)
            }
        } else {
            let n = result.parse::<i64>();
            match n {
                Ok(num) => Ok(Sexp::I64(num)),
                Err(_) => self.error(InvalidNumber)
            }
        }
    }

    // `parse_hexadecimal` handles a special case of parsing numeric values.
    // Like `parse_numeric`, it reads until it encounters a space, applying
    // appropriate 'modifiers', bailing out if a modifier is invalid for a
    // particular configuration.
    fn parse_hexadecimal(&mut self) -> ParseResult {
        debug("Parsing Hexadecimal");
        let mut accumulator: u64 = 0; // Could be shortened to acc ...
        let mut length: usize = 0;

        if self.next_char() != Some('x') {
            return self.error(UnrecognizedHex);
        }

        while !self.eof() {
            let significand: u64;
            // Take out the last digit, shift the base by 10 and add the
            // least significant digit
            match self.next_char() {
                Some(c @ '0' ... '9') =>
                    significand = (c as u8 - b'0') as u64,
                Some(c @ 'a' ... 'f') =>
                    significand = (c as u8 - b'a') as u64 + 10,
                Some(c @ 'A' ... 'F') =>
                    significand = (c as u8 - b'A') as u64 + 10,
                // DRYing this out is tough: Patterns are a 'metafeature' and
                // can't be enconded in a variable - a function could perhaps
                // replace this.
                Some(' ') | Some('\t') | Some('\n') | Some(')') => break,
                None => unreachable!(),
                _ => return self.error(InvalidNumber),
            }

            length += 1;
            accumulator = accumulator * 10 + significand;
        }

        if length == 0 {
            // a length of 0 means we've encountered "#x" - Invalid
            self.error(UnexpectedEndOfHexEscape)
        } else {
            Ok(Sexp::U64(accumulator))
        }
    }

    // `parse_list` is called when we've encountered a bracket (paren or square
    // bracket).
    // the `opening_ch` signifies what variety of bracket we're dealing with.
    // Knowing which bracket allows us to hold multiple different 'stacks' of
    // s-expressions, each with their own brackets:
    // e.g (a [b c (d e [f g])])
    fn parse_list(&mut self, opening_ch: char) -> ParseResult {
        let mut result: Vec<Sexp> = vec![];

        loop {
            self.parse_whitespace();
            match self.ch_or_null() {
                // The end of a list is defined as #nil
                '.' => {
                    self.bump();
                    result.push(self.parse_value()?);

                    // We can safely assume there are at least two elts (or one elt and an empty list)
                    match self.ch {
                        Some(')') => {
                            self.bump();
                            return Ok(Sexp::new_pair(&result[0], &result[1]));
                        }
                        _ => return self.error(MissingCloseParen)
                    }

                },
                ch @ ')' | ch @ ']' => {
                    // This predicate checks for unbalanced expressions -- If
                    // our opening character differs from an appropriate closing
                    // character
                    if (ch == ')' && opening_ch == '[') |
                       (self.config.square_brackets && ch == ']' && opening_ch == '(') {
                        return self.error(UnbalancedClosingParen)
                    } else {
                        self.bump();
                        break;
                    }
                },
                '\x00' => return self.error(EOFWhileParsingList),
                _ => {
                    // parse a value, put it in car.

                    // This code could be really funky, might want to check for EOF
                    // after parse_value
                    result.push(self.parse_value()?);
                    // If we'v
                    if self.eof() {
                        break;
                    }
                }
            }
        }

        Ok(Sexp::List(result))
    }

    // Invoked at each iteration, consumes the stream until it has enough
    // information to return a `ParseResult`.
    // Manages an internal state so that parsing can be interrupted and resumed.
    // Also keeps track of the position in the logical structure of the sexp
    // stream which may be queried by the user while running.
    pub fn parse_value(&mut self) -> ParseResult {
        if self.eof() { return self.error(EOFWhileParsingValue); }
        self.parse_whitespace();
        match self.ch_or_null() {
            paren @ '(' | paren @ '[' if self.config.square_brackets => {
                self.bump();
                self.parse_list(paren)
            },
            ')' => self.error(UnexpectedEndOfList),
            '-' | '0' ... '9' => self.parse_numeric(),
            '"' => self.parse_string(),
            '#' if self.config.hex_escapes =>
                self.parse_hexadecimal(),
            ':' if self.config.colon_keywords => self.parse_keyword(),
            '\x00' => self.error(EOFWhileParsingValue),
            _ => self.parse_atom(),
        }
    }


    // Presently, `parse` is a stub function for coordinating pre & post action
    // hooks.
    pub fn parse(&mut self) -> ParseResult {
        self.parse_value()
    }
}
