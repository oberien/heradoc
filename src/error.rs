use std::io;
use std::result;
use std::fmt;
use diagnostic::ErrorCode;

pub type Result<T> = result::Result<T, Error>;
pub type FatalResult<T> = result::Result<T, Fatal>;

#[must_use]
#[derive(Debug)]
pub enum Fatal {
    /// Output file write error.
    Output(io::Error),

    /// An unrecoverable internal error (ICE).
    ///
    /// Used for assertions that are made but not completely or badly proven. For cases where this
    /// can be related directly to an particular structure in an input file this error also
    /// provides opportunities to log the relevant context in diagnostics as opposed to `unwrap` or
    /// `expect` panicking.  For example this may occur due to an interface of a dependency being
    /// more general than its usage herein but we can not expect full control over its
    /// implementation.
    InternalCompilerError,
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
            Fatal::InternalCompilerError => None,
        }
    }
}

impl fmt::Display for Fatal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fatal::Output(io) => write!(f, "output file write error: {}", io),
            Fatal::InternalCompilerError => write!(f, "can not continue due to internal error"),
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

#[derive(Clone)]
pub enum DiagnosticCode {
    FoundTwoLabels,
    InvalidFigureValue,
    MultipleFigureKeys,
    InvalidCskvp,
    InvalidCskv,
    NotYetImplemented,
    UnapplicableElementConfig,
    MultipleLabels,
    InvalidReference,
    CodeHasMultipleConfigs,
    UnknownAttributesInElementConfig,
    InvalidDoubleQuotedString,
    ErrorReadingIncludedMarkdownFile,
    ErrorReadingGraphvizFile,
    ErrorResolvingFile,
    UnsupportedDomain,
    ErrorCanonicalizingPath,
    PermissionDenied,
    InvalidUrl,
    InvalidCommand,
    ErrorWritingToCache,
    ErrorDownloadingContent,
    UnknownFileFormat,
    MissingFileExtension,

    GraphvizError,
    TempFileError,
    InvalidConfigPath(String),
    InvalidHeaderLevel,
    Unsupported,
    SvgConversionError,
    EspeakCreationError,

    InternalCompilerError,
}

impl ErrorCode for DiagnosticCode {
    fn code(&self) -> String {
        match self {
            DiagnosticCode::FoundTwoLabels => "0001",
            DiagnosticCode::InvalidFigureValue => "0002",
            DiagnosticCode::MultipleFigureKeys => "0003",
            DiagnosticCode::InvalidCskvp => "0004",
            DiagnosticCode::InvalidCskv => "0005",
            DiagnosticCode::NotYetImplemented => "0006",
            DiagnosticCode::UnapplicableElementConfig => "0007",
            DiagnosticCode::MultipleLabels => "0008",
            DiagnosticCode::InvalidReference => "0009",
            DiagnosticCode::CodeHasMultipleConfigs => "0010",
            DiagnosticCode::UnknownAttributesInElementConfig => "0011",
            DiagnosticCode::InvalidDoubleQuotedString => "0012",
            DiagnosticCode::ErrorReadingIncludedMarkdownFile => "0013",
            DiagnosticCode::ErrorReadingGraphvizFile => "0014",
            DiagnosticCode::ErrorResolvingFile => "0015",
            DiagnosticCode::UnsupportedDomain => "0016",
            DiagnosticCode::ErrorCanonicalizingPath => "0017",
            DiagnosticCode::PermissionDenied => "0018",
            DiagnosticCode::InvalidUrl => "0019",
            DiagnosticCode::InvalidCommand => "0020",
            DiagnosticCode::ErrorWritingToCache => "0021",
            DiagnosticCode::ErrorDownloadingContent => "0022",
            DiagnosticCode::UnknownFileFormat => "0023",
            DiagnosticCode::MissingFileExtension => "0024",

            DiagnosticCode::GraphvizError => "1000",
            DiagnosticCode::TempFileError => "1001",
            DiagnosticCode::InvalidConfigPath(_) => "1002",
            DiagnosticCode::InvalidHeaderLevel => "1003",
            DiagnosticCode::Unsupported => "1004",
            DiagnosticCode::SvgConversionError => "1005",
            DiagnosticCode::EspeakCreationError => "1006",

            DiagnosticCode::InternalCompilerError => "9999",
        }.to_string()
    }

    fn message(&self) -> String {
        match self {
            DiagnosticCode::FoundTwoLabels => "found two labels".to_string(),
            DiagnosticCode::InvalidFigureValue => "invalid figure attribute value".to_string(),
            DiagnosticCode::MultipleFigureKeys => "found multiple figure keys".to_string(),
            DiagnosticCode::InvalidCskvp => "invalid comma seperated key-value-pair".to_string(),
            DiagnosticCode::InvalidCskv => "invalid comma seperated value".to_string(),
            DiagnosticCode::NotYetImplemented => "not yet implemented".to_string(),
            DiagnosticCode::UnapplicableElementConfig => "found element config, but there wasn't an element to apply it to".to_string(),
            DiagnosticCode::MultipleLabels => "multiple labels for the same element".to_string(),
            DiagnosticCode::InvalidReference => "found biber reference, but no bibliography file found".to_string(),
            DiagnosticCode::CodeHasMultipleConfigs => "code has both prefix and inline style config".to_string(),
            DiagnosticCode::UnknownAttributesInElementConfig => "unknown attributes in element config".to_string(),
            DiagnosticCode::InvalidDoubleQuotedString => "invalid double-quoted string".to_string(),
            DiagnosticCode::ErrorReadingIncludedMarkdownFile => "error reading markdown include file".to_string(),
            DiagnosticCode::ErrorReadingGraphvizFile => "error reading graphviz file".to_string(),
            DiagnosticCode::ErrorResolvingFile => "error resolving file".to_string(),
            DiagnosticCode::UnsupportedDomain => "unsupported domain".to_string(),
            DiagnosticCode::ErrorCanonicalizingPath => "error canonicalizing path".to_string(),
            DiagnosticCode::PermissionDenied => "permission denied".to_string(),
            DiagnosticCode::InvalidUrl => "invalid URL".to_string(),
            DiagnosticCode::InvalidCommand => "invalid command".to_string(),
            DiagnosticCode::ErrorWritingToCache => "error writing to cache".to_string(),
            DiagnosticCode::ErrorDownloadingContent => "error downloading content".to_string(),
            DiagnosticCode::UnknownFileFormat => "unknown file format".to_string(),
            DiagnosticCode::MissingFileExtension => "missing file extension".to_string(),

            DiagnosticCode::GraphvizError => "graphviz rendering failed".to_string(),
            DiagnosticCode::TempFileError => "error creating temporary file".to_string(),
            DiagnosticCode::InvalidConfigPath(name) => format!("invalid path to `{}` in the config", name),
            DiagnosticCode::InvalidHeaderLevel => "invalid header level".to_string(),
            DiagnosticCode::Unsupported => "unsupported".to_string(),
            DiagnosticCode::SvgConversionError => "error converting svg".to_string(),
            DiagnosticCode::EspeakCreationError => "error creating espeak file".to_string(),

            DiagnosticCode::InternalCompilerError => "internal compiler error".to_string(),
        }
    }
}

pub(crate) type Diagnostics = diagnostic::Diagnostics<DiagnosticCode>;
