use std::error;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

/// An Error type.
#[derive(Debug)]
pub enum Error {
    /// Denotes I/O error.
    Io(io::Error),
    /// Denotes error that produces this crate.
    Own(String),
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Own(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Own(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self::Own(e.to_string())
    }
}

impl From<trash::Error> for Error {
    fn from(e: trash::Error) -> Self {
        Self::Own(e.to_string())
    }
}
