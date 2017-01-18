#![allow(dead_code)]

use Sexp;

use error::ErrorCode;
use error::ErrorCode::*;
use error::ParserError;
use error::ParserError::*;

// Contains the configuration parameters to the parser
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParsePipeBehavior {
    // Accept a base64 encoding of the octet string, e.g (|NFGq/E3wh9f4rJIQVXhS|)
    Base64Interior,
    // Accept everything within two pipes as a valid atom, e.g (|this is an atom with spaces|)
    QuoteInterior,
    // Pipes are treated just like any other atom character.
    None
}

#[derive(Clone, Copy, Debug)]
pub struct ParseConfig {
    // Should semicolons ignore the remainder of the line?
    pub semi_comments: bool,
    // Should atoms be read case-insensitively?
    pub case_sensitive_atoms: bool,

    // Accept '[' and ']' in addition to parenthesis
    pub square_brackets: bool,

    // Pipes can accept a multitude of differing options
    pub pipe_action: ParsePipeBehavior,
    // Escape #NUMBER# to it's appropriate hex decoding.

    pub hex_escapes: bool,
    // Escapes #xNUMBER (hex) and #bNUMBER (binary) to their respective encodings
    pub radix_escape: bool,
}

/// Configuration for RFC 4648 standard base64 encoding
pub static STANDARD: ParseConfig = ParseConfig {
    semi_comments: true,
    square_brackets: true,
    case_sensitive_atoms: false,
    pipe_action: ParsePipeBehavior::None,
    hex_escapes: true,
    radix_escape: false,
};

/// A streaming S-Exp parser implemented as an iterator of `SexpEvent`, consuming
/// an iterator of char.
pub struct Parser<T> {
    reader: T,
    ch: Option<char>,
    line: usize,
    col: usize,
    config: ParseConfig,
}

type ParseResult = Result<Sexp, ParserError>;

fn debug(s: &str) { if false {println!("{}", s)} }

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

    fn parse_comment(&mut self) {
        while !self.ch_is('\n') { self.bump(); }
    }

    fn parse_whitespace(&mut self) {
        while self.ch_is(' ') ||
            self.ch_is('\n') ||
            self.ch_is('\t') ||
            self.ch_is('\r') { self.bump(); }
    }

    fn parse_atom(&mut self) -> ParseResult {
        debug("Parsing Atom");
        let mut result = String::new();
        loop {
            // In cases with large number of or-cases, it's more convienent to
            // unwrap the character ahead of time and check for the null byte as
            // a sentinel for EOF, this of course prevents us from checking for
            // an actual null byte, which is valid in a s-expression however.
            match self.ch_or_null() {
                ch @ 'a' ... 'z' => result.push(ch),
                '\t' | ' ' | '\n' | ')' => return Ok(Sexp::Symbol(result)),
                // We've encountered an EOF
                '\x00' => return self.error(EOFWhileParsingAtom),
                _ => return self.error(InvalidAtom),
            };
            self.bump();
        }
    }

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

    /// There is
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
                _ => return self.error(InvalidNumber)
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

    fn parse_list(&mut self) -> ParseResult {
        let mut result: Vec<Sexp> = vec![];

        loop {
            self.parse_whitespace();
            match self.ch {
                // The end of a list is defined as #nil
                Some('.') => {
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
                Some(')') => {
                    self.bump();
                    break;
                },
                Some(_) => {
                    // parse a value, put it in car.

                    // This code could be really funky, might want to check for EOF
                    // after parse_value
                    result.push(self.parse_value()?);
                    // If we'v
                    if self.eof() {
                        break;
                    }
                }
                None => return self.error(EOFWhileParsingList),
            }
        }

        Ok(Sexp::List(result))
    }

    // Parsing begins at `parse_value` and functions that can build recursive
    // structures may use parse_value to select it's next data element (it's
    // `car`).
    pub fn parse_value(&mut self) -> ParseResult {
        if self.eof() { return self.error(EOFWhileParsingValue); }
        self.parse_whitespace();
        match self.ch_or_null() {
            '(' => {
                self.bump();
                self.parse_list()
            },
            ')' => self.error(UnexpectedEndOfList),
            '-' | '0' ... '9' => self.parse_numeric(),
            '"' => self.parse_string(),
            '#' if self.config.hex_escapes =>
                self.parse_hexadecimal(),
            '\x00' => self.error(EOFWhileParsingValue),
            _ => self.parse_atom(),
        }
    }
}
