use core::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownIdentifier(String),
    IO(io::Error),
    Tantivy(tantivy::TantivyError),
}

impl Error {
    pub fn from_identifier(ident: impl Into<String>) -> Self {
        Error::UnknownIdentifier(ident.into())
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::IO(v)
    }
}

impl From<tantivy::TantivyError> for Error {
    fn from(v: tantivy::TantivyError) -> Self {
        Self::Tantivy(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownIdentifier(ident) => write!(f, "unknown identifier: {ident}"),
            Error::IO(e) => e.fmt(f),
            Error::Tantivy(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
