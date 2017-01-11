#![allow(dead_code)]

use Sexp;

/// The errors that can arise while parsing a S-expression stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidNumber,
    InvalidEscape,
    UnrecognizedBase64,
    UnrecognizedHex,
    UnexpectedEndOfHexEscape,
    EOFWhileParsingList,
    EOFWhileParsingValue,
    EOFWhileParsingNumeric,
    ControlCharacterInString,
    TrailingCharacters,
}

#[derive(Debug)]
pub enum ParserError {
    ///         msg,      line,   col
    SyntaxError(ErrorCode, usize, usize),
    // IoError(io::Error),
}


use self::ErrorCode::*;
use self::ParserError::*;

pub struct ParseConfig {
    // Escape #number# to it's appropriate hex decoding.
    allow_hex_escapes: bool,
    // Accept '[' and ']' in addition to parenthesis
    accepts_square_brackets: bool,
    // Should atoms be read case-insensitively?
    case_insensitive: bool,
}

/// A streaming S-Exp parser implemented as an iterator of SexpEvent, consuming
/// an iterator of char.
pub struct Parser<T> {
    reader: T,
    ch: Option<char>,
    line: usize,
    col: usize,
    configuration: Option<ParseConfig>,
}

type ParseResult = Result<Sexp, ParserError>;

impl<T: Iterator<Item = char>> Parser<T> {

    pub fn new(reader: T) -> Parser<T> {
        let mut p = Parser {
            reader: reader,
            ch: Some('\x00'),
            line: 1,
            col: 0,
            configuration: None
        };
        p.bump();
        return p;
    }

    fn bump(&mut self) {
        self.ch = self.reader.next();

        if self.ch_is('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn error(&mut self, reason: ErrorCode) -> ParseResult {
        Err(SyntaxError(reason, self.line, self.col))
    }

    fn accept_brackets(&self) -> bool {
        false
    }

    fn next_char(&mut self) -> Option<char> { self.bump(); self.ch }
    fn ch_or_null(&self) -> char { self.ch.unwrap_or('\x00') }
    fn ch_is(&self, c: char) -> bool {
        self.ch == Some(c)
    }
    fn eof(&self) -> bool { self.ch.is_none() }

    fn parse_whitespace(&mut self) {
        while self.ch_is(' ') ||
            self.ch_is('\n') ||
            self.ch_is('\t') ||
            self.ch_is('\r') { self.bump(); }
    }

    fn parse_string(&mut self) -> ParseResult {
        debug("Parsing String");
        let mut result = String::new();
        let mut escape = false;

        loop {
            if escape {
                // do escape bullshit
                match self.ch_or_null() {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return self.error(InvalidEscape),
                }
            } else if self.ch_is('\\') {
                escape = true;
            } else {
                match self.ch {
                    Some('"') => {
                        self.bump();
                        return Ok(Sexp::Atom(result));
                    },
                    Some(ch) if ch <= '\u{1F}' =>
                        return self.error(ControlCharacterInString),
                    Some(ch) => result.push(ch),
                    None => unreachable!()
                }
            }
        }

    }

    fn parse_numeric(&mut self) -> ParseResult {
        debug("Parsing Numeric");
        let mut result = String::new();
        result.push(self.ch.unwrap());;
        let mut is_float = false;

        loop {
            match self.next_char() {
                Some('.') => is_float = true,
                ch @ Some('0' ... '9') => result.push(ch.unwrap()),
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
        // skip whitespace
        self.parse_whitespace();
        match self.ch {
            Some('.') => {
                self.bump();
                self.parse_value()
            },
            // The end of a list is defined as #nil
            Some(')') => Ok(Sexp::Nil),
            Some(_ch) => {
                // parse a value, put it in car.
                Ok(Sexp::Cons {
                    car: Box::new(self.parse_value()?),
                    cdr: Box::new(self.parse_list()?)
                })
            }
            None => Ok(Sexp::Nil)
        }
    }

    pub fn parse_value(&mut self) -> ParseResult {
        if self.eof() { return self.error(EOFWhileParsingValue); }

        debug(&format!("self.ch: {:?}", self.ch));
        match self.ch {
            Some('(') => {
                self.bump();
                self.parse_list()
            },
            // Some(')') | Some(']') if self.config.SquareBrackets => (),
            Some('-') | Some('0' ... '9') => self.parse_numeric(),
            // Some('"') => self.parse_string(),
            // Some('#') if self.config.HexEscapes => (),
            Some(ch) => {
                // if (self.accept_canonical) {
                //     parse_canonical_value()
                // }
                self.parse_atom()
            },
            None => self.error(EOFWhileParsingValue)
        }
    }
}
