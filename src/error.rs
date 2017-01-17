#![allow(non_snake_case)]

/// The errors that can arise while parsing a S-expression stream.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidAtom,
    InvalidNumber,
    InvalidEscape,
    MissingCloseParen,
    UnrecognizedBase64,
    UnrecognizedHex,
    UnexpectedEndOfHexEscape,
    UnexpectedEndOfList,
    EOFWhileParsingAtom,
    EOFWhileParsingList,
    EOFWhileParsingValue,
    EOFWhileParsingNumeric,
    EOFWhileParsingString,
    ControlCharacterInString,
    TrailingCharacters,
}

use self::ErrorCode::*;

#[derive(Debug)]
pub enum ParserError {
    ///         msg,      line,   col
    SyntaxError(ErrorCode, usize, usize),
    // IoError(io::Error),
}

/// Returns a readable error string for a given error code.
#[allow(dead_code)]
fn error_str(error: ErrorCode) -> &'static str {
    match error {
        InvalidSyntax         => "invalid syntax",
        InvalidNumber         => "invalid number",
        UnrecognizedBase64    => "Base64-encoded string can only include valid base64 characters",
        EOFWhileParsingList   => "EOF While parsing list",
        EOFWhileParsingString => "EOF While parsing string",
        _                     => "something else entirely"
    }
}
