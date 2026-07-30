use core::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownIdentifier(String),
    IO(io::Error),
    Redb(String),
    Fst(fst::Error),
    Codec(String),
}

impl Error {
    pub fn from_identifier(ident: impl Into<String>) -> Self {
        Error::UnknownIdentifier(ident.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownIdentifier(ident) => write!(f, "unknown identifier: {ident}"),
            Error::IO(e) => e.fmt(f),
            Error::Redb(msg) => write!(f, "database error: {msg}"),
            Error::Fst(e) => e.fmt(f),
            Error::Codec(msg) => write!(f, "codec error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::IO(v)
    }
}

impl From<redb::Error> for Error {
    fn from(v: redb::Error) -> Self {
        Self::Redb(v.to_string())
    }
}

impl From<redb::TableError> for Error {
    fn from(v: redb::TableError) -> Self {
        Self::Redb(v.to_string())
    }
}

impl From<redb::StorageError> for Error {
    fn from(v: redb::StorageError) -> Self {
        Self::Redb(v.to_string())
    }
}

impl From<redb::TransactionError> for Error {
    fn from(v: redb::TransactionError) -> Self {
        Self::Redb(v.to_string())
    }
}

impl From<redb::CommitError> for Error {
    fn from(v: redb::CommitError) -> Self {
        Self::Redb(v.to_string())
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(v: redb::DatabaseError) -> Self {
        Self::Redb(v.to_string())
    }
}

impl From<fst::Error> for Error {
    fn from(v: fst::Error) -> Self {
        Self::Fst(v)
    }
}

impl From<fst::automaton::LevenshteinError> for Error {
    fn from(v: fst::automaton::LevenshteinError) -> Self {
        Self::Codec(v.to_string())
    }
}

impl From<csv::Error> for Error {
    fn from(v: csv::Error) -> Self {
        Self::Codec(v.to_string())
    }
}
