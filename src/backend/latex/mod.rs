use std::borrow::Cow;
use std::io::{Result, Write};
use diagnostic::Spanned;

mod complex;
mod document;
mod preamble;
mod replace;
mod simple;

pub use self::document::{Article, Beamer, BeamerFrameEvent, Report, Thesis};

use self::simple::{
    AppendixGen,
    BiberReferencesGen,
    BibliographyGen,
    FootnoteReferenceGen,
    SoftBreakGen,
    HardBreakGen,
    RuleGen,
    PageBreakGen,
    ImageGen,
    SvgGen,
    InterLinkGen,
    LabelGen,
    LatexGen,
    ListOfFiguresGen,
    ListOfListingsGen,
    ListOfTablesGen,
    PdfGen,
    TableOfContentsGen,
    TaskListMarkerGen,
    TextGen,
    UrlGen,
};

use self::complex::{
    BlockQuoteGen,
    BookHeaderGen,
    CodeBlockGen,
    EnumerateGen,
    EquationGen,
    FigureGen,
    FootnoteDefinitionGen,
    GraphvizGen,
    HeaderGen,
    BeamerHeaderGen,
    InlineCodeGen,
    InlineEmphasisGen,
    InlineMathGen,
    InlineStrikethroughGen,
    InlineStrongGen,
    InterLinkWithContentGen,
    ItemGen,
    ListGen,
    NumberedEquationGen,
    ParagraphGen,
    BeamerPageBreakGen,
    TableCellGen,
    TableFigureGen,
    TableGen,
    TableHeadGen,
    TableRowGen,
    UrlWithContentGen,
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
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
    environment: &'static str,
}

impl<'a> InlineEnvironment<'a> {
    pub fn new_figure(
        label: Option<Spanned<Cow<'a, str>>>, caption: Option<Spanned<Cow<'a, str>>>,
    ) -> InlineEnvironment<'a> {
        InlineEnvironment { label, caption, environment: "figure" }
    }

    pub fn new_table(
        label: Option<Spanned<Cow<'a, str>>>, caption: Option<Spanned<Cow<'a, str>>>,
    ) -> InlineEnvironment<'a> {
        InlineEnvironment { label, caption, environment: "table" }
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

        if let Some(Spanned { value: caption, .. }) = &self.caption {
            if self.label.is_some() {
                writeln!(out, "\\caption{{{}}}", caption)?;
            } else {
                writeln!(out, "\\caption*{{{}}}", caption)?;
            }
        } else if self.label.is_some() {
            writeln!(out, "\\caption{{}}")?;
        }

        if let Some(Spanned { value: label, .. }) = &self.label {
            writeln!(out, "\\label{{{}}}", label)?;
        }

        writeln!(out, "\\end{{{}}}", self.environment)?;

        Ok(())
    }
}
