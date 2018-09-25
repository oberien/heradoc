mod article;
mod simple;
mod complex;

pub use self::article::Article;

use self::simple::{TextGen, FootnoteReferenceGen, SoftBreakGen, HardBreakGen};

use self::complex::{
    ParagraphGen,
    RuleGen,
    HeaderGen,
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
    LinkGen,
    ImageGen,
};


