#![allow(non_snake_case)]

use std::fmt;
use std::error::Error as StdError;


/// The errors that can arise while parsing a S-expression stream.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidAtom,
    InvalidNumber,
    InvalidEscape,
    UnbalancedClosingParen,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntoAlistError {
    ContainerSexpNotList,
    KeyValueMustBePair,
    CannotStringifyKey,
    DuplicateKey
}


#[derive(Copy, Debug)]
/// Returns a readable error for `Encodable` faults
pub enum EncoderError {
    FmtError(fmt::Error),
    #[allow(dead_code)]
    BadHashmapKey,
}

impl PartialEq for EncoderError {
    fn eq(&self, other: &EncoderError) -> bool {
        match (*self, *other) {
            (EncoderError::FmtError(_), EncoderError::FmtError(_)) => true,
            (EncoderError::BadHashmapKey, EncoderError::BadHashmapKey) => true,
            _ => false,
        }
    }
}

impl Clone for EncoderError {
    fn clone(&self) -> Self { *self }
}

impl StdError for EncoderError {
    fn description(&self) -> &str { "encoder error" }
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<fmt::Error> for EncoderError {
    fn from(err: fmt::Error) -> EncoderError { EncoderError::FmtError(err) }
}

