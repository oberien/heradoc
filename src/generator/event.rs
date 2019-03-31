use std::borrow::Cow;
use std::path::PathBuf;

pub use crate::frontend::{
    BiberReference,
    CodeBlock,
    Enumerate,
    Equation,
    Figure,
    FootnoteDefinition,
    FootnoteReference,
    Graphviz,
    Header,
    InterLink,
    Table,
    TaskListMarker,
    Url,
};
pub use pulldown_cmark::Alignment;

use crate::frontend::{Event as FeEvent, Tag as FeTag};
use crate::resolve::Command;

// transformation of frontend::Event
#[derive(Debug)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    InlineHtml(Cow<'a, str>),
    Latex(Cow<'a, str>),
    FootnoteReference(FootnoteReference<'a>),
    BiberReferences(Vec<BiberReference<'a>>),
    /// Url without content
    Url(Url<'a>),
    /// InterLink without content
    InterLink(InterLink<'a>),
    Image(Image<'a>),
    Label(Cow<'a, str>),
    Pdf(Pdf),
    SoftBreak,
    HardBreak,
    TaskListMarker(TaskListMarker),
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
    HtmlBlock,
    /// Url with content
    Url(Url<'a>),
    /// InterLink with content
    InterLink(InterLink<'a>),
    Figure(Figure<'a>),
    TableFigure(Figure<'a>),

    Table(Table<'a>),
    TableHead,
    TableRow,
    TableCell,

    InlineEmphasis,
    InlineStrong,
    InlineStrikethrough,
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
    pub title: Option<Cow<'a, str>>,
    pub alt_text: Option<String>,
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
            FeEvent::Latex(latex) => Event::Latex(latex),
            FeEvent::FootnoteReference(fnote) => Event::FootnoteReference(fnote),
            FeEvent::BiberReferences(biber) => Event::BiberReferences(biber),
            FeEvent::Url(url) => Event::Url(url),
            FeEvent::InterLink(interlink) => Event::InterLink(interlink),
            FeEvent::Include(_img) => unreachable!("Include is handled by Generator"),
            FeEvent::Label(label) => Event::Label(label),
            FeEvent::SoftBreak => Event::SoftBreak,
            FeEvent::HardBreak => Event::HardBreak,
            FeEvent::TaskListMarker(marker) => Event::TaskListMarker(marker),

            FeEvent::Command(command) => command.into(),
            FeEvent::ResolveInclude(_include) => {
                unreachable!("ResolveInclude is handled by Generator")
            },
        }
    }
}

impl<'a> From<Command> for Event<'a> {
    fn from(command: Command) -> Self {
        match command {
            Command::Toc => Event::TableOfContents,
            Command::Bibliography => Event::Bibliography,
            Command::ListOfTables => Event::ListOfTables,
            Command::ListOfFigures => Event::ListOfFigures,
            Command::ListOfListings => Event::ListOfListings,
            Command::Appendix => Event::Appendix,
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
            FeTag::Url(url) => Tag::Url(url),
            FeTag::InterLink(interlink) => Tag::InterLink(interlink),
            FeTag::HtmlBlock => Tag::HtmlBlock,
            FeTag::Figure(figure) => Tag::Figure(figure),
            FeTag::TableFigure(figure) => Tag::TableFigure(figure),
            FeTag::Table(table) => Tag::Table(table),
            FeTag::TableHead => Tag::TableHead,
            FeTag::TableRow => Tag::TableRow,
            FeTag::TableCell => Tag::TableCell,
            FeTag::InlineEmphasis => Tag::InlineEmphasis,
            FeTag::InlineStrong => Tag::InlineStrong,
            FeTag::InlineStrikethrough => Tag::InlineStrikethrough,
            FeTag::InlineCode => Tag::InlineCode,
            FeTag::InlineMath => Tag::InlineMath,
            FeTag::Equation(equation) => Tag::Equation(equation),
            FeTag::NumberedEquation(equation) => Tag::NumberedEquation(equation),
            FeTag::Graphviz(graphviz) => Tag::Graphviz(graphviz),
        }
    }
}
