use std::borrow::Cow;
use std::path::PathBuf;

pub use crate::frontend::{
    Tag,
    BiberReference,
    CodeBlock,
    Enumerate,
    Figure,
    FootnoteDefinition,
    FootnoteReference,
    Graphviz,
    Header,
    InterLink,
    MathBlock,
    MathBlockKind,
    Table,
    TaskListMarker,
    Url,
};
pub use pulldown_cmark::Alignment;

use crate::frontend::Event as FeEvent;
use crate::frontend::range::WithRange;
use crate::generator::Events;
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
    IncludeMarkdown(Box<Events<'a>>),
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

/// Image to display as figure.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Image<'a> {
    pub label: Option<WithRange<Cow<'a, str>>>,
    pub caption: Option<WithRange<Cow<'a, str>>>,
    pub title: Option<Cow<'a, str>>,
    pub alt_text: Option<String>,
    /// Path to read image from.
    pub path: PathBuf,
    pub scale: Option<WithRange<Cow<'a, str>>>,
    pub width: Option<WithRange<Cow<'a, str>>>,
    pub height: Option<WithRange<Cow<'a, str>>>,
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
            FeEvent::Start(tag) => Event::Start(tag),
            FeEvent::End(tag) => Event::End(tag),
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
