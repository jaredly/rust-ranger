

use serde::de::{
    self,
};
use std::error::{Error as StdError};

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
    Syntax
}
pub type Result<T> = std::result::Result<T, Error>;


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

impl StdError for Error {
    fn description(&self) -> &str {
        match &self {
          Error::ExpectedStruct => "Expected a struct",
          Error::ExpectedUnit => "Expected a unit",
          Error::ExpectedMap => "Expected a map",
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
          Error::Syntax => "Unknown syntax error"
        }
    }
}
