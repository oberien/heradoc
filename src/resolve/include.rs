//! Result type of the resolution of a file include.

use std::path::PathBuf;

use crate::resolve::Context;

/// Typed representation of the resolved resource.
///
/// This is matched on by the `Generator` to call the respective appropriate handler
#[derive(Debug, PartialEq, Eq)]
pub enum Include {
    Command(Command),
    Markdown(PathBuf, Context),
    Image(PathBuf),
    Pdf(PathBuf),
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
    /// Appendix formatting
    Appendix,
}
