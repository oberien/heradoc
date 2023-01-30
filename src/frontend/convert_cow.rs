use std::borrow::Cow;

use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event as CmarkEvent, LinkType, OffsetIter, Tag as CmarkTag};

use crate::frontend::range::WithRange;

pub struct ConvertCow<'a>(pub OffsetIter<'a, 'a>);

impl<'a> Iterator for ConvertCow<'a> {
    type Item = WithRange<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(e, r)| WithRange(e.into(), r.into()))
    }
}

fn convert<'a>(s: CowStr<'a>) -> Cow<'a, str> {
    match s {
        CowStr::Borrowed(s) => Cow::Borrowed(s),
        CowStr::Boxed(b) => Cow::Owned(b.into_string()),
        CowStr::Inlined(i) => Cow::Owned(i.to_string()),
    }
}

// `Event` and `Tag` taken from pulldown-cmark and only `CowStr` replaced to `Cow`
#[derive(Clone, Debug, PartialEq)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Code(Cow<'a, str>),
    Html(Cow<'a, str>),
    FootnoteReference(Cow<'a, str>),
    SoftBreak,
    HardBreak,
    Rule,
    /// A task list marker, rendered as a checkbox in HTML. Contains a true when it is checked
    TaskListMarker(bool),
}

impl<'a> From<CmarkEvent<'a>> for Event<'a> {
    fn from(evt: CmarkEvent<'a>) -> Event<'a> {
        match evt {
            CmarkEvent::Start(tag) => Event::Start(tag.into()),
            CmarkEvent::End(tag) => Event::End(tag.into()),
            CmarkEvent::Text(cow) => Event::Text(convert(cow)),
            CmarkEvent::Code(cow) => Event::Code(convert(cow)),
            CmarkEvent::Html(cow) => Event::Html(convert(cow)),
            CmarkEvent::FootnoteReference(cow) => Event::FootnoteReference(convert(cow)),
            CmarkEvent::SoftBreak => Event::SoftBreak,
            CmarkEvent::HardBreak => Event::HardBreak,
            CmarkEvent::Rule => Event::Rule,
            CmarkEvent::TaskListMarker(b) => Event::TaskListMarker(b),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tag<'a> {
    // block-level tags
    Paragraph,

    /// A heading. The field indicates the level of the heading.
    Header(i32),

    BlockQuote,
    CodeBlock(Cow<'a, str>),

    /// A list. If the list is ordered the field indicates the number of the first item.
    List(Option<u64>), // TODO: add delim and tight for ast (not needed for html)
    Item,
    FootnoteDefinition(Cow<'a, str>),

    // tables
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,

    // span-level tags
    Emphasis,
    Strong,
    Strikethrough,

    /// A link. The first field is the link type, the second the destination URL and the third is a
    /// title
    Link(LinkType, Cow<'a, str>, Cow<'a, str>),

    /// An image. The first field is the link type, the second the destination URL and the third is
    /// a title
    Image(LinkType, Cow<'a, str>, Cow<'a, str>),
}

impl<'a> From<CmarkTag<'a>> for Tag<'a> {
    fn from(tag: CmarkTag<'a>) -> Self {
        match tag {
            CmarkTag::Paragraph => Tag::Paragraph,
            CmarkTag::Heading(level, _, _) => Tag::Header(level as i32),
            CmarkTag::BlockQuote => Tag::BlockQuote,
            CmarkTag::CodeBlock(CodeBlockKind::Indented) => Tag::CodeBlock(Cow::Borrowed("")),
            CmarkTag::CodeBlock(CodeBlockKind::Fenced(cow)) => Tag::CodeBlock(convert(cow)),
            CmarkTag::List(start) => Tag::List(start),
            CmarkTag::Item => Tag::Item,
            CmarkTag::FootnoteDefinition(cow) => Tag::FootnoteDefinition(convert(cow)),
            CmarkTag::Table(alignment) => Tag::Table(alignment),
            CmarkTag::TableHead => Tag::TableHead,
            CmarkTag::TableRow => Tag::TableRow,
            CmarkTag::TableCell => Tag::TableCell,
            CmarkTag::Emphasis => Tag::Emphasis,
            CmarkTag::Strong => Tag::Strong,
            CmarkTag::Strikethrough => Tag::Strikethrough,
            CmarkTag::Link(typ, dst, title) => Tag::Link(typ, convert(dst), convert(title)),
            CmarkTag::Image(typ, dst, title) => Tag::Image(typ, convert(dst), convert(title)),
        }
    }
}
