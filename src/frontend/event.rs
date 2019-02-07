use std::borrow::Cow;

pub use pulldown_cmark::Alignment;

use super::Link;

// extension of pulldown_cmark::Event with custom types
#[derive(Debug)]
pub enum Event<'a> {
    Start(Tag<'a>),
    /// End Events may have invalid data, the data should be taken from the
    /// respective Start event.
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    InlineHtml(Cow<'a, str>),
    FootnoteReference(FootnoteReference<'a>),
    Link(Link<'a>),
    Include(Include<'a>),
    Label(Cow<'a, str>),
    SoftBreak,
    HardBreak,
}

#[derive(Debug)]
pub struct FootnoteReference<'a> {
    pub label: Cow<'a, str>,
}

// extension of pulldown_cmark::Tag with custom types
#[derive(Debug, Clone)]
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

    Table(Table<'a>),
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
pub struct Header<'a> {
    pub label: Cow<'a, str>,
    pub level: i32,
}

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub label: Option<Cow<'a, str>>,
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
pub struct Table<'a> {
    pub label: Option<Cow<'a, str>>,
    pub alignment: Vec<Alignment>,
}

#[derive(Debug)]
pub struct Include<'a> {
    pub label: Option<Cow<'a, str>>,
    pub dst: Cow<'a, str>,
    pub width: Option<Cow<'a, str>>,
    pub height: Option<Cow<'a, str>>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Graphviz<'a> {
    pub label: Option<Cow<'a, str>>,
    pub scale: Option<&'a str>,
    pub width: Option<&'a str>,
    pub height: Option<&'a str>,
    pub caption: Option<&'a str>,
}

