//! Result type of the resolution of a file include.

use std::path::PathBuf;
use std::str::FromStr;

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
