use std::borrow::Cow;

pub use pulldown_cmark::Alignment;

use super::Link;

// extension of pulldown_cmark::Event with custom types
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
    SoftBreak,
    HardBreak,
}

#[derive(Debug)]
pub struct FootnoteReference<'a> {
    pub label: Cow<'a, str>,
}

// extension of pulldown_cmark::Tag with custom types
#[derive(Debug)]
pub enum Tag<'a> {
    Paragraph,
    Rule,
    Header(Header),
    BlockQuote,
    CodeBlock(CodeBlock<'a>),
    List,
    Enumerate(Enumerate),
    Item,
    FootnoteDefinition(FootnoteDefinition<'a>),

    Table(Table),
    TableHead,
    TableRow,
    TableCell,

    InlineEmphasis,
    InlineStrong,
    InlineCode,
    InlineMath,

    Equation,
    NumberedEquation,
    Graphviz(Graphviz<'a>),
}

#[derive(Debug)]
pub struct Header {
    pub level: i32,
}

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub language: Cow<'a, str>,
}

#[derive(Debug)]
pub struct Enumerate {
    pub start_number: usize,
}

#[derive(Debug)]
pub struct FootnoteDefinition<'a> {
    pub label: Cow<'a, str>,
}

#[derive(Debug)]
pub struct Table {
    pub alignment: Vec<Alignment>,
}

#[derive(Debug)]
pub struct Image<'a> {
    pub dst: Cow<'a, str>,
    pub width: Option<Cow<'a, str>>,
    pub height: Option<Cow<'a, str>>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Graphviz<'a> {
    pub scale: Option<&'a str>,
    pub width: Option<&'a str>,
    pub height: Option<&'a str>,
    pub caption: Option<&'a str>,
}

