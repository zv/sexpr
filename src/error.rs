#![allow(non_snake_case)]
use self::ErrorCode::*;
use self::ParserError::*;
use self::DecoderError::*;

use std::{fmt};
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

#[derive(Debug)]
pub enum ParserError {
    ///         msg,      line,   col
    SyntaxError(ErrorCode, usize, usize),
    // IoError(io::Error),
}

impl PartialEq for ParserError {
    fn eq(&self, other: &ParserError) -> bool {
        match (self, other) {
            (&SyntaxError(msg0, line0, col0), &SyntaxError(msg1, line1, col1)) =>
                msg0 == msg1 && line0 == line1 && col0 == col1
        }
    }
}

impl StdError for ParserError {
    fn description(&self) -> &str { "failed to parse json" }
}


impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
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

//
// Encoder
//

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

//
// Decoder
//

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum DecoderError {
    ParserError(ParserError),
    ExpectedError(String, String),
    MissingFieldError(String),
    UnknownVariantError(String),
    ApplicationError(String),
    EOF,
}

impl StdError for DecoderError {
    fn description(&self) -> &str { "decoder error" }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            DecoderError::ParserError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<ParserError> for DecoderError {
    fn from(err: ParserError) -> DecoderError {
        ParserError(From::from(err))
    }
}
