use std::borrow::Cow;
use diagnostic::Spanned;

pub use pulldown_cmark::Alignment;

use enum_kinds::EnumKind;

use crate::resolve::{Command, ResolveSecurity};

// extension of pulldown_cmark::Event with custom types
#[derive(Debug, EnumKind)]
#[enum_kind(EventKind)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    Latex(Cow<'a, str>),
    FootnoteReference(FootnoteReference<'a>),
    BiberReferences(Vec<BiberReference<'a>>),
    /// Url without content
    Url(Url<'a>),
    /// InterLink without content
    InterLink(InterLink<'a>),
    /// An include created with `![](foo)`
    Include(Include<'a>),
    /// Include to be resolved by the resolver
    ResolveInclude(Cow<'a, str>),
    Label(Cow<'a, str>),
    SoftBreak,
    HardBreak,
    Rule,
    PageBreak,
    TaskListMarker(TaskListMarker),

    Command(Command),
}

#[derive(Debug, Clone)]
pub struct FootnoteReference<'a> {
    pub label: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub struct BiberReference<'a> {
    pub reference: Cow<'a, str>,
    pub attributes: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Url<'a> {
    pub destination: Cow<'a, str>,
    pub title: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct InterLink<'a> {
    pub label: Cow<'a, str>,
    pub uppercase: bool,
}

#[derive(Debug, Clone)]
pub struct TaskListMarker {
    pub checked: bool,
}

// extension of pulldown_cmark::Tag with custom types
#[derive(Debug, Clone)]
pub enum Tag<'a> {
    Paragraph,
    Header(Header<'a>),
    BlockQuote,
    CodeBlock(CodeBlock<'a>),
    List,
    Enumerate(Enumerate),
    Item,
    FootnoteDefinition(FootnoteDefinition<'a>),
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

#[derive(Debug, Clone)]
pub struct Header<'a> {
    pub label: Spanned<Cow<'a, str>>,
    pub level: i32,
}

#[derive(Debug, Clone)]
pub struct CodeBlock<'a> {
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
    pub language: Option<Spanned<Cow<'a, str>>>,
}

#[derive(Debug, Clone)]
pub struct Enumerate {
    pub start_number: u64,
}

#[derive(Debug, Clone)]
pub struct FootnoteDefinition<'a> {
    pub label: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub struct Figure<'a> {
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
}

#[derive(Debug, Clone)]
pub struct ColumnWidthPercent(pub f32);

#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
    pub columns: Vec<(Alignment, ColumnWidthPercent)>
}

#[derive(Debug, Clone)]
pub struct Include<'a> {
    pub resolve_security: ResolveSecurity,
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
    pub title: Option<Cow<'a, str>>,
    /// rendered already
    pub alt_text: Option<String>,
    pub dst: Cow<'a, str>,
    pub scale: Option<Spanned<Cow<'a, str>>>,
    pub width: Option<Spanned<Cow<'a, str>>>,
    pub height: Option<Spanned<Cow<'a, str>>>,
}

#[derive(Debug, Clone)]
pub struct Equation<'a> {
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
}

#[derive(Debug, Clone)]
pub struct Graphviz<'a> {
    pub label: Option<Spanned<Cow<'a, str>>>,
    pub caption: Option<Spanned<Cow<'a, str>>>,
    pub scale: Option<Spanned<Cow<'a, str>>>,
    pub width: Option<Spanned<Cow<'a, str>>>,
    pub height: Option<Spanned<Cow<'a, str>>>,
}
