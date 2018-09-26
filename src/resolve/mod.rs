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
use std::path::{Path, PathBuf};
use std::env;

use url::Url;

mod include;
mod source;

pub use self::include::*;
use self::source::{Source, SourceGroup};

pub struct Resolver {
    base: Url,
    permissions: Permissions,
}

/// Manages permissions if includes as allowed explicitly from the Cli.
struct Permissions {
    allowed_absolute_folders: Vec<PathBuf>,
}

impl Resolver {
    pub fn new(workdir: PathBuf) -> Self {
        Resolver {
            base: Url::parse("pundoc://document/").unwrap(),
            permissions: Permissions {
                allowed_absolute_folders: vec![workdir],
            },
        }
    }

    /// Make a request to an uri in the context of a document with the specified source.
    pub fn request(&self, context: &Context, url: &str) -> io::Result<Include> {
        let url = self.base.join(url)
            .map_err(|err| io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("Malformed reference: {:?}", err),
            ))?;

        let target = Source::from_url(url, context)?;
        // check if context is allowed to access target
        self.check_access(context, &target)?;

        target.into_include()
    }

    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user.  Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    fn check_access(&self, context: &Context, target: &Source) -> io::Result<()> {
        match (context, &target.group) {
            (Context::LocalRelative(_), SourceGroup::Implementation)
            | (Context::LocalRelative(_), SourceGroup::LocalRelative(_))
            | (Context::LocalRelative(_), SourceGroup::Remote) => Ok(()),

            (Context::LocalAbsolute(_), SourceGroup::Implementation) => Ok(()),
            (Context::LocalAbsolute(_), SourceGroup::LocalRelative(_))
            | (Context::LocalAbsolute(_), SourceGroup::Remote)
                => Err(io::Error::new(io::ErrorKind::PermissionDenied,
                    "Local absolute path not allowed to access remote file")),

            (_, SourceGroup::LocalAbsolute(path)) => {
                if self.permissions.allowed_absolute_folders.contains(path) {
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::PermissionDenied,
                        format!("Not allowed to access absolute path {:?}", path)))
                }
            }

            // TODO: think about proper remote rules
            (Context::Remote, SourceGroup::Remote) => Ok(()),
            (Context::Remote, _) => Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "Remote can only access remote")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Context {
    LocalRelative(PathBuf),
    LocalAbsolute(PathBuf),
    Remote,
}

impl Context {
    fn path(&self) -> Option<&Path> {
        match self {
            Context::LocalRelative(path) => Some(&path),
            Context::LocalAbsolute(path) => Some(&path),
            Context::Remote => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempdir::TempDir;

    fn prepare() -> TempDir {
        let dir = TempDir::new("pundoc-test")
            .expect("Can't create tempdir");
        let _ = File::create(dir.path().join("main.md"))
            .expect("Can't create main.md");
        let _ = File::create(dir.path().join("test.md"))
            .expect("Can't create main.md");
        let _ = File::create(dir.path().join("image.png"))
            .expect("Can't create image.png");
        let _ = File::create(dir.path().join("pdf.pdf"))
            .expect("Can't create pdf.pdf");
        dir
    }

    #[test]
    fn standard_resolves() {
        let dir = prepare();
        let resolver = Resolver::new(PathBuf::from("."));
        let top = Context::LocalRelative(Path::new(dir.path()).canonicalize().unwrap());

        let main = resolver.request(&top, "main.md")
            .expect("Failed to resolve direct path");
        let sibling = resolver.request(main.context().unwrap(), "image.png")
            .expect("Failed to resolve sibling file");

        assert_eq!(main.path(), Some(dir.path().join("main.md").as_ref()));
        assert_eq!(sibling.path(), Some(dir.path().join("image.png").as_ref()));
        drop(dir);
    }

    #[test]
    fn domain_resolves() {
        let dir = prepare();
        let mut resolver = Resolver::new(PathBuf::from("."));
        let top = Context::LocalRelative(Path::new(dir.path()).canonicalize().unwrap());
        let main = resolver.request(&top, "main.md")
            .expect("Failed to resolve direct path");

        let toc = resolver.request(main.context().unwrap(), "//toc")
            .expect("Failed to resolve path in different domain");

        assert_eq!(toc, Include::Command(Command::Toc));
        drop(dir);
    }
}

