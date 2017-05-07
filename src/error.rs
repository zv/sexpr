#![allow(non_snake_case)]
use self::ErrorCode::*;
use self::ParserError::*;
use self::DecoderError::*;

use std;
use std::fmt::{self, Display};
use std::error::Error as StdError;

use serde::{ser, de};


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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidType {
    ExpectingList,
    ExpectingPair,
    ExpectingNumber
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Errors for handling various selections
pub enum SexpError {
    InvalidType(InvalidType),
    NotFound
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
pub type SerdeResult<T> = Result<T, SerdeError>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Clone, Debug, PartialEq)]
pub enum SerdeError {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl ser::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string())
    }
}

impl de::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string())
    }
}

impl Display for SerdeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for SerdeError {
    fn description(&self) -> &str {
        "something is wrong"
    }
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
