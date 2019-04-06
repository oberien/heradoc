use std::io;
use std::result;
use std::fmt;

#[must_use]
pub type Result<T> = result::Result<T, Error>;
#[must_use]
pub type FatalResult<T> = result::Result<T, Fatal>;

#[must_use]
#[derive(Debug)]
pub enum Fatal {
    /// Output file write error.
    Output(io::Error),
}

impl From<io::Error> for Fatal {
    fn from(err: io::Error) -> Self {
        Fatal::Output(err)
    }
}

impl std::error::Error for Fatal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Fatal::Output(io) => Some(io),
        }
    }
}

impl fmt::Display for Fatal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fatal::Output(io) => write!(f, "output file write error: {}", io),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub enum Error {
    /// Fatal unrecoverable error, but somewhat expected.
    Fatal(Fatal),
    /// Diagnostic was printed, event wasn't handled, skip over the event.
    ///
    /// This means that the error was already handled internally and is used to inform the caller
    /// about it. It allows the generator to skip over events, if an error happens e.g. in an
    /// `Event::Start`.
    Diagnostic,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Fatal(err.into())
    }
}

impl From<Fatal> for Error {
    fn from(err: Fatal) -> Self {
        Error::Fatal(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Fatal(fatal) => Some(fatal),
            Error::Diagnostic => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fatal(fatal) => write!(f, "fatal error: {}", fatal),
            Error::Diagnostic => write!(f, "error during event handling, diagnostic written"),
        }
    }
}
