use std::io;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use std::env;

use url::Url;

use crate::diagnostics::Diagnostics;
use crate::error::{Error, Fatal, Result};
use crate::frontend::range::SourceRange;
use crate::resolve::remote::{ContentType, Error as RemoteError, Remote};
use crate::resolve::{Command, Context, Include, Permissions, ContextType};
use crate::resolve::target::TargetInner::LocalAbsolute;

/// Target pointed to by URL before the permission check.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Target<'a, 'b> {
    inner: TargetInner,
    meta: Meta<'a, 'b>,
}

/// Target after canonicalization
pub struct TargetCanonicalized<'a, 'b> {
    inner: TargetInner,
    meta: Meta<'a, 'b>,
}

/// Target after its permissions have been checked
pub struct TargetChecked<'a, 'b> {
    inner: TargetInner,
    meta: Meta<'a, 'b>,
}

struct Meta<'a, 'd> {
    url: Url,
    context: &'a Context,
    project_root: &'a Path,
    permissions: &'a Permissions,
    range: SourceRange,
    diagnostics: &'a Diagnostics<'d>,
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

impl<'a, 'b> Target<'a, 'b> {
    /// Create a new Target for the given URL, resolved in the given context.
    ///
    /// This target can be canonicalized and access-checked within the context before being converted
    /// to the respective Include.
    /// Local relative files are resolved relative to the project_root.
    pub fn new(
        url: &str, context: &Context, project_root: &Path, permissions: &Permissions,
        range: SourceRange, diagnostics: &Diagnostics<'_>,
    ) -> Result<Self> {
        let url = match context.url.join(url) {
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
        let inner = match url.scheme() {
            "heradoc" => match url.domain() {
                Some("document") => TargetInner::LocalRelative(url.path_segments().unwrap().collect()),
                Some(domain) => TargetInner::Implementation(domain.to_string()),
                None => {
                    diagnostics
                        .error("no heradoc implementation domain found")
                        .with_error_section(range, "defined here")
                        .emit();
                    return Err(Error::Diagnostic);
                }
            },
            "file" => {
                match url.to_file_path() {
                    Ok(path) => TargetInner::LocalAbsolute(path),
                    Err(()) => {
                        diagnostics.error("error converting url to path")
                            .with_info_section(range, "defined here")
                            .error("the file url can't be converted to a path")
                            .help("this could be due to a malformed URL like a non-empty or non-localhost domain")
                            .emit();
                        return Err(Error::Diagnostic);
                    }
                }
            },
            _ => TargetInner::Remote(url),
        };
        Ok(Target {
            inner,
            meta: Meta {
                url,
                context,
                project_root,
                permissions,
                range,
                diagnostics,
            }
        })
    }

    pub fn canonicalize(self) -> Result<TargetCanonicalized<'a, 'b>> {
        let Target { inner, meta } = self;

        let canonicalize = |path| {
            match path.canonicalize() {
                Ok(path) => Ok(path),
                Err(e) => {
                    meta.diagnostics
                        .error("error canonicalizing path")
                        .with_error_section(meta.range, "trying to include this")
                        .note(format!("canonicalizing the path: {:?}", path))
                        .error(e.to_string())
                        .emit();
                    Err(Error::Diagnostic)
                }
            }
        };

        let inner = match inner {
            inner @ TargetInner::Implementation(command) => inner,
            inner @ TargetInner::Remote() => inner,
            TargetInner::LocalAbsolute(abs) => TargetInner::LocalAbsolute(canonicalize(abs)?),
            TargetInner::LocalRelative(rel) => {
                assert!(rel.is_relative(), "TargetInner::LocalRelative not relative before canonicalizing: {:?}", rel);
                let relative_to_project_root = meta.project_root.join(rel);
                let canonicalized = canonicalize(relative_to_project_root)?;
                TargetInner::LocalRelative(canonicalized)
            },
        };
        Ok(TargetCanonicalized {
            inner,
            meta,
        })
    }
}

impl <'a, 'b> TargetCanonicalized<'a, 'b> {
    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user. Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    pub fn check_access(self) -> Result<TargetChecked<'a, 'b>> {
        let TargetCanonicalized { inner, meta } = self;
        match (meta.context.typ(), &inner) {
            (ContextType::LocalRelative, TargetInner::Implementation(_))
            | (ContextType::LocalRelative, TargetInner::LocalRelative(_))
            | (ContextType::LocalRelative, TargetInner::Remote(_)) => (),

            (ContextType::LocalAbsolute, TargetInner::Implementation(_)) => (),
            (ContextType::LocalAbsolute, TargetInner::LocalRelative(_))
            | (ContextType::LocalAbsolute, TargetInner::Remote(_)) => {
                meta.diagnostics
                    .error("permission denied")
                    .with_error_section(meta.range, "trying to include this")
                    .note(
                        "local absolute path not allowed to access remote or local relative files",
                    )
                    .emit();
                return Err(Error::Diagnostic)
            },

            // TODO: discuss proper remote rules
            // check CORS
            (ContextType::Remote, TargetInner::Remote(url)) if meta.context.url.domain() == url.domain() => (),
            (ContextType::Remote, TargetInner::Remote(_)) => {
                meta.diagnostics
                    .error("permission denied")
                    .with_error_section(meta.range, "trying to include this")
                    .error("CORS request detected")
                    .note("remote inclusions can only include remote content from the same domain")
                    .emit();
                return Err(Error::Diagnostic)
            },
            (ContextType::Remote, _) => {
                meta.diagnostics
                    .error("permission denied")
                    .with_error_section(meta.range, "trying to include this")
                    .note("remote file can only include other remote content")
                    .emit();
                return Err(Error::Diagnostic)
            },

            (_, TargetInner::LocalAbsolute(path)) => {
                if !meta.permissions.is_allowed_absolute(path) {
                    meta.diagnostics
                        .error("permission denied")
                        .with_error_section(meta.range, "trying to include this")
                        .note(format!("not allowed to access absolute path {:?}", path))
                        .emit();
                    return Err(Error::Diagnostic)
                }
            },
        }
        Ok(TargetChecked {
            inner,
            meta,
        })
    }
}

impl<'a, 'b> TargetChecked<'a, 'b> {
    pub fn into_include(self, remote: &Remote) -> Result<Include> {
        let TargetChecked { inner, meta } = self;
        match inner {
            TargetInner::Implementation(command) => {
                match Command::from_str(&command) {
                    Ok(command) => Ok(Include::Command(command)),
                    Err(()) => {
                        meta.diagnostics
                            .error(format!(
                                "no heradoc implementation found for domain {:?}",
                                command
                            )).with_error_section(meta.range, "defined here")
                            .emit();
                        return Err(Error::Diagnostic)
                    }
                }
            },
            TargetInner::LocalRelative(path) => {
                // Making doubly sure for future changes.
                match path.strip_prefix(meta.project_root) {
                    Ok(_) => (),
                    Err(e) => {
                        meta.diagnostics
                            .bug("Local relative path resolved to non-relative path")
                            .error(format!("cause: {}", e))
                            .emit();
                        return Err(Error::Diagnostic);
                    }
                }
                let context = Context::from_url(meta.url);
                to_include(path, context, meta.range, meta.diagnostics)
            },
            TargetInner::LocalAbsolute(path) => {
                let context = Context::from_url(meta.url);
                to_include(path, context, meta.range, meta.diagnostics)
            },
            TargetInner::Remote(url) => {
                let downloaded = match remote.http(&url) {
                    Ok(downloaded) => downloaded,
                    Err(RemoteError::Io(err, path)) => {
                        meta.diagnostics
                            .error("error writing downloaded content to cache")
                            .with_error_section(meta.range, "trying to download this")
                            .error(format!("cause: {}", err))
                            .note(format!("file: {}", path.display()))
                            .emit();
                        return Err(Error::Diagnostic);
                    },
                    Err(RemoteError::Request(err)) => {
                        meta.diagnostics
                            .error("error downloading content")
                            .with_error_section(meta.range, "trying to download this")
                            .error(format!("cause: {}", err))
                            .emit();
                        return Err(Error::Diagnostic);
                    },
                };

                let path = downloaded.path().to_owned();
                let context = Context::from_url(meta.url);

                match downloaded.content_type() {
                    Some(ContentType::Image) => Ok(Include::Image(path)),
                    Some(ContentType::Markdown) => Ok(Include::Markdown(path, context)),
                    Some(ContentType::Pdf) => Ok(Include::Pdf(path)),
                    None => to_include(path, context, meta.range, meta.diagnostics),
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
