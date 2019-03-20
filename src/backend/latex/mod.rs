use std::io::{Write, Result};
use std::borrow::Cow;

mod document;
mod presentation;
mod preamble;
mod replace;
mod simple;
mod complex;

pub use self::document::{Article, Report, Thesis};

use self::simple::{
    TextGen,
    LatexGen,
    FootnoteReferenceGen,
    LinkGen,
    ImageGen,
    LabelGen,
    PdfGen,
    SoftBreakGen,
    HardBreakGen,
    TableOfContentsGen,
    BibliographyGen,
    ListOfTablesGen,
    ListOfFiguresGen,
    ListOfListingsGen,
    AppendixGen,
};

use self::complex::{
    ParagraphGen,
    RuleGen,
    HeaderGen,
    BookHeaderGen,
    BlockQuoteGen,
    CodeBlockGen,
    ListGen,
    EnumerateGen,
    ItemGen,
    FootnoteDefinitionGen,
    FigureGen,
    TableFigureGen,
    TableGen,
    TableHeadGen,
    TableRowGen,
    TableCellGen,
    InlineEmphasisGen,
    InlineStrongGen,
    InlineCodeGen,
    InlineMathGen,
    EquationGen,
    NumberedEquationGen,
    GraphvizGen,
};

/// Used for inline elements (not wrapped in a floating figure) that want a label or caption.
///
/// Latex requires a figure to be able to have a caption.
/// Also labels not in an environment reference the section instead of the element.
/// There is `\captionof`, but that can result in a floating Figure 3 to appear before the inline
/// Figure 2, which might be surprising.
/// Thus we create an inline figure / table with placement specifier `H` (from the `float` package).
#[derive(Debug)]
struct InlineEnvironment<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
    environment: &'static str,
}

impl<'a> InlineEnvironment<'a> {
    pub fn new_figure(label: Option<Cow<'a, str>>, caption: Option<Cow<'a, str>>) -> InlineEnvironment<'a> {
        InlineEnvironment {
            label,
            caption,
            environment: "figure",
        }
    }

    pub fn new_table(label: Option<Cow<'a, str>>, caption: Option<Cow<'a, str>>) -> InlineEnvironment<'a> {
        InlineEnvironment {
            label,
            caption,
            environment: "table",
        }
    }

    pub fn write_begin(&self, mut out: impl Write) -> Result<()> {
        if self.label.is_some() || self.caption.is_some() {
            writeln!(out, "\\begin{{{}}}[H]", self.environment)?;
        }
        Ok(())
    }

    pub fn write_end(&self, mut out: impl Write) -> Result<()> {
        if self.label.is_none() && self.caption.is_none() {
            return Ok(());
        }

        if let Some(caption) = &self.caption {
            if self.label.is_some() {
                writeln!(out, "\\caption{{{}}}", caption)?;
            } else {
                writeln!(out, "\\caption*{{{}}}", caption)?;
            }
        } else if self.label.is_some() {
            writeln!(out, "\\caption{{}}")?;
        }

        if let Some(label) = &self.label {
            writeln!(out, "\\label{{{}}}", label)?;
        }

        writeln!(out, "\\end{{{}}}", self.environment)?;

        Ok(())
    }
}

