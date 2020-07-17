//! Result type of the resolution of a file include.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use url::{Url, ParseError};

use super::BASE_URL;

/// Represents the current context we're including from
#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    /// Full URL to the file / content
    ///
    /// Can be `heradoc://document/foo/bar.md` (relative) or `file:///foo/bar.md` (absolute)
    /// or `https://foo.bar/baz/qux.md` (remote).
    pub(super) url: Url,
}

/// Type of a Context (Relative / Absolute / Remote)
#[derive(Debug, PartialEq, Eq)]
pub enum ContextType {
    /// A local file, relative to the project root
    LocalRelative,
    /// A local file with an absolute path
    LocalAbsolute,
    /// A remote resource
    Remote,
}

impl Context {
    /// Creates a context from the given path.
    ///
    /// A relative path will be interpreted as being relative to the project_root (`heradoc://document/…`).
    /// An absolute path will return an absolute `file:///…` context.
    pub fn from_path<P: AsRef<Path>>(p: P) -> Result<Self, ParseError> {
        let p = p.as_ref();
        let url = if p.is_relative() {
            Url::parse(BASE_URL).unwrap()
        } else {
            Url::parse("file:///").unwrap()
        };
        let url = url.join(&p.display().to_string())?;

        Ok(Context {
            url,
        })
    }

    pub fn from_project_root() -> Self {
        // the project root is "." if interpreted as relative to the project root
        Context {
            url: Url::parse(BASE_URL).unwrap(),
        }
    }

    pub fn from_url(url: Url) -> Self {
        Context {
            url,
        }
    }

    pub fn typ(&self) -> ContextType {
        match self.url.scheme() {
            "heradoc" => ContextType::LocalRelative,
            "file" => ContextType::LocalAbsolute,
            _ => ContextType::Remote,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

/// Typed representation of the resolved resource.
///
/// This is matched on by the `Generator` to call the respective appropriate handler
#[derive(Debug, PartialEq, Eq)]
pub enum Include {
    Command(Command),
    Markdown(PathBuf, Context),
    Image(PathBuf),
    Svg(PathBuf),
    Pdf(PathBuf),
    Graphviz(PathBuf),
}

/// A direct command to the generator.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Table of contents.
    Toc,
    /// References / Bibliography
    Bibliography,
    /// List of Tables
    ListOfTables,
    // List of Figures
    ListOfFigures,
    /// List of Listings / Code blocks
    ListOfListings,
    /// Appendix formatting
    Appendix,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(domain: &str) -> Result<Self, ()> {
        if domain.eq_ignore_ascii_case("toc") || domain.eq_ignore_ascii_case("tableofcontents") {
            Ok(Command::Toc)
        } else if domain.eq_ignore_ascii_case("bibliography")
            || domain.eq_ignore_ascii_case("references")
        {
            Ok(Command::Bibliography)
        } else if domain.eq_ignore_ascii_case("listoftables") {
            Ok(Command::ListOfTables)
        } else if domain.eq_ignore_ascii_case("listoffigures") {
            Ok(Command::ListOfFigures)
        } else if domain.eq_ignore_ascii_case("listoflistings") {
            Ok(Command::ListOfListings)
        } else if domain.eq_ignore_ascii_case("appendix") {
            Ok(Command::Appendix)
        } else {
            Err(())
        }
    }
}
