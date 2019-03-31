use std::io;
use std::result;

#[must_use]
pub type Result<T> = result::Result<T, Error>;
#[must_use]
pub type FatalResult<T> = result::Result<T, Fatal>;

#[must_use]
pub enum Fatal {
    /// Output file write error.
    Output(io::Error),
}

impl From<io::Error> for Fatal {
    fn from(err: io::Error) -> Self {
        Fatal::Output(err)
    }
}

#[must_use]
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
