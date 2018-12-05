mod document;
mod presentation;
mod preamble;
mod replace;
mod simple;
mod complex;

pub use self::document::{Article, Beamer, Thesis};

use self::simple::{TextGen, FootnoteReferenceGen, LinkGen, ImageGen, PdfGen, SoftBreakGen, HardBreakGen,
    TableOfContentsGen, BibliographyGen, ListOfTablesGen, ListOfFiguresGen, ListOfListingsGen,
    AppendixGen};

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


