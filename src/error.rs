use failure::Fail;
use std::{io, string::FromUtf8Error};


/// Error type for kvs
#[derive(Fail, Debug)]
pub enum KvsError {
    /// Error with a string message
    #[fail(display = "{}", _0)]
    StringError(String),

    /// Utf8 error
    #[fail(display = "{}", _0)]
    Utf8Error(String),

    /// IO error.
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    /// Sled error.
    #[fail(display = "{}", _0)]
    Sled(#[cause] sled::Error),

    /// Serialization or deserialization error.
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),

    /// Removing non-existing key error.
    #[fail(display = "Key not found")]
    KeyNotFound,

    /// Unexpected command type error.
    /// It indicates a corrupted log or a program bug.
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

impl From<sled::Error> for KvsError {
    fn from(err: sled::Error) -> KvsError {
        KvsError::Sled(err)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(err: serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(err: FromUtf8Error) -> KvsError {
        KvsError::Utf8Error(err.to_string())
    }
}

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;

