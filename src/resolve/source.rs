use std::env;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use url::Url;

use crate::diagnostics::Diagnostics;
use crate::error::{Error, Fatal, Result};
use crate::frontend::range::SourceRange;
use crate::resolve::remote::{ContentType, Error as RemoteError, Remote};
use crate::resolve::{Command, Context, Include};

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
    LocalRelative(PathBuf),
    /// Absolute file, not within context directory.
    ///
    /// Ex: `![](file:///foo.md)`
    LocalAbsolute(PathBuf),
    /// Remote source / file.
    ///
    /// Ex: `![](https://foo.bar/baz.md)`
    Remote,
}

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
    NoCanonical(io::Error),
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
            (diagnostics.bug("internal error converting url to path"), Error::Fatal(Fatal::InternalError))
        },
        PathError::InvalidBase | PathError::InvalidComponent | PathError::NoCanonical(_) => {
            (diagnostics.error("error converting url to path"), Error::Diagnostic)
        }
    };

    let message = match err {
        PathError::NoPath => "since the file url does not contain a path".into(),
        PathError::InvalidBase => "since the file url contains an unexpected base".into(),
        PathError::InvalidComponent => "since the url segment is not a valid path component".into(),
        PathError::MissingComponent => "since an url segment did not correspond to any path component".into(),
        PathError::NoCanonical(io) => format!("couldn't determine canonical filepath: {}", io),
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
                    let workdir = context
                        .path()
                        .ok_or_else(|| error_include_local_from_remote(diagnostics, range))?;
                    let path = to_path(&url, workdir)
                        .map_err(|err| error_to_path(diagnostics, range, err))?;
                    SourceGroup::LocalRelative(path)
                },
                _ => SourceGroup::Implementation,
            },
            "file" => {
                let workdir = context
                    .path()
                    .ok_or_else(|| error_include_local_from_remote(diagnostics, range))?;
                let path = url
                    .to_file_path()
                    .map_err(|()| PathError::InvalidBase)
                    .map_err(|err| error_to_path(diagnostics, range, err))?;
                let path = workdir.join(path);
                let is_relative = match context {
                    Context::LocalRelative(workdir) => path.starts_with(workdir),
                    _ => false,
                };
                if is_relative {
                    SourceGroup::LocalRelative(path)
                } else {
                    SourceGroup::LocalAbsolute(path)
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
            SourceGroup::LocalRelative(path) => {
                let parent = path.parent().unwrap().to_owned();
                to_include(path, Context::LocalRelative(parent), range, diagnostics)
            },
            SourceGroup::LocalAbsolute(path) => {
                let parent = path.parent().unwrap().to_owned();
                to_include(path, Context::LocalAbsolute(parent), range, diagnostics)
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

fn to_path<P: AsRef<Path>>(url: &Url, workdir: P) -> std::result::Result<PathBuf, PathError> {
    let mut full_path = workdir.as_ref().to_path_buf();

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

    full_path
        .canonicalize()
        .map_err(PathError::NoCanonical)
}
