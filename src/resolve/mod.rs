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
use std::path::{Path, PathBuf};

use url::Url;

mod include;
pub mod remote;
mod source;

pub use self::include::*;
use self::remote::Remote;
use self::source::{Source, SourceGroup};
use crate::diagnostics::Diagnostics;
use crate::error::{Error, Result};
use crate::frontend::range::SourceRange;

pub struct Resolver {
    base: Url,
    permissions: Permissions,
    remote: Remote,
}

/// Manages permissions if includes as allowed explicitly from the Cli.
struct Permissions {
    allowed_absolute_folders: Vec<PathBuf>,
}

impl Resolver {
    pub fn new(workdir: PathBuf, tempdir: PathBuf) -> Self {
        Resolver {
            base: Url::parse("heradoc://document/").unwrap(),
            permissions: Permissions { allowed_absolute_folders: vec![workdir] },
            remote: Remote::new(tempdir).unwrap(),
        }
    }

    /// Make a request to an uri in the context of a document with the specified source.
    pub fn resolve(
        &self, context: &Context, url: &str, range: SourceRange, diagnostics: &Diagnostics<'_>,
    ) -> Result<Include> {
        let url = match context.as_heradoc_url().join(url) {
            Ok(url) => url,
            Err(err) => {
                diagnostics
                    .error("couldn't resolve file")
                    .with_error_section(range, "defined here")
                    .note(format!("tried to resolve {}", url))
                    .note(format!("malformed reference: {}", err))
                    .emit();
                return Err(Error::Diagnostic);
            },
        };

        let target = Source::new(url, context, range, diagnostics)?;
        // check if context is allowed to access target
        self.check_access(context, &target, range, diagnostics)?;

        target.into_include(&self.remote, range, diagnostics)
    }

    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user.  Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    fn check_access(
        &self, context: &Context, target: &Source, range: SourceRange,
        diagnostics: &Diagnostics<'_>,
    ) -> Result<()> {
        match (context, &target.group) {
            (Context::LocalRelative(_), SourceGroup::Implementation)
            | (Context::LocalRelative(_), SourceGroup::LocalRelative(_))
            | (Context::LocalRelative(_), SourceGroup::Remote) => Ok(()),

            (Context::LocalAbsolute(_), SourceGroup::Implementation) => Ok(()),
            (Context::LocalAbsolute(_), SourceGroup::LocalRelative(_))
            | (Context::LocalAbsolute(_), SourceGroup::Remote) => {
                diagnostics
                    .error("permission denied")
                    .with_error_section(range, "trying to include this")
                    .note(
                        "local absolute path not allowed to access remote or local relative files",
                    )
                    .emit();
                Err(Error::Diagnostic)
            },

            (_, SourceGroup::LocalAbsolute(path)) => {
                if self.permissions.is_allowed_absolute(path) {
                    Ok(())
                } else {
                    diagnostics
                        .error("permission denied")
                        .with_error_section(range, "trying to include this")
                        .note(format!("not allowed to access absolute path {:?}", path))
                        .emit();
                    Err(Error::Diagnostic)
                }
            },

            // TODO: think about proper remote rules
            (Context::Remote(_), SourceGroup::Remote) => Ok(()),
            (Context::Remote(_), _) => {
                diagnostics
                    .error("permission denied")
                    .with_error_section(range, "trying to include this")
                    .note("remote file can only include other remote content")
                    .emit();
                Err(Error::Diagnostic)
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Context {
    LocalRelative(LocalRelative),
    LocalAbsolute(PathBuf),
    Remote(Url),
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalRelative {
    work_dir: PathBuf,
    relative: PathBuf,
}

impl Context {
    pub fn relative_root(work_dir: PathBuf) -> Self {
        Context::LocalRelative(LocalRelative::new(work_dir, Path::new(".").to_path_buf()))
    }

    /// The path to the working directory of the resource.
    ///
    /// Relative lookups are first resolved to absolute ones within the working directory and the
    /// working dir is then used to resolve that locally absolute one.
    fn work_dir(&self) -> Option<&Path> {
        match self {
            Context::LocalRelative(local) => Some(local.work_dir()),
            Context::LocalAbsolute(_) | Context::Remote(_) => None,
        }
    }

    /// Get the heradoc url to which to join references.
    fn as_heradoc_url(&self) -> Url {
        match self {
            Context::LocalRelative(local) =>{
                let mut base = Url::parse("heradoc://document/").unwrap();
                base.set_path(local.relative.to_str().unwrap());
                base
            },
            Context::LocalAbsolute(document) => {
                Url::from_file_path(document).unwrap()
            },
            Context::Remote(url) => url.clone(),
        }
    }
}

impl LocalRelative {
    pub fn new(work_dir: PathBuf, relative: PathBuf) -> Self {
        assert!(relative.is_relative());
        LocalRelative {
            work_dir,
            relative,
        }
    }

    fn work_dir(&self) -> &Path {
        self.work_dir.as_path()
    }
}

impl Permissions {
    fn is_allowed_absolute(&self, path: impl AsRef<Path>) -> bool {
        self.allowed_absolute_folders
            .iter()
            .any(|allowed| allowed == path.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{DirBuilder, File};
    use std::sync::{Arc, Mutex};
    use tempdir::TempDir;
    use codespan_reporting::termcolor::{ColorChoice, StandardStream};
    use crate::diagnostics::Input;

    macro_rules! assert_match {
    ($left:expr, $right:pat if $cond:expr) => ({
        let left_val = $left;
        match &left_val {
            $right if $cond => (),
            _ => {
                panic!(r#"assertion failed: `match left`
  left: `{:?}`,
 right: `{:?}`"#, left_val, stringify!($right))
            }
        }
    });
    ($left:expr, $right:pat) => ({
        assert_match!($left, $right if true)
    });
}

    fn prepare() -> (TempDir, SourceRange, Diagnostics<'static>) {
        let dir = TempDir::new("heradoc-test").expect("Can't create tempdir");
        let _ = File::create(dir.path().join("main.md")).expect("Can't create main.md");
        let _ = File::create(dir.path().join("test.md")).expect("Can't create test.md");
        let _ = File::create(dir.path().join("image.png")).expect("Can't create image.png");
        let _ = File::create(dir.path().join("pdf.pdf")).expect("Can't create pdf.pdf");
        DirBuilder::new().create(dir.path().join("chapter")).expect("Can't create chapter subdir");
        DirBuilder::new().create(dir.path().join("images")).expect("Can't create images subdir");
        let _ = File::create(dir.path().join("chapter/main.md")).expect("Can't create chapter main.md");
        let _ = File::create(dir.path().join("chapter/sibling.md")).expect("Can't create chapter sibling.md");
        let _ = File::create(dir.path().join("images/image.png")).expect("Can't create subdir image.png");
        let range = SourceRange { start: 0, end: 0 };
        let diagnostics = Diagnostics::new("", Input::Stdin, Arc::new(Mutex::new(StandardStream::stderr(ColorChoice::Auto))));
        (dir, range, diagnostics)
    }

    #[test]
    fn standard_resolves() {
        let (dir, range, diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let top = Context::relative_root(dir.path().to_path_buf());

        let main = resolver
            .resolve(&top, "main.md", range, &diagnostics)
            .expect("Failed to resolve direct path");
        let sibling = resolver
            .resolve(&top, "image.png", range, &diagnostics)
            .expect("Failed to resolve sibling file");

        assert_match!(main, Include::Markdown(path, _) if path == &dir.path().join("main.md"));
        assert_match!(sibling, Include::Image(path) if path == &dir.path().join("image.png"));
    }

    #[test]
    fn domain_resolves() {
        let (dir, range, diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let top = Context::relative_root(dir.path().to_path_buf());

        let toc = resolver
            .resolve(&top, "//toc", range, &diagnostics)
            .expect("Failed to resolve path in different domain");

        assert_eq!(toc, Include::Command(Command::Toc));
    }

    #[test]
    fn http_resolves_needs_internet() {
        let (dir, range, diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let top = Context::relative_root(dir.path().to_path_buf());

        let external = resolver
            .resolve(
                &top,
                "https://raw.githubusercontent.com/oberien/heradoc/master/README.md",
                range,
                &diagnostics,
            )
            .expect("Failed to download external document");

        assert_match!(external, Include::Markdown(_, Context::Remote(_)));
    }

    #[test]
    fn local_resolves_not_exist_not_internal_bug() {
        let (dir, range, mut diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let top = Context::relative_root(dir.path().to_path_buf());

        let error = resolver
            .resolve(&top, "this_file_does_not_exist.md", range, &mut diagnostics)
            .expect_err("Only files that exist on disk can be resolved");

        assert_match!(error, Error::Diagnostic);
    }

    #[test]
    fn local_absolute_url_to_relative() {
        let (dir, range, mut diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let top = Context::relative_root(dir.path().to_path_buf());

        let url = Url::from_file_path(dir.path().join("main.md")).unwrap();
        let main = resolver
            .resolve(&top, url.as_str(), range, &mut diagnostics)
            .expect("Failed to resolve absolute file url");

        assert_match!(main, Include::Markdown(_, Context::LocalRelative(local)) if local.work_dir() == dir.path());
    }

    #[test]
    fn local_url_does_not_exist() {
        let (dir, range, mut diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let top = Context::relative_root(dir.path().to_path_buf());

        let url = Url::from_file_path(dir.path().join("this_file_does_not_exist.md")).unwrap();
        let error = resolver
            .resolve(&top, url.as_str(), range, &mut diagnostics)
            .expect_err("Failed to resolve absolute file url");

        assert_match!(error, Error::Diagnostic);
    }

    #[test]
    fn relative_in_subdirectory() {
        let (dir, range, diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let main = Context::LocalRelative(
            LocalRelative::new(dir.path().to_path_buf(), Path::new("chapter/main.md").to_path_buf()));

        let sibling = resolver
            .resolve(&main, "sibling.md", range, &diagnostics)
            .expect("Failed to resolve sibling file");

        let alternative = resolver
            .resolve(&main, "./sibling.md", range, &diagnostics)
            .expect("Failed to resolve sibling file via explicitely relative path");
        assert_eq!(sibling, alternative);

        assert_match!(sibling, Include::Markdown(path, Context::LocalRelative(local))
                        if path == &dir.path().join("chapter/sibling.md") && local.work_dir() == dir.path());
    }

    #[test]
    fn local_relative_to_higher_directory() {
        let (dir, range, diagnostics) = prepare();
        let resolver = Resolver::new(PathBuf::from("."), dir.path().join("download"));
        let main = Context::relative_root(dir.path().to_path_buf());

        let up_and_over = resolver
            .resolve(&main, "../images/image.png", range, &diagnostics)
            .expect("Failed to resolve sibling file");

        assert_match!(up_and_over, Include::Image(path) if path == &dir.path().join("images/image.png"));
    }
}
