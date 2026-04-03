use std::{
    num::{IntErrorKind, ParseIntError},
    str::ParseBoolError,
};

use nom::error::{ErrorKind as NomEk, ParseError as NomParseError};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Error {
    pub kind: ErrorKind,
    pub position: usize,
}

impl Error {
    pub fn new(kind: ErrorKind, position: usize) -> Self {
        Self { kind, position }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum ErrorKind {
    InvalidCommand(String),
    UnexpectedToken {
        found: String,
        expected: String,
    },
    InvalidSyntax(String),
    ParseFailure(String),
    DuplicateIdent(String, IdentType),
    MissingIdent(String, IdentType),
    MissingPrimaryKey(String),
    InvalidForeignKey {
        table: String,
        column: Option<String>,
    },
    DuplicateConstraint(String),
    InvalidConstraint(String),
    InvalidIndexMethod(String),
    InvalidStorageParam {
        key: String,
        value: String,
    },
    InvalidValue(String),
    NonWhitespace(String),
    #[default]
    Unknown,
    UnexpectedEOF,
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
            ErrorKind::NonWhitespace(s) => {
                write!(f, "Expected whitespace\nFound: {}", s)
            }
            ErrorKind::Unknown => write!(f, "Unknown error"),
        }?;

        write!(f, "\nAt byte: {} in input SQL", self.position)?;

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
                found: input.next_token().to_string(),
                expected: "<KEYWORD>".into(),
            },

            NomEk::Count | NomEk::ManyTill => {
                ErrorKind::ParseFailure(input.into())
            }

            NomEk::AlphaNumeric | NomEk::Alpha => ErrorKind::UnexpectedToken {
                found: input.next_token().to_string(),
                expected: "<IDENTIFIER>".into(),
            },

            NomEk::Digit => ErrorKind::UnexpectedToken {
                found: input.next_token().to_string(),
                expected: "<NUMBER>".into(),
            },

            NomEk::OneOf => {
                ErrorKind::DuplicateIdent(input.into(), IdentType::Column)
            }

            _ => ErrorKind::Unknown,
        };

        Self::new(error_kind, 0)
    }

    fn append(_: &'a str, _kind: NomEk, other: Self) -> Self {
        other
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
        Self::new(ErrorKind::ParseFailure(msg.into()), 0)
    }
}

impl From<ParseBoolError> for Error {
    fn from(_: ParseBoolError) -> Self {
        Self::new(ErrorKind::ParseFailure("invalid boolean value".into()), 0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) trait ParserExt<T> {
    fn map_into(self, kind: ErrorKind, position: usize) -> Result<T>;
}

impl<'a, T, E: NomParseError<&'a str>> ParserExt<T>
    for std::result::Result<T, E>
{
    fn map_into(self, kind: ErrorKind, position: usize) -> Result<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Error::new(kind, position)),
        }
    }
}

pub(crate) trait StrExt<'a> {
    fn next_token(&'a self) -> &'a str;
}

impl<'a> StrExt<'a> for str {
    fn next_token(&'a self) -> &'a str {
        self.split_whitespace().next().unwrap_or_default()
    }
}
