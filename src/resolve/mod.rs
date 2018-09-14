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
use std::io;
use std::path::PathBuf;

use url::Url;

mod resource;
mod providers;

pub use self::resource::*;

use self::providers::ResourceProviderProvider;

pub struct Resolver {
    grants: Grants,
    provider_provider: ResourceProviderProvider,
}

/// Manages additional request types explicitely allowed by command line options.
#[derive(Default)]
struct Grants {
}

/// Context to resolve a Url in
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Context {
    source: Source,
}

/// Differentiate between sources based on their access right characteristics.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Source {
    inner: InnerSource
}

#[derive(Clone, Eq, PartialEq, Hash)]
enum InnerSource {
    /// Implementation detail with vetted access.
    ///
    /// The burden is on this implemention to ensure that no confused deputy situation arises. This
    /// implements special such as `//TOC`. The `//document` host allows access to the all project
    /// local files and other files with user setup.
    Implementation(Url),

    /// Sandboxed file, access only to itself.
    ///
    /// Implemented with the `file` scheme.
    Local(Url),

    /// Any other arbitrary url.
    Remote(Url),
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            grants: Default::default(),
            // TODO: correct base folder
            provider_provider: ResourceProviderProvider::new(PathBuf::from(".")),
        }
    }

    /// Make a request to an uri in the context of a document with the specified source.
    pub fn request(&self, context: &Context, url: &str) -> io::Result<Resource> {
        let target = context.as_url().join(url)
            .map_err(|err| io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("Malformed reference: {:?}", err),
            ))?;
        let target = Source::from(target);

        context.check_access(&target, &self.grants)?;

        let builder = ResourceBuilder::new(target.clone());
        let provider = self.provider_provider.get_provider(&target);

        provider
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("No handler providing url {:?}", target.as_url()),
            ))?
            .build(target.into(), builder)
    }

    pub fn builder(&self, source: Source) -> ResourceBuilder {
        ResourceBuilder::new(source)
    }
}

impl Context {
    /// Create a new Context from given Source.
    pub fn new(source: Source) -> Context {
        Context {
            source,
        }
    }

    /// Construct the local top level Context.
    pub fn document_root() -> Context {
        Self::new(Source::from_url("pundoc://document/".parse().unwrap()))
    }

    fn as_url(&self) -> &Url {
        self.source.as_url()
    }

    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user.  Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    fn check_access(&self, target: &Source, _grants: &Grants) -> io::Result<()> {
        match (&self.source.inner, &target.inner) {
            | (InnerSource::Implementation(_), InnerSource::Implementation(_))
            | (InnerSource::Implementation(_), InnerSource::Remote(_))
            | (InnerSource::Remote(_), InnerSource::Remote(_))
                => Ok(()),
            | (InnerSource::Local(_), _) // Local may not access but itself
            | (InnerSource::Remote(_), _) // Remote sites may not access local
                => Err(io::ErrorKind::PermissionDenied.into()),
            | (InnerSource::Implementation(_), InnerSource::Local(_target))
                => unimplemented!("Local access should be configurable"),
        }
    }
}

impl Source {
    pub fn as_url(&self) -> &Url {
        match &self.inner {
            InnerSource::Implementation(url) => url,
            InnerSource::Local(url) => url,
            InnerSource::Remote(url) => url,
        }
    }

    pub fn from_url(url: Url) -> Self {
        Self::from(url)
    }

    pub fn into_url(self) -> Url {
        self.into()
    }
}

impl From<Url> for Source {
    fn from(url: Url) -> Self {
        let inner = match url.scheme() {
            "pundoc" => InnerSource::Implementation(url),
            "file" => InnerSource::Local(url),
            _ => InnerSource::Remote(url),
        };
        Source {
            inner,
        }
    }
}

impl Into<Url> for Source {
    fn into(self) -> Url {
        match self.inner {
            InnerSource::Implementation(url) => url,
            InnerSource::Local(url) => url,
            InnerSource::Remote(url) => url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_resolves() {
        let resolver = Resolver::default();
        let top = Context::document_root();

        let main_doc = resolver.request(&top, "main.md")
            .expect("Failed to resolve direct path");
        let sibling = resolver.request(main_doc.source(), "image.png")
            .expect("Failed to resolve sibling file");

        assert_eq!(main_doc.to_path(), Some(Path::new("./main.md")));
        assert_eq!(sibling.to_path(), Some(Path::new("./image.png")));
    }

    #[test]
    fn domain_resolves() {
        struct Toc;

        impl ResourceProvider for Toc {
            fn build(&self, _target: Url, builder: ResourceBuilder) -> io::Result<Resource> {
                Ok(builder.build(Include::Command(Command::Toc)))
            }
        }

        let mut resolver = Resolver::default();
        let top = Source::document_root();
        let main_doc = resolver.request(&top, "main.md")
            .expect("Failed to resolve direct path");

        resolver.add_provider("toc", Box::new(Toc));
        let toc = resolver.request(main_doc.source(), "//toc")
            .expect("Failed to resolve path in different domain");

        assert_eq!(toc.include(), &Include::Command(Command::Toc));
    }
}

