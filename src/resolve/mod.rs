//! Resolves and prepares URI paths in Markdown source.
//!
//! Backend document processors generally expect files to reside locally in the file system.
//! Markdown allows links and images to resolve to generic urls. At the same time, we do not want
//! to allow all links and documents especially when potentially incorporating the internet as a
//! source of interpreted code such as includes.
//!
//! This module provides an interface for both problems. First, it allows resolution of an url to
//! an open read stream or to an auxiliary file. Secondly, this resolution will automatically apply
//! a restrictive-by-default filter and error when violating security boundaries.
use std::collections::HashMap;
use std::io;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use url::{Url, Origin, ParseError};

pub struct Resolver {
    grants: Grants,
    special_provider: HashMap<Origin, Box<DocumentProvider>>,
}

/// Manages additional request types explicitely allowed by command line options.
struct Grants {
}

pub trait DocumentProvider {
    fn build(&self, target: Url, builder: DocumentBuilder) -> io::Result<Document>;
}

struct Documents;

struct Locals;

struct Remotes;

pub struct DocumentBuilder {
    source: Source,
    mime: Option<String>,
}

/// Metadata about a data source.
///
/// Each document, either local or possible remote, is associated with a source identified by the
/// host tuple as established by browsers. A path can be retrieved for every document so that you
/// can refer to them even in other auxiliary files and generic programs. This operation
/// potentially stores the data stream in a cache or temporary file. Where available, it is also
/// possible to query an indication of the documents MIME-type.
pub struct Document {
    source: Source,
    resource: Resource,
}

/// Differentiates between sources based on their access right characteristics.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Source {
    inner: InnerSource,
}

#[derive(Clone, Eq, PartialEq, Hash)]
enum InnerSource {
    /// Implementation detail with vetted access.
    ///
    /// The burden is on this implemention to ensure that no confused deputy situation arises. This
    /// implements special such as `//TOC`. The `//document` host allows qccess to the all project
    /// local files and other files with user setup.
    Implementation(Url),

    /// Sandboxed file, access only to itself.
    ///
    /// Implemented with the `file` scheme.
    Local(Url),

    /// Any other arbitrary url.
    Remote(Url),
}

enum Resource {
    File(PathBuf),
    Abstract(Box<io::Read>),
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            grants: Default::default(),
            special_provider: Default::default(),
        }
    }

    pub fn add_provider(&mut self, origin: Origin, provider: Box<DocumentProvider>) {
        let previous = self.special_provider.insert(origin.clone(), provider);
        assert!(previous.is_none(), "Two providers for same origin {:?}", origin);
    }

    /// Make a request to an uri in the context of a document with the specified source.
    pub fn request(&self, source: &Source, url: &str) -> io::Result<Document> {
        let target = source.resolve(url)
            .map_err(|err| io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("Malformed reference: {:?}", err),
            ))?;

        source.check_access(&target, &self.grants)?;

        let builder = self.builder(&target);
        let provider = self.provider(&target);

        provider.build(target.as_url().clone(), builder)
    }

    pub fn provider(&self, target: &Source) -> &DocumentProvider {
        unimplemented!()
    }

    pub fn builder(&self, source: &Source) -> DocumentBuilder {
        DocumentBuilder {
            source: source.clone(),
            mime: None,
        }
    }
}

impl Grants {

}

impl DocumentBuilder {
    pub fn with_path<P: Into<PathBuf>>(self, path: P) -> Document {
        Document {
            source: self.source,
            resource: Resource::File(path.into()),
        }
    }

    pub fn with_reader(self, reader: Box<io::Read>) -> Document {
        Document {
            source: self.source,
            resource: Resource::Abstract(reader),
        }
    }
}

impl Document {
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// Return the origin of this URL (<https://url.spec.whatwg.org/#origin>)
    pub fn origin(&self) -> Origin {
        self.source.as_url().origin()
    }

    /// Return a backing file path if one has already been established.
    ///
    /// Documents that are backed by in-memory `std::io::Read` instances have no path until they
    /// are committed to a file with `into_file`.
    pub fn to_path(&self) -> Option<&Path> {
        match self.resource {
            Resource::File(ref path) => Some(path),
            _ => None,
        }
    }

    /// Turn this document in a readable data stream.
    ///
    /// This will not give a sensible result for all documents. Some documents might be filled
    /// during the processing of the markdown document. These will of course initially appear empty
    /// or contain data of previous runs.
    pub fn into_reader(self) -> io::Result<Box<io::Read>> {
        Ok(match self.resource {
            Resource::File(path) => Box::new(File::open(path)?),
            Resource::Abstract(io) => io,
        })
    }
}

impl Source {
    /// Construct the local top level source.
    pub fn document_root(&self) -> Source {
        Source {
            inner: InnerSource::Local("pundoc://document/".parse().unwrap()),
        }
    }

    /// Try to parse the source as one of the categories.
    pub fn from_url(url: Url) -> Source {
        let inner = if url.scheme() == "pundoc" {
            InnerSource::Implementation(url)
        } else if url.scheme() == "file" {
            InnerSource::Local(url)
        } else {
            InnerSource::Remote(url)
        };

        Source {
            inner 
        }
    }

    /// Resolve a reference in the context of this source.
    pub fn resolve(&self, reference: &str) -> Result<Source, ParseError> {
        self.as_url()
            .join(reference)
            .map(Self::from_url)
    }

    fn as_url(&self) -> &Url {
        match self.inner {
            InnerSource::Implementation(ref url) => url,
            InnerSource::Local(ref url) => url,
            InnerSource::Remote(ref url) => url,
        }
    }

    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user.  Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    fn check_access(&self, target: &Source, grants: &Grants) -> io::Result<()> {
        match (&self.inner, &target.inner) {
            | (InnerSource::Implementation(_), InnerSource::Implementation(_)) 
            | (InnerSource::Implementation(_), InnerSource::Remote(_)) 
            | (InnerSource::Remote(_), InnerSource::Remote(_))
                => Ok(()),
            | (InnerSource::Local(_), _) // Local may not access but itself
            | (InnerSource::Remote(_), _) // Remote sites may not access local
                => Err(io::ErrorKind::PermissionDenied.into()),
            | (InnerSource::Implementation(_), InnerSource::Local(ref target))
                => unimplemented!("Local access should be configurable"),
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Grants {
    fn default() -> Grants {
        Grants { }
    }
}

impl DocumentProvider for Documents {
    fn build(&self, target: Url, builder: DocumentBuilder) -> io::Result<Document> {
        unimplemented!("Return a reference to the file in the working directory")
    }
}

impl DocumentProvider for Locals {
    fn build(&self, target: Url, builder: DocumentBuilder) -> io::Result<Document> {
        let path = target.to_file_path().unwrap();
        Ok(builder.with_path(path))
    }
}

impl DocumentProvider for Remotes {
    fn build(&self, _target: Url, _builder: DocumentBuilder) -> io::Result<Document> {
        unimplemented!("Remote urls can not yet be used inside documents")
    }
}

