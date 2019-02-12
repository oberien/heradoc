use std::io::{Write, Result};
use std::borrow::Cow;

mod document;
mod presentation;
mod preamble;
mod replace;
mod simple;
mod complex;

pub use self::document::{Article, Report, Thesis};

use self::simple::{TextGen, FootnoteReferenceGen, LinkGen, ImageGen, LabelGen, PdfGen, SoftBreakGen,
    HardBreakGen, TableOfContentsGen, BibliographyGen, ListOfTablesGen, ListOfFiguresGen,
    ListOfListingsGen, AppendixGen};

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

fn inline_figure_begin(out: impl Write, label: &Option<Cow<'_, str>>, caption: &Option<Cow<'_, str>>) -> Result<()> {
    inline_any_figure_begin(out, "figure", label, caption)
}
fn inline_figure_end(out: impl Write, label: Option<Cow<'_, str>>, caption: Option<Cow<'_, str>>) -> Result<()> {
    inline_any_figure_end(out, "figure", label, caption)
}

fn inline_table_begin(out: impl Write, label: &Option<Cow<'_, str>>, caption: &Option<Cow<'_, str>>) -> Result<()> {
    inline_any_figure_begin(out, "table", label, caption)
}
fn inline_table_end(out: impl Write, label: Option<Cow<'_, str>>, caption: Option<Cow<'_, str>>) -> Result<()> {
    inline_any_figure_end(out, "table", label, caption)
}

fn inline_any_figure_begin(mut out: impl Write, env: &'_ str, label: &Option<Cow<'_, str>>, caption: &Option<Cow<'_, str>>) -> Result<()> {
    if label.is_some() || caption.is_some() {
        writeln!(out, "\\begin{{{}}}[H]", env)?;
    }
    Ok(())
}

fn inline_any_figure_end(mut out: impl Write, env: &'_ str, label: Option<Cow<'_, str>>, caption: Option<Cow<'_, str>>) -> Result<()> {
    if label.is_none() && caption.is_none() {
        return Ok(());
    }

    if let Some(caption) = caption {
        if label.is_some() {
            writeln!(out, "\\caption{{{}}}", caption)?;
        } else {
            writeln!(out, "\\caption*{{{}}}", caption)?;
        }
    } else if label.is_some() {
        writeln!(out, "\\caption{{}}")?;
    }

    if let Some(label) = label {
        writeln!(out, "\\label{{{}}}", label)?;
    }

    writeln!(out, "\\end{{{}}}", env)?;

    Ok(())
}
