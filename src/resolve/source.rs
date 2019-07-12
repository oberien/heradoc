use std::io;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use url::Url;

use crate::diagnostics::Diagnostics;
use crate::error::{Error, Fatal, Result};
use crate::frontend::range::SourceRange;
use crate::resolve::remote::{ContentType, Error as RemoteError, Remote};
use crate::resolve::{Command, Context, Include, LocalRelative as RelativeContext};

/// Differentiate between sources based on their access right characteristics.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Source {
    pub url: Url,
    pub group: SourceGroup,
}

/// Types URLs are handled as / put into
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SourceGroup {
    /// Implemented commands / codegen.
    ///
    /// Ex: `![](//TOC)`
    Implementation,
    /// Local file relative to (without backrefs) or inside of context directory.
    ///
    /// Ex: `![](/foo.md)`, `![](foo.md)`, `![](file:///absolute/path/to/workdir/foo.md)`
    LocalRelative(LocalRelative),
    /// Absolute file, not within context directory.
    ///
    /// Ex: `![](file:///foo.md)`
    LocalAbsolute(Canonical),
    /// Remote source / file.
    ///
    /// Ex: `![](https://foo.bar/baz.md)`
    Remote,
}

/// A local relative include resolution.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LocalRelative {
    work_dir: PathBuf,
    path: Canonical,
}

/// A canonicalized path.
///
/// This helps ensure that local relative paths really are relative to the working directory. It's
/// a simple wrapper to avoid accidentally missing the canonicalization step.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Canonical(PathBuf);

#[derive(Debug)]
enum PathError {
    /// The url does not contain a path.
    NoPath,

    /// `file:` url included a hostname that was not a local prefix on Windows, or empty on Linux.
    ///
    /// See `Url::to_file_path` for details.
    InvalidBase,

    /// A path component mapped to more than one path component or was otherwise invalid.
    InvalidComponent,

    /// Some path component did not map to any path component.
    ///
    /// I don't expect this to ever occur, this error is Fatal.
    MissingComponent,

    /// Could not determine canonical path.
    ///
    /// Canonicalization is important to ensure that the file does not refer to internal files,
    /// potentially circumventing access restrictions.
    NoCanonical(PathBuf, io::Error),
}

fn error_include_local_from_remote(
    diagnostics: &Diagnostics<'_>, range: SourceRange,
) -> Error {
    diagnostics
        .error("tried to include local file from remote origin")
        .with_error_section(range, "specified here")
        .note("local files can only be included from within local files")
        .emit();
    Error::Diagnostic
}

fn error_to_path(diagnostics: &Diagnostics<'_>, range: SourceRange, err: PathError) -> Error {
    let (diagnostics, error) = match &err {
        PathError::NoPath | PathError::MissingComponent => {
            (diagnostics.bug("internal error converting url to path"), Error::Fatal(Fatal::InteralCompilerError))
        },
        PathError::InvalidBase | PathError::InvalidComponent | PathError::NoCanonical(..) => {
            (diagnostics.error("error converting url to path"), Error::Diagnostic)
        }
    };

    let message = match err {
        PathError::NoPath => "since this file url does not contain a path".into(),
        PathError::InvalidBase => "since the file url contains an unexpected base".into(),
        PathError::InvalidComponent => "since the url segment is not a valid path component".into(),
        PathError::MissingComponent => "since an url segment did not correspond to any path component".into(),
        PathError::NoCanonical(path, io) => format!("couldn't determine canonical filepath of {:?}: {}", path, io),
    };

    diagnostics
        .with_info_section(range, "defined here")
        .error(message)
        .emit();

    error
}

impl Source {
    pub fn new(
        url: Url, context: &Context, range: SourceRange, diagnostics: &Diagnostics<'_>,
    ) -> Result<Self> {
        let group = match url.scheme() {
            "heradoc" => match url.domain() {
                Some("document") => {
                    let work_dir = context
                        .work_dir()
                        .ok_or_else(|| error_include_local_from_remote(diagnostics, range))?;
                    to_local_relative(&url, work_dir)
                        .map_err(|err| error_to_path(diagnostics, range, err))?
                },
                _ => SourceGroup::Implementation,
            },
            "file" => {
                if url.cannot_be_a_base() { // Relative file path.
                    let work_dir = context
                        .work_dir()
                        .ok_or_else(|| error_include_local_from_remote(diagnostics, range))?;
                    to_local_relative(&url, work_dir)
                        .map_err(|err| error_to_path(diagnostics, range, err))?
                } else { // Absolute file path.
                    let path = url
                        .to_file_path()
                        .map_err(|()| PathError::InvalidBase)
                        .and_then(Canonical::try_from_path)
                        .map_err(|err| error_to_path(diagnostics, range, err))?;
                    // Absolute path is considered effectively relative if it points to the working
                    // directory and occurs in a local context. All other contexts consider all
                    // paths with a non-relative base absolute.
                    if let Context::LocalRelative(local) = context {
                        let work_dir = local.work_dir();
                        if path.as_ref().strip_prefix(work_dir).is_ok() {
                            SourceGroup::LocalRelative(LocalRelative {
                                work_dir: work_dir.to_owned(),
                                path,
                            })
                        } else {
                            SourceGroup::LocalAbsolute(path)
                        }
                    } else {
                        SourceGroup::LocalAbsolute(path)
                    }
                }
            },
            _ => SourceGroup::Remote,
        };

        Ok(Source { url, group })
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

fn to_local_relative<P: AsRef<Path>>(url: &Url, work_dir: P) -> std::result::Result<SourceGroup, PathError> {
    let mut full_path = work_dir.as_ref().to_path_buf();

    url
        .path_segments()
        .ok_or(PathError::NoPath)?
        .try_for_each(|segment| {
            let mut components = Path::new(segment)
                .components();
            let file = match components.next() {
                Some(Component::Normal(file)) => file,
                Some(_) => return Err(PathError::InvalidComponent),
                _ => return Err(PathError::MissingComponent),
            };
            if components.next().is_some() {
                return Err(PathError::InvalidComponent)
            }
            full_path.push(file);
            Ok(())
        })?;

    Ok(SourceGroup::LocalRelative(LocalRelative {
        work_dir: work_dir.as_ref().to_path_buf(),
        path: Canonical::try_from_path(full_path)?,
    }))
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
