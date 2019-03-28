use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use url::Url;

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

impl Source {
    pub fn new(url: Url, context: &Context) -> io::Result<Self> {
        let group = match url.scheme() {
            "heradoc" => match url.domain() {
                Some("document") => {
                    let workdir = context.path().ok_or(io::ErrorKind::PermissionDenied)?;
                    // url is "heradoc://document/path"
                    SourceGroup::LocalRelative(to_path(&url.as_str()[19..], workdir)?)
                },
                _ => SourceGroup::Implementation,
            },
            "file" => {
                let workdir = context.path().ok_or(io::ErrorKind::PermissionDenied)?;
                let path = to_path(url.path(), workdir)?;
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

    pub fn into_include(self, remote: &Remote) -> io::Result<Include> {
        let Source { url, group } = self;
        match group {
            SourceGroup::Implementation => {
                if let Some(domain) = url.domain() {
                    if let Ok(command) = Command::from_str(domain) {
                        Ok(Include::Command(command))
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No heradoc implementation found for domain {:?}", domain),
                        ))
                    }
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "No heradoc implementation domain found",
                    ))
                }
            },
            SourceGroup::LocalRelative(path) => {
                let parent = path.parent().unwrap().to_owned();
                to_include(path, Context::LocalRelative(parent))
            },
            SourceGroup::LocalAbsolute(path) => {
                let parent = path.parent().unwrap().to_owned();
                to_include(path, Context::LocalAbsolute(parent))
            },
            SourceGroup::Remote => {
                let downloaded = match remote.http(&url) {
                    Ok(downloaded) => downloaded,
                    Err(RemoteError::Io(io)) => return Err(io),
                    // TODO: proper error handling with failure
                    Err(RemoteError::Request(_req)) => {
                        return Err(io::ErrorKind::ConnectionAborted.into());
                    },
                };

                let path = downloaded.path().to_owned();
                let context = Context::Remote;

                match downloaded.content_type() {
                    Some(ContentType::Image) => Ok(Include::Image(path)),
                    Some(ContentType::Markdown) => Ok(Include::Markdown(path, context)),
                    Some(ContentType::Pdf) => Ok(Include::Pdf(path)),
                    None => to_include(path, context),
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
fn to_include(path: PathBuf, context: Context) -> io::Result<Include> {
    // TODO: switch on file header type first
    match path.extension().map(|s| s.to_str().unwrap()) {
        Some("md") => Ok(Include::Markdown(path, context)),
        Some("png") | Some("jpg") | Some("jpeg") => Ok(Include::Image(path)),
        Some("pdf") => Ok(Include::Pdf(path)),
        Some(ext) => {
            Err(io::Error::new(io::ErrorKind::NotFound, format!("Unknown file format `{:?}`", ext)))
        },
        None => Err(io::Error::new(io::ErrorKind::NotFound, "no file extension")),
    }
}

fn to_path<P: AsRef<Path>>(path: &str, workdir: P) -> io::Result<PathBuf> {
    let path = Path::new(&path);
    let old_workdir = env::current_dir()?;
    env::set_current_dir(workdir)?;
    let path = path.canonicalize()?;
    env::set_current_dir(old_workdir)?;
    Ok(path)
}
