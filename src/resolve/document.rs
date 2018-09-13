//! Result type of the resolution of a file include.
use std::path::{Path, PathBuf};
use super::{Origin, Source};

/// Metadata about a data source.
///
/// Each document, either local or possible remote, is associated with a source identified by the
/// host tuple as established by browsers. A path can be retrieved for every document so that you
/// can refer to them even in other auxiliary files and generic programs. This operation
/// potentially stores the data stream in a cache or temporary file. Each resource has one of the
/// include types.
pub struct Document {
    source: Source,
    resource: Include,
}

pub struct DocumentBuilder {
    source: Source,
}

/// Typed representation of the resolved resource.
///
/// This is matched on by the `Generator` to call the respective appropriate handler
#[derive(Debug, PartialEq, Eq)]
pub enum Include {
    Command(Command),
    Markdown(PathBuf),
    Image(PathBuf, ImageMeta),
    Pdf(PathBuf, PdfMeta),
}

/// A direct command to the pundoc processor.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Table of contents.
    Toc,
}

/// Additional meta data about an image.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImageMeta {
    pub width: Option<String>,
    pub height: Option<String>,
}

/// Additional available meta data about a pdf.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PdfMeta;

impl Document {
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Return the origin of this URL (<https://url.spec.whatwg.org/#origin>)
    pub fn origin(&self) -> Origin {
        self.source.as_url().origin()
    }

    /// Return a backing file path if the include type has one.
    pub fn to_path(&self) -> Option<&Path> {
        self.resource.to_path()
    }

    /// The type of include that this document represents.
    pub fn include(&self) -> &Include {
        &self.resource
    }
}

impl DocumentBuilder {
    pub(super) fn new(source: Source) -> Self {
        DocumentBuilder {
            source,
        }
    }

    pub fn build(self, include: Include) -> Document {
        Document {
            source: self.source,
            resource: include,
        }
    }

    pub fn with_path<P: Into<PathBuf>>(self, path: P) -> Document {
        // TODO: deduce more types.
        self.build(Include::Image(path.into(), ImageMeta::default()))
    }

    // TODO: save to (cached) temporary file.
    // TODO: method to query cache for http downloader?
}

impl Include {
    /// Return a backing file path if this is backed by a file.
    pub fn to_path(&self) -> Option<&Path> {
        match &self {
            | Include::Markdown(path)
            | Include::Image(path, _)
            | Include::Pdf(path, _)
                => Some(path),
            Include::Command(_) => None,
        }
    }
}
