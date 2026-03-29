use std::{
    num::{IntErrorKind, ParseIntError},
    str::ParseBoolError,
};

use nom::error::{ErrorKind as NomEk, ParseError as NomParseError};

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub context: String,
}

impl Error {
    pub fn new(kind: ErrorKind, context: impl Into<String>) -> Self {
        Self { kind, context: context.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    InvalidCommand(String),
    UnexpectedToken { found: String, expected: String },
    UnexpectedEOF,
    InvalidSyntax(String),
    ParseFailure(String),
    DuplicateIdent(String, IdentType),
    MissingIdent(String, IdentType),
    MissingPrimaryKey(String),
    InvalidForeignKey { table: String, column: Option<String> },
    DuplicateConstraint(String),
    InvalidConstraint(String),
    InvalidIndexMethod(String),
    InvalidStorageParam { key: String, value: String },
    InvalidValue(String),
    Unknown(String),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum IdentType {
    Table,
    Column,
}

impl std::fmt::Display for IdentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if *self == Self::Table { "table" } else { "column" })
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::InvalidCommand(cmd) => {
                write!(
                    f,
                    "Invalid command: {}\nOnly `CREATE` statements are accepted",
                    cmd
                )
            }
            ErrorKind::UnexpectedToken { found, expected } => {
                write!(f, "Unexpected token: {}\nExpected: {}", found, expected)
            }
            ErrorKind::UnexpectedEOF => f.write_str("Unexpected EOF"),
            ErrorKind::InvalidSyntax(s) => write!(f, "Invalid syntax: {}", s),
            ErrorKind::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            ErrorKind::ParseFailure(s) => write!(f, "Failed to parse: {}", s),
            ErrorKind::DuplicateIdent(s, ty) => {
                write!(f, "Duplicate {}: {}", ty, s)
            }
            ErrorKind::MissingIdent(s, ty) => {
                write!(f, "Missing {}: {}", ty, s)
            }
            ErrorKind::MissingPrimaryKey(table) => {
                write!(f, "Table '{}' has no primary key", table)
            }
            ErrorKind::InvalidForeignKey { table, column } => match column {
                Some(col) => write!(
                    f,
                    "Foreign key references non-existent column '{}.{}'",
                    table, col
                ),
                None => write!(
                    f,
                    "Foreign key references non-existent table '{}'",
                    table
                ),
            },
            ErrorKind::DuplicateConstraint(s) => {
                write!(f, "Duplicate constraint: {}", s)
            }
            ErrorKind::InvalidConstraint(s) => {
                write!(f, "Invalid constraint: {}", s)
            }
            ErrorKind::InvalidIndexMethod(s) => {
                write!(f, "Invalid index method: {}", s)
            }
            ErrorKind::InvalidStorageParam { key, value } => {
                write!(f, "Invalid storage parameter '{}' = '{}'", key, value)
            }
            ErrorKind::Unknown(s) => write!(f, "Unknown error: {}", s),
        }?;

        write!(f, "\nContext:\n  {}", self.context)?;

        Ok(())
    }
}

impl<'a> NomParseError<&'a str> for Error {
    fn from_error_kind(input: &'a str, kind: NomEk) -> Self {
        let error_kind = match kind {
            NomEk::MultiSpace
            | NomEk::Space
            | NomEk::TakeWhile1
            | NomEk::Eof => ErrorKind::UnexpectedEOF,

            NomEk::Tag => ErrorKind::UnexpectedToken {
                found: input
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                expected: "<KEYWORD>".into(),
            },

            NomEk::Count | NomEk::ManyTill => {
                ErrorKind::ParseFailure(input.into())
            }

            NomEk::AlphaNumeric | NomEk::Alpha => ErrorKind::UnexpectedToken {
                found: input
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                expected: "<IDENTIFIER>".into(),
            },

            NomEk::Digit => ErrorKind::UnexpectedToken {
                found: input
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                expected: "<NUMBER>".into(),
            },

            NomEk::OneOf => {
                ErrorKind::DuplicateIdent(input.into(), IdentType::Column)
            }

            _ => ErrorKind::Unknown(input.into()),
        };

        Self::new(error_kind, input.to_string())
    }

    fn append(input: &'a str, _kind: NomEk, mut other: Self) -> Self {
        other.context.push_str(input);
        other
    }
}

impl From<nom::Err<Error>> for Error {
    fn from(e: nom::Err<Error>) -> Self {
        match e {
            nom::Err::Incomplete(_) => Self::new(ErrorKind::UnexpectedEOF, ""),
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
        }
    }
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(e: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match e {
            nom::Err::Incomplete(_) => Self::new(ErrorKind::UnexpectedEOF, ""),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                Self::from_error_kind(e.input, e.code)
            }
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        let msg = match value.kind() {
            IntErrorKind::Empty => "empty input",
            IntErrorKind::Zero => "zero not allowed",
            IntErrorKind::PosOverflow => "number too large",
            IntErrorKind::NegOverflow => "number too small",
            _ => "invalid number",
        };
        Self::new(
            ErrorKind::ParseFailure(msg.into()),
            value.to_string().as_str(),
        )
    }
}

impl From<ParseBoolError> for Error {
    fn from(value: ParseBoolError) -> Self {
        Self::new(
            ErrorKind::ParseFailure("invalid boolean value".into()),
            value.to_string().as_str(),
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) trait ParserExt<T> {
    fn map_into(self, kind: ErrorKind, ctx: impl Into<String>) -> Result<T>;
}

impl<'a, T, E: NomParseError<&'a str>> ParserExt<T>
    for std::result::Result<T, E>
{
    fn map_into(self, kind: ErrorKind, ctx: impl Into<String>) -> Result<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Error::new(kind, ctx)),
        }
    }
}
