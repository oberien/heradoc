//! Result type of the resolution of a file include.

use std::path::{Path, PathBuf};

use url::Origin;

use crate::resolve::{Source, Context};

/// Typed representation of the resolved resource.
///
/// This is matched on by the `Generator` to call the respective appropriate handler
#[derive(Debug, PartialEq, Eq)]
pub enum Include {
    Command(Command),
    Markdown(Markdown),
    Image(Image),
    Pdf(Pdf),
}

/// A direct command to the pundoc processor.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Table of contents.
    Toc,
    /// References / Bibliography
    Bibliography,
    /// List of Tables
    ListOfTables,
    /// List of Figures
    ListOfFigures,
    /// List of Listings / Code blocks
    ListOfListings,
}

/// A markdown file to parse and generate output for.
#[derive(Debug, PartialEq, Eq)]
pub struct Markdown {
    /// Path to read file from.
    pub path: PathBuf,
    /// Context of this file, to be used for includes of this file.
    pub context: Context,
}

/// Image to display as figure.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Image {
    /// Path to read image from.
    pub path: PathBuf,
    pub width: Option<String>,
    pub height: Option<String>,
}

/// Pdf to include at that point inline.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Pdf {
    /// Path to read pdf from.
    pub path: PathBuf,
}

impl Include {
    pub fn path(&self) -> Option<&Path> {
        match self {
            Include::Command(_) => None,
            Include::Markdown(Markdown { path, .. }) => Some(path),
            Include::Image(Image { path, .. }) => Some(path),
            Include::Pdf(Pdf { path, .. }) => Some(path),
        }
    }

    pub fn context(&self) -> Option<&Context> {
        match self {
            Include::Command(_) => None,
            Include::Markdown(Markdown { context, .. }) => Some(context),
            Include::Image(Image { .. }) => None,
            Include::Pdf(Pdf { .. }) => None,
        }
    }
}
