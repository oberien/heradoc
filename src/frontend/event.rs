use std::borrow::Cow;

pub use pulldown_cmark::Alignment;

use super::Link;
use crate::resolve::Command;

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
    Include(Include<'a>),
    Label(Cow<'a, str>),
    SoftBreak,
    HardBreak,

    Command(Command),
    /// Include to be resolved by the resolver
    ResolveInclude(Cow<'a, str>),
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

#[derive(Debug, Clone)]
pub struct Header<'a> {
    pub label: Cow<'a, str>,
    pub level: i32,
}

#[derive(Debug, Clone)]
pub struct CodeBlock<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
    pub language: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub start_number: usize,
}

#[derive(Debug, Clone)]
pub struct FootnoteDefinition<'a> {
    pub label: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub struct Figure<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
    pub alignment: Vec<Alignment>,
}

#[derive(Debug, Clone)]
pub struct Include<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
    pub dst: Cow<'a, str>,
    pub scale: Option<Cow<'a, str>>,
    pub width: Option<Cow<'a, str>>,
    pub height: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Equation<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Graphviz<'a> {
    pub label: Option<Cow<'a, str>>,
    pub caption: Option<Cow<'a, str>>,
    pub scale: Option<Cow<'a, str>>,
    pub width: Option<Cow<'a, str>>,
    pub height: Option<Cow<'a, str>>,
}

