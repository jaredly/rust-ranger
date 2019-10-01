use serde::{de, ser};
use std::error::Error as StdError;
use crate::ast::Pos;


#[derive(Debug, Clone)]
pub struct EvalError {
    pub desc: EvalErrorDesc,
    pub pos: Pos,
}

impl PartialEq for EvalError {
    fn eq(&self, other: &Self) -> bool {
        self.desc == other.desc
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum EvalErrorDesc {
    InvalidType(&'static str),
    MissingMember(String),
    CannotGetMember(String, &'static str),
    MissingReference(String),
    MemberMovedValue,
    FunctionValue,
    FunctionWrongNumberArgs(usize, usize),
    Unmatched,
}

impl From<EvalErrorDesc> for EvalError {
    fn from(other: EvalErrorDesc) -> Self {
        EvalError {
            desc: other,
            pos: Pos::default(),
        }
    }
}

impl EvalErrorDesc {
    pub fn with_pos(self, pos: Pos) -> EvalError {
        EvalError { desc: self, pos }
    }
}

#[derive(Clone, Debug)]
pub struct DeserializeError {
    pos: Pos,
    desc: DeserializeErrorDesc
}

impl PartialEq for DeserializeError {
    fn eq(&self, other: &Self) -> bool {
        self.desc == other.desc
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeserializeErrorDesc {
    Message(String),

    ExpectedStruct,
    ExpectedUnit,
    ExpectedMap,
    ExpectedArray,
    ExpectedEnum,
    ExpectedNamedTuple,
    ExpectedSequence,
    Unevaluated(String),
    EvalError(EvalError),
    WrongName(String, String),
    WrongTupleLength(usize, usize),
    // WrongStructLength(usize, usize),
}

impl DeserializeError {
    pub fn with_pos(self, pos: Pos) -> DeserializeError {
        DeserializeError { pos: if self.pos.is_empty() { pos } else { self.pos }, desc: self.desc }
    }
}

impl DeserializeErrorDesc {
    pub fn with_pos(self, pos: Pos) -> DeserializeError {
        DeserializeError { pos, desc: self }
    }
}

// use error::{Error, Result};
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    DeserializeError(DeserializeError),
    EvalError(EvalError),
    ParseError(pest::error::Error<crate::parser::Rule>),
    Syntax,
}

impl From<DeserializeError> for Error {
    fn from(other: DeserializeError) -> Self {
        Error::DeserializeError(other)
    }
}

impl From<DeserializeErrorDesc> for DeserializeError {
    fn from(desc: DeserializeErrorDesc) -> Self {
        DeserializeError { pos: Pos::default(), desc }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<EvalError> for Error {
    fn from(other: EvalError) -> Error {
        Error::EvalError(other)
    }
}

impl From<EvalError> for DeserializeError {
    fn from(other: EvalError) -> DeserializeError {
        let pos = other.pos;
        DeserializeErrorDesc::EvalError(other).with_pos(pos)
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

impl de::Error for DeserializeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeserializeErrorDesc::Message(msg.to_string()).with_pos(Pos::default())
    }
}

impl ser::Error for DeserializeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeserializeErrorDesc::Message(msg.to_string()).with_pos(Pos::default())
    }
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
        // match *self {
        //     Error::IoError(ref s) => write!(f, "{}", s),
        //     Error::Message(ref s) => write!(f, "{}", s),
        //     Error::Parser(_, pos) => write!(f, "{}: {}", pos, self.description()),
        // }
    }
}

impl StdError for DeserializeError {
    fn description(&self) -> &str {
        match &self.desc {
            DeserializeErrorDesc::ExpectedStruct => "Expected a struct",
            DeserializeErrorDesc::ExpectedUnit => "Expected a unit",
            DeserializeErrorDesc::ExpectedMap => "Expected a map",
            DeserializeErrorDesc::Unevaluated(_) => "Unevaluated",
            DeserializeErrorDesc::EvalError(_) => "Eval error",
            DeserializeErrorDesc::ExpectedArray => "Expected an array",
            DeserializeErrorDesc::ExpectedEnum => "Expected an enum",
            DeserializeErrorDesc::ExpectedSequence => "Expected a sequence",
            DeserializeErrorDesc::ExpectedNamedTuple => "Expected a named tuple",
            DeserializeErrorDesc::WrongName(_found, _expected) => "Wrong struct name",
            DeserializeErrorDesc::WrongTupleLength(_expected, _found) => "Wrong namedtuple length",
            // DeserializeErrorDesc::WrongStructLength(_expected, _found) => "Wrong struct length",
            DeserializeErrorDesc::Message(ref message) => message,
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match &self {
            Error::EvalError(_) => "Error while evaluating",
            Error::ParseError(_) => "Invalid syntax",
            Error::DeserializeError(err) => err.description(),
            // Error::Message(ref message) => &message,
            Error::Message(ref message) => message,
            // Error::WrongName(found, expected) => &format!("Wrong Struct name: Expected '{}', found '{}'", expected, found),
            // Error::WrongTupleLength(expected, found) => &format!("Wrong naamedtuple length: '{}', found '{}'", expected, found),
            Error::Syntax => "Unknown syntax error",
        }
    }
}
