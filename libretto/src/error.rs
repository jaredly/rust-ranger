use serde::{de, ser};
use std::error::Error as StdError;

// use error::{Error, Result};
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ExpectedStruct,
    ExpectedUnit,
    ExpectedMap,
    ExpectedArray,
    ExpectedEnum,
    ExpectedNamedTuple,
    ExpectedSequence,
    Message(String),
    WrongName(String, String),
    WrongTupleLength(usize, usize),
    Unevaluated(String),
    EvalError(crate::ast::EvalError),
    ParseError(pest::error::Error<crate::parser::Rule>),
    Syntax,
}
pub type Result<T> = std::result::Result<T, Error>;

impl From<crate::ast::EvalError> for Error {
    fn from(other: crate::ast::EvalError) -> Error {
        Error::EvalError(other)
    }
}

impl From<pest::error::Error<crate::parser::Rule>> for Error {
    fn from(other: pest::error::Error<crate::parser::Rule>) -> Error {
        Error::ParseError(other)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
        // match *self {
        //     Error::IoError(ref s) => write!(f, "{}", s),
        //     Error::Message(ref s) => write!(f, "{}", s),
        //     Error::Parser(_, pos) => write!(f, "{}: {}", pos, self.description()),
        // }
    }
}

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match &self {
            Error::EvalError(_) => "Error while evaluating",
            Error::ParseError(_) => "Invalid syntax",
            Error::ExpectedStruct => "Expected a struct",
            Error::ExpectedUnit => "Expected a unit",
            Error::ExpectedMap => "Expected a map",
            Error::Unevaluated(_) => "Unevaluated",
            Error::ExpectedArray => "Expected an array",
            Error::ExpectedEnum => "Expected an enum",
            Error::ExpectedSequence => "Expected a sequence",
            Error::ExpectedNamedTuple => "Expected a named tuple",
            // Error::Message(ref message) => &message,
            Error::Message(ref message) => message,
            Error::WrongName(_found, _expected) => "Wrong struct name",
            Error::WrongTupleLength(_expected, _found) => "Wrong namedtuple length",
            // Error::WrongName(found, expected) => &format!("Wrong Struct name: Expected '{}', found '{}'", expected, found),
            // Error::WrongTupleLength(expected, found) => &format!("Wrong naamedtuple length: '{}', found '{}'", expected, found),
            Error::Syntax => "Unknown syntax error",
        }
    }
}
