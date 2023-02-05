use std::path::{Path, PathBuf};
use std::str::FromStr;
use diagnostic::Span;

use url::Url;
use crate::Diagnostics;

use crate::error::{DiagnosticCode, Error, Result};
use crate::resolve::remote::{ContentType, Error as RemoteError, Remote};
use crate::resolve::{Command, Context, Include, Permissions, ContextType};

/// Target pointed to by URL before the permission check.
#[must_use]
#[derive(Debug)]
pub struct Target<'a> {
    inner: TargetInner,
    meta: Meta<'a>,
}

/// Target after canonicalization
#[must_use]
#[derive(Debug)]
pub struct TargetCanonicalized<'a> {
    inner: TargetInner,
    meta: Meta<'a>,
}

/// Target after its permissions have been checked
#[must_use]
#[derive(Debug)]
pub struct TargetChecked<'a> {
    inner: TargetInner,
    meta: Meta<'a>,
}

#[derive(Debug)]
struct Meta<'a> {
    url: Url,
    context: &'a Context,
    project_root: &'a Path,
    document_root: &'a Path,
    permissions: &'a Permissions,
    span: Span,
    diagnostics: &'a Diagnostics,
}

#[derive(Debug)]
enum TargetInner {
    /// Implemented commands / codegen.
    ///
    /// Ex: `![](//TOC)`
    Implementation(String),
    /// Local file relative to the document root.
    ///
    /// The `PathBuf` must be relative.
    ///
    /// Ex: `![](//document/foo.md)`
    LocalDocumentRelative(PathBuf),
    /// Local file relative to the project root.
    ///
    /// The `PathBuf` must be relative.
    /// When the path to resolve is file-relative, we can get its project-relative path
    /// by joining it on the context's url (which points to the current file).
    /// Therefore, file-relative is also project-relative at the same time.
    ///
    /// Ex: `![](foo.md)`, `![](/foo.md)`, `![](/images/bar.png)`
    LocalProjectRelative(PathBuf),
    /// Any file with an absolute path.
    ///
    /// Ex: `![](file:///foo.md)`
    LocalAbsolute(PathBuf),
    /// Remote source / file.
    ///
    /// Ex: `![](https://foo.bar/baz.md)`
    Remote(Url),
}

impl<'a> Target<'a> {
    /// Create a new Target for the given URL, resolved in the given context.
    ///
    /// This target can be canonicalized and access-checked within the context before being converted
    /// to the respective Include.
    /// Local relative files are resolved relative to the project_root.
    pub fn new(
        to_resolve: &str, context: &'a Context, project_root: &'a Path, document_root: &'a Path,
        permissions: &'a Permissions, span: Span, diagnostics: &'a Diagnostics,
    ) -> Result<Target<'a>> {
        let url = match context.url.join(to_resolve) {
            Ok(url) => url,
            Err(err) => {
                diagnostics
                    .error(DiagnosticCode::ErrorResolvingFile)
                    .with_error_label(span, "couldn't resolve this file")
                    .with_note(format!("tried to resolve {}", to_resolve))
                    .with_note(format!("malformed reference: {}", err))
                    .emit();
                return Err(Error::Diagnostic);
            },
        };
        let inner = match url.scheme() {
            "heradoc" => match url.domain() {
                Some("project") => TargetInner::LocalProjectRelative(url.path_segments().unwrap().collect()),
                Some("document") => TargetInner::LocalDocumentRelative(url.path_segments().unwrap().collect()),
                Some(domain) => TargetInner::Implementation(domain.to_string()),
                None => {
                    diagnostics
                        .error(DiagnosticCode::UnsupportedDomain)
                        .with_error_label(span, "no heradoc implementation domain provided")
                        .with_note("the domain must be either `document` for includes or an implementation command")
                        .emit();
                    return Err(Error::Diagnostic);
                }
            },
            "file" => match url.to_file_path() {
                Ok(path) => TargetInner::LocalAbsolute(path),
                Err(()) => {
                    diagnostics
                        .error(DiagnosticCode::InvalidUrl)
                        .with_info_label(span, "error converting this url to path")
                        .with_note("this could be due to a malformed URL like a non-empty or non-localhost domain")
                        .emit();
                    return Err(Error::Diagnostic);
                }
            },
            _ => TargetInner::Remote(url.clone()),
        };
        Ok(Target {
            inner,
            meta: Meta {
                url,
                context,
                project_root,
                document_root,
                permissions,
                span,
                diagnostics,
            }
        })
    }

    pub fn canonicalize(self) -> Result<TargetCanonicalized<'a>> {
        let Target { inner, meta } = self;

        let canonicalize = |path: PathBuf| {
            match path.canonicalize() {
                Ok(path) => Ok(path),
                Err(e) => {
                    meta.diagnostics
                        .error(DiagnosticCode::ErrorCanonicalizingPath)
                        .with_error_label(meta.span, "error canonicalizing path of this include")
                        .with_error_label(meta.span, e.to_string())
                        .with_note(format!("canonicalizing the path: {:?}", path))
                        .emit();
                    Err(Error::Diagnostic)
                }
            }
        };

        let inner = match inner {
            inner @ TargetInner::Implementation(_) => inner,
            inner @ TargetInner::Remote(_) => inner,
            TargetInner::LocalAbsolute(abs) => TargetInner::LocalAbsolute(canonicalize(abs)?),
            TargetInner::LocalDocumentRelative(rel) => {
                assert!(rel.is_relative(), "TargetInner::LocalDocumentRelative not relative before canonicalizing: {:?}", rel);
                TargetInner::LocalDocumentRelative(canonicalize(meta.document_root.join(rel))?)
            },
            TargetInner::LocalProjectRelative(rel) => {
                assert!(rel.is_relative(), "TargetInner::LocalProjectRelative not relative before canonicalizing: {:?}", rel);
                TargetInner::LocalProjectRelative(canonicalize(meta.project_root.join(rel))?)
            },
        };
        Ok(TargetCanonicalized {
            inner,
            meta,
        })
    }

    /// Skip path canonicalization. Use with care, as this skips some security checks.
    ///
    /// This function should only be used when the target is known to be fine,
    /// for example if it was created from within heradoc.
    pub fn skip_canonicalization(self) -> TargetCanonicalized<'a> {
        let Target { inner, meta } = self;
        TargetCanonicalized { inner, meta }
    }
}

impl <'a> TargetCanonicalized<'a> {
    /// Test if the source is allowed to request the target document.
    ///
    /// Some origins are not allowed to read all documents or only after explicit clearance by the
    /// invoking user. Even more restrictive, the target handler could terminate the request at a
    /// later time. For example when requesting a remote document make a CORS check.
    pub fn check_access(self) -> Result<TargetChecked<'a>> {
        let TargetCanonicalized { inner, meta } = self;
        match (meta.context.typ(), &inner) {
            (ContextType::LocalRelative, TargetInner::Implementation(_))
            | (ContextType::LocalRelative, TargetInner::LocalProjectRelative(_))
            | (ContextType::LocalRelative, TargetInner::LocalDocumentRelative(_))
            | (ContextType::LocalRelative, TargetInner::Remote(_)) => (),

            (ContextType::LocalAbsolute, TargetInner::Implementation(_)) => (),
            (ContextType::LocalAbsolute, TargetInner::LocalProjectRelative(_))
            | (ContextType::LocalAbsolute, TargetInner::LocalDocumentRelative(_))
            | (ContextType::LocalAbsolute, TargetInner::Remote(_)) => {
                meta.diagnostics
                    .error(DiagnosticCode::PermissionDenied)
                    .with_error_label(meta.span, "permission denied for this include")
                    .with_note(
                        "local absolute path not allowed to access remote or local relative files",
                    )
                    .emit();
                return Err(Error::Diagnostic)
            },

            // TODO: discuss proper remote rules
            // deny cross-origin
            // TODO: proper CORS implementation
            (ContextType::Remote, TargetInner::Remote(url)) if meta.context.url.domain() == url.domain() => (),
            (ContextType::Remote, TargetInner::Remote(_)) => {
                meta.diagnostics
                    .error(DiagnosticCode::PermissionDenied)
                    .with_error_label(meta.span, "permission denied: cross-origin request detected")
                    .with_note("remote inclusions can only include remote content from the same domain")
                    .emit();
                return Err(Error::Diagnostic)
            },
            (ContextType::Remote, _) => {
                meta.diagnostics
                    .error(DiagnosticCode::PermissionDenied)
                    .with_error_label(meta.span, "permission denied: remote file can only include other remote content")
                    .with_note("")
                    .emit();
                return Err(Error::Diagnostic)
            },

            (_, TargetInner::LocalAbsolute(path)) => {
                if !meta.permissions.is_allowed_absolute(path) {
                    meta.diagnostics
                        .error(DiagnosticCode::PermissionDenied)
                        .with_error_label(meta.span, format!("permission denied: not allowed to access absolute path {:?}", path))
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

    /// Skips the access checks. Use with care, as this could result in security problems.
    /// This function should only be used when the target is known to be fine,
    /// for example if it was created from within heradoc.
    pub fn skip_check_access(self) -> TargetChecked<'a> {
        let TargetCanonicalized { inner, meta } = self;
        TargetChecked { inner, meta }
    }
}

impl<'a> TargetChecked<'a> {
    pub fn into_include(self, remote: &Remote) -> Result<Include> {
        let TargetChecked { inner, meta } = self;
        let check_still_relative = |path: &Path, root: &Path, name: &str| -> Result<()> {
            match path.strip_prefix(root) {
                Ok(_) => Ok(()),
                Err(e) => {
                    meta.diagnostics
                        .bug(DiagnosticCode::InternalCompilerError)
                        .with_note(format!("Local {}-relative path resolved to non-{}-relative path", name, name))
                        .with_note(format!("cause: {}", e))
                        .emit();
                    Err(Error::Diagnostic)
                }
            }
        };
        match inner {
            TargetInner::Implementation(command) => {
                match Command::from_str(&command) {
                    Ok(command) => Ok(Include::Command(command)),
                    Err(()) => {
                        meta.diagnostics
                            .error(DiagnosticCode::InvalidCommand)
                            .with_error_label(meta.span, format!("{:?} isn't a valid implementation command", command))
                            .emit();
                        return Err(Error::Diagnostic)
                    }
                }
            },
            TargetInner::LocalDocumentRelative(path) => {
                // Making doubly sure for future changes.
                // Number of times this error was hit during changes: 0
                check_still_relative(&path, meta.document_root, "document-root")?;
                to_include(path, Context::from_url(meta.url), meta.span, meta.diagnostics)
            },
            TargetInner::LocalProjectRelative(path) => {
                // Making doubly sure for future changes.
                // Number of times this error was hit during changes: 1
                check_still_relative(&path, meta.project_root, "project-root")?;
                to_include(path, Context::from_url(meta.url), meta.span, meta.diagnostics)
            },
            TargetInner::LocalAbsolute(path) => {
                let context = Context::from_url(meta.url);
                to_include(path, context, meta.span, meta.diagnostics)
            },
            TargetInner::Remote(url) => {
                println!("{url}");
                let downloaded = match remote.http(&url) {
                    Ok(downloaded) => downloaded,
                    Err(RemoteError::Io(err, path)) => {
                        meta.diagnostics
                            .error(DiagnosticCode::ErrorWritingToCache)
                            .with_error_label(meta.span, "error writing downloaded content to cache downloading this")
                            .with_note(format!("cause: {}", err))
                            .with_note(format!("file: {}", path.display()))
                            .emit();
                        return Err(Error::Diagnostic);
                    },
                    Err(RemoteError::Request(err)) => {
                        meta.diagnostics
                            .error(DiagnosticCode::ErrorDownloadingContent)
                            .with_error_label(meta.span, "error trying to download this")
                            .with_note(format!("cause: {}", err))
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
                    None => to_include(path, context, meta.span, meta.diagnostics),
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
    path: PathBuf, context: Context, span: Span, diagnostics: &Diagnostics,
) -> Result<Include> {
    let ext = path.extension().map(|s| s.to_str().unwrap().to_lowercase());
    match ext.as_ref().map(String::as_str) {
        Some("md") => Ok(Include::Markdown(path, context)),
        Some("png") | Some("jpg") | Some("jpeg") => Ok(Include::Image(path)),
        Some("svg") => Ok(Include::Svg(path)),
        Some("pdf") => Ok(Include::Pdf(path)),
        Some("gv") | Some("dot") => Ok(Include::Graphviz(path)),
        Some(ext) => {
            diagnostics
                .error(DiagnosticCode::UnknownFileFormat)
                .with_error_label(span, format!("unknown file format {:?}", ext))
                .emit();
            Err(Error::Diagnostic)
        },
        None => {
            diagnostics
                .error(DiagnosticCode::MissingFileExtension)
                .with_error_label(span, "missing file extension")
                .with_note("need file extension to differentiate file type")
                .emit();
            Err(Error::Diagnostic)
        },
    }
}
