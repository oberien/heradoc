use std::io::Error as IoError;
use std::ops::Range;

/// Wraps an io error with additional position information.
///
/// The error can be converted from and to io error, adding no information.
#[derive(Debug)]
pub struct Error {
    /// The underlying error.
    io: IoError,

    /// Span information if supplied. The offsets are arbitrarily chosen by the event supplier.
    span: Option<Range<usize>>,

    /// The markdown source code that cause this error. 
    ///
    /// TODO: this could be a Cow but that would introduce a lifetime and complicated things.
    source: Option<String>,
}

impl Error {
    /// The parser position where this error occurred, if such information exists.
    pub fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }

    pub fn with_span(self, span: Range<usize>) -> Self {
        Error {
            span: Some(span),
            .. self
        }
    }

    /// Add information about the part in the source that cause this error.
    pub fn with_source<S: Into<String>>(self, source: S) -> Self {
        Error {
            source: Some(source.into()),
            .. self
        }
    }

    /// Copy from the source the range that was given as span information.
    pub fn with_source_span<S: AsRef<str>>(mut self, source: S) -> Self {
        if let Some(span) = self.span() {
            self.source = Some(source.as_ref()[span].to_owned());
        }
        
        self
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for IoError {
    fn from(err: Error) -> Self {
        err.io
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error {
            io: err,
            span: None,
            source: None,
        }
    }
}

