use std::borrow::Cow;
use std::path::PathBuf;

pub use pulldown_cmark::Alignment;
pub use crate::frontend::{FootnoteReference, Link, Header, CodeBlock, Enumerate, FootnoteDefinition,
    Figure, Table, Graphviz, Equation};
use crate::frontend::{Event as FeEvent, Tag as FeTag};

// transformation of frontend::Event
#[derive(Debug)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    InlineHtml(Cow<'a, str>),
    FootnoteReference(FootnoteReference<'a>),
    Link(Link<'a>),
    Image(Image<'a>),
    Label(Cow<'a, str>),
    Pdf(Pdf),
    SoftBreak,
    HardBreak,
    TableOfContents,
    Bibliography,
    ListOfTables,
    ListOfFigures,
    ListOfListings,
    Appendix,
}

// transformation of frontend::Tag
#[derive(Debug)]
pub enum Tag<'a> {
    Paragraph,
    Rule,
    Header(Header<'a>),
    BlockQuote,
    CodeBlock(CodeBlock<'a>),
    List,
    Enumerate(Enumerate),
    Item,
    FootnoteDefinition(FootnoteDefinition<'a>),
    Figure(Figure<'a>),
    TableFigure(Figure<'a>),

    Table(Table<'a>),
    TableHead,
    TableRow,
    TableCell,

    InlineEmphasis,
    InlineStrong,
    InlineCode,
    InlineMath,

    Equation(Equation<'a>),
    NumberedEquation(Equation<'a>),
    Graphviz(Graphviz<'a>),
}

/// Image to display as figure.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Image<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
    /// Path to read image from.
    pub path: PathBuf,
    pub scale: Option<Cow<'a, str>>,
    pub width: Option<Cow<'a, str>>,
    pub height: Option<Cow<'a, str>>,
}

/// Pdf to include at that point inline.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Pdf {
    /// Path to read pdf from.
    pub path: PathBuf,
}

impl<'a> From<FeEvent<'a>> for Event<'a> {
    fn from(e: FeEvent<'a>) -> Self {
        match e {
            FeEvent::Start(tag) => Event::Start(tag.into()),
            FeEvent::End(tag) => Event::End(tag.into()),
            FeEvent::Text(text) => Event::Text(text),
            FeEvent::Html(html) => Event::Html(html),
            FeEvent::InlineHtml(html) => Event::InlineHtml(html),
            FeEvent::FootnoteReference(fnote) => Event::FootnoteReference(fnote),
            FeEvent::Link(link) => Event::Link(link),
            FeEvent::Include(_img) => unreachable!("handled by Generator"),
            FeEvent::Label(label) => Event::Label(label),
            FeEvent::SoftBreak => Event::SoftBreak,
            FeEvent::HardBreak => Event::HardBreak,
        }
    }
}

impl<'a> From<FeTag<'a>> for Tag<'a> {
    fn from(tag: FeTag<'a>) -> Self {
        match tag {
            FeTag::Paragraph => Tag::Paragraph,
            FeTag::Rule => Tag::Rule,
            FeTag::Header(header) => Tag::Header(header),
            FeTag::BlockQuote => Tag::BlockQuote,
            FeTag::CodeBlock(code) => Tag::CodeBlock(code),
            FeTag::List => Tag::List,
            FeTag::Enumerate(e) => Tag::Enumerate(e),
            FeTag::Item => Tag::Item,
            FeTag::FootnoteDefinition(fnote) => Tag::FootnoteDefinition(fnote),
            FeTag::Figure(figure) => Tag::Figure(figure),
            FeTag::TableFigure(figure) => Tag::TableFigure(figure),
            FeTag::Table(table) => Tag::Table(table),
            FeTag::TableHead => Tag::TableHead,
            FeTag::TableRow => Tag::TableRow,
            FeTag::TableCell => Tag::TableCell,
            FeTag::InlineEmphasis => Tag::InlineEmphasis,
            FeTag::InlineStrong => Tag::InlineStrong,
            FeTag::InlineCode => Tag::InlineCode,
            FeTag::InlineMath => Tag::InlineMath,
            FeTag::Equation(equation) => Tag::Equation(equation),
            FeTag::NumberedEquation(equation) => Tag::NumberedEquation(equation),
            FeTag::Graphviz(graphviz) => Tag::Graphviz(graphviz),
        }
    }
}
