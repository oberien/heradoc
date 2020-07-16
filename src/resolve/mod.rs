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

mod include;
pub mod remote;
mod target;
#[cfg(test)]
mod tests;

pub use self::include::*;
use self::remote::Remote;
use self::target::Target;
use crate::diagnostics::Diagnostics;
use crate::error::Result;
use crate::frontend::range::SourceRange;

const BASE_URL: &'static str = "heradoc://project/";

pub struct Resolver {
    permissions: Permissions,
    project_root: PathBuf,
    document_root: PathBuf,
    remote: Remote,
}

#[derive(Debug, Clone, Copy)]
pub enum ResolveSecurity {
    Default,
    /// Skip permission and security checks when resolving an include.
    /// If this option is used, the path to-be-resolved must be either relative to the document
    /// directory, or an absolute file using the `file:///foo/bar` URL scheme.
    /// This should be used only if includes are created within heradoc and are known to be safe
    /// but would otherwise fail the checks during the resolve phase.
    /// This can be the case if files are created by heradoc and written to the out_dir to be included.
    SkipChecks,
}

/// Manages permissions if includes as allowed explicitly from the Cli.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Permissions {
    allowed_absolute_folders: Vec<PathBuf>,
}

impl Resolver {
    pub fn new(project_root: PathBuf, document_root: PathBuf, tempdir: PathBuf) -> Self {
        Resolver {
            permissions: Permissions { allowed_absolute_folders: vec![project_root.clone()] },
            project_root,
            document_root,
            remote: Remote::new(tempdir).unwrap(),
        }
    }

    /// Make a request to an url in the context of a document with the specified source.
    pub fn resolve(
        &self, resolve_security: ResolveSecurity, context: &Context, url: &str, range: SourceRange,
        diagnostics: &Diagnostics<'_>,
    ) -> Result<Include> {
        let target = Target::new(url, context, &self.project_root, &self.document_root, &self.permissions, range, diagnostics)?;
        let include = match resolve_security {
            ResolveSecurity::Default =>
                target.canonicalize()?.check_access()?.into_include(&self.remote)?,
            ResolveSecurity::SkipChecks =>
                target.skip_canonicalization().skip_check_access().into_include(&self.remote)?
        };
        Ok(include)
    }
}

impl Permissions {
    fn is_allowed_absolute(&self, path: impl AsRef<Path>) -> bool {
        self.allowed_absolute_folders
            .iter()
            .any(|allowed| path.as_ref().strip_prefix(allowed).is_ok())
    }
}
