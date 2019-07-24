use std::io;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use std::env;

use url::Url;

use crate::diagnostics::Diagnostics;
use crate::error::{Error, Fatal, Result};
use crate::frontend::range::SourceRange;
use crate::resolve::remote::{ContentType, Error as RemoteError, Remote};
use crate::resolve::{Command, Context, Include, LocalRelative as RelativeContext, Permissions};
use crate::resolve::source::TargetInner::LocalAbsolute;

/// Target pointed to by URL before the permission check.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Target {
    inner: TargetInner,
}

/// Target after canonicalization
pub struct TargetCanonicalized {
    inner: TargetInner,
}

/// Target after its permissions have been checked
pub struct TargetChecked {
    inner: TargetInner,
}

enum TargetInner {
    /// Implemented commands / codegen.
    ///
    /// Ex: `![](//TOC)`
    Implementation(String),
    /// Local file inside the workdir or the context directory.
    ///
    /// The `PathBuf` must be relative.
    ///
    /// Ex: `![](/foo.md)`, `![](foo.md)`
    LocalRelative(PathBuf),
    /// Any file with an absolute path.
    ///
    /// Ex: `![](file:///foo.md)`
    LocalAbsolute(PathBuf),
    /// Remote source / file.
    ///
    /// Ex: `![](https://foo.bar/baz.md)`
    Remote(Url),
}

impl Target {
    pub fn new(
        url: Url, range: SourceRange, diagnostics: &Diagnostics<'_>,
    ) -> Result<Self> {
        let inner = match url.scheme() {
            "heradoc" => match url.domain() {
                Some("document") => TargetInner::LocalRelative(url.path_segments().unwrap().collect()),
                _ => TargetInner::Implementation(url.domain().to_string()),
            },
            "file" => {
                match url.to_file_path() {
                    Ok(path) => TargetInner::LocalAbsolute(path),
                    Err(()) => {
                        diagnostics.error("error converting url to path")
                            .with_info_section(range, "defined here")
                            .error("the file url can't be converted to a path")
                            .emit();
                        return Err(Error::Diagnostic);
                    }
                }
            },
            _ => TargetInner::Remote(url),
        };
        Ok(Target { inner })
    }

    pub fn canonicalize(
        self, context: &Context, range: SourceRange, diagnostics: &Diagnostics<'_>,
    ) -> Result<TargetCanonicalized> {
        let inner = match self.inner {
            inner @ TargetInner::Implementation(command) => inner,
            inner @ TargetInner::Remote() => inner,
            TargetInner::LocalAbsolute(abs) => {
                match abs.canonicalize() {
                    Ok(path) => TargetInner::LocalAbsolute(path),
                    Err(e) => {
                        diagnostics
                            .error("error canonicalizing absolute path")
                            .with_error_section(range, "trying to include this")
                            .note(format!("canonicalizing the path: {:?}", abs))
                            .error(e.to_string())
                            .emit();
                        return Err(Error::Diagnostic);
                    }
                }
            },
            TargetInner::LocalRelative(rel) => {
                assert!(rel.is_relative);
                let relative_to_context_dir = env::current_dir().unwrap().join()
            }
        };
        Ok(TargetCanonicalized { inner })
    }
}

impl TargetChecked {
    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user. Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    pub fn check_access(
        self, context: &Context, permissions: &Permissions, range: SourceRange, diagnostics: &Diagnostics<'_>
    ) -> Result<TargetChecked> {
        match (context, &self.inner) {
            (Context::LocalRelative(_), TargetInner::Implementation(_))
            | (Context::LocalRelative(_), TargetInner::LocalRelative(_))
            | (Context::LocalRelative(_), TargetInner::Remote(_)) => (),

            (Context::LocalAbsolute(_), TargetInner::Implementation(_)) => (),
            (Context::LocalAbsolute(_), TargetInner::LocalRelative(_))
            | (Context::LocalAbsolute(_), TargetInner::Remote(_)) => {
                diagnostics
                    .error("permission denied")
                    .with_error_section(range, "trying to include this")
                    .note(
                        "local absolute path not allowed to access remote or local relative files",
                    )
                    .emit();
                return Err(Error::Diagnostic)
            },

            // TODO: discuss proper remote rules
            (Context::Remote(_), TargetInner::Remote(_)) => (),
            (Context::Remote(_), _) => {
                diagnostics
                    .error("permission denied")
                    .with_error_section(range, "trying to include this")
                    .note("remote file can only include other remote content")
                    .emit();
                return Err(Error::Diagnostic)
            },

            (_, TargetInner::LocalAbsolute(path)) => {
                if !permissions.is_allowed_absolute(path) {
                    diagnostics
                        .error("permission denied")
                        .with_error_section(range, "trying to include this")
                        .note(format!("not allowed to access absolute path {:?}", path))
                        .emit();
                    return Err(Error::Diagnostic)
                }
            },
        }
        Ok(TargetChecked { inner: self.inner })
    }
}
    pub fn into_include(
        self, remote: &Remote, range: SourceRange, diagnostics: &Diagnostics<'_>,
    ) -> Result<Include> {
        let Source { url, group } = self;
        match group {
            SourceGroup::Implementation => {
                if let Some(domain) = url.domain() {
                    if let Ok(command) = Command::from_str(domain) {
                        Ok(Include::Command(command))
                    } else {
                        diagnostics
                            .error(format!(
                                "no heradoc implementation found for domain {:?}",
                                domain
                            ))
                            .with_error_section(range, "defined here")
                            .emit();
                        Err(Error::Diagnostic)
                    }
                } else {
                    diagnostics
                        .error("no heradoc implementation domain found")
                        .with_error_section(range, "defined here")
                        .emit();
                    Err(Error::Diagnostic)
                }
            },
            SourceGroup::LocalRelative(LocalRelative { work_dir, path: canonical }) => {
                let path = canonical.into_inner();
                // Making doubly sure for future changes.
                let relative = path.strip_prefix(&work_dir)
                    .map_err(|err| {
                        diagnostics
                            .bug("Local relative path resolved to non-relative path")
                            .error(format!("cause: {}", err))
                            .emit();
                        Error::Diagnostic
                    })?
                    .to_path_buf();
                let context = Context::LocalRelative(RelativeContext::new(work_dir, relative));
                to_include(path, context, range, diagnostics)
            },
            SourceGroup::LocalAbsolute(canonical) => {
                let path = canonical.into_inner();
                let context = Context::LocalAbsolute(path.clone());
                to_include(path, context, range, diagnostics)
            },
            SourceGroup::Remote => {
                let downloaded = match remote.http(&url) {
                    Ok(downloaded) => downloaded,
                    Err(RemoteError::Io(err, path)) => {
                        diagnostics
                            .error("error writing downloaded content to cache")
                            .with_error_section(range, "trying to download this")
                            .error(format!("cause: {}", err))
                            .note(format!("file: {}", path.display()))
                            .emit();
                        return Err(Error::Diagnostic);
                    },
                    Err(RemoteError::Request(err)) => {
                        diagnostics
                            .error("error downloading content")
                            .with_error_section(range, "trying to download this")
                            .error(format!("cause: {}", err))
                            .emit();
                        return Err(Error::Diagnostic);
                    },
                };

                let path = downloaded.path().to_owned();
                let context = Context::Remote(url);

                match downloaded.content_type() {
                    Some(ContentType::Image) => Ok(Include::Image(path)),
                    Some(ContentType::Markdown) => Ok(Include::Markdown(path, context)),
                    Some(ContentType::Pdf) => Ok(Include::Pdf(path)),
                    None => to_include(path, context, range, diagnostics),
                }
            },
        }
    }

impl Canonical {
    fn try_from_path(path: impl AsRef<Path>) -> std::result::Result<Self, PathError> {
        match path.as_ref().canonicalize() {
            Ok(path) => Ok(Canonical(path)),
            Err(io) => Err(PathError::NoCanonical(path.as_ref().to_path_buf(), io)),
        }
    }

    pub fn into_inner(self) -> PathBuf {
        self.into()
    }
}

/// Guess the type of include based on the file extension.
///
/// Used to detect the type of include for relative and absolute file paths or for webrequest
/// includes that did not receive repsonse with a media type header. Matching is performed purely
/// based on the file extension.
fn to_include(
    path: PathBuf, context: Context, range: SourceRange, diagnostics: &Diagnostics<'_>,
) -> Result<Include> {
    // TODO: switch on file header type first
    match path.extension().map(|s| s.to_str().unwrap()) {
        Some("md") => Ok(Include::Markdown(path, context)),
        Some("png") | Some("jpg") | Some("jpeg") => Ok(Include::Image(path)),
        Some("pdf") => Ok(Include::Pdf(path)),
        Some(ext) => {
            diagnostics
                .error(format!("unknown file format {:?}", ext))
                .with_error_section(range, "trying to include this")
                .emit();
            Err(Error::Diagnostic)
        },
        None => {
            diagnostics
                .error("no file extension")
                .with_error_section(range, "trying to include this")
                .note("need file extension to differentiate file type")
                .emit();
            Err(Error::Diagnostic)
        },
    }
}

impl AsRef<Path> for Canonical {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl From<Canonical> for PathBuf {
    fn from(canonical: Canonical) -> PathBuf {
        canonical.0
    }
}
