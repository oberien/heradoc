use std::borrow::Cow;

use pulldown_cmark::{Alignment, LinkType, Event as CmarkEvent, Tag as CmarkTag, Parser as CmarkParser};

pub struct Parser<'a> {
    parser: CmarkParser<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(parser: CmarkParser<'a>) -> Parser {
        Parser {
            parser,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        self.parser.next().map(|e| e.into())
    }
}

// extension of pulldown_cmark::Event with custom types
#[derive(Debug)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    InlineHtml(Cow<'a, str>),
    FootnoteReference(FootnoteReference<'a>),
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

    Emphasis,
    Strong,
    Code,

    Link(Link<'a>),
    Image(Link<'a>),
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
pub struct Link<'a> {
    pub typ: LinkType,
    pub dst: Cow<'a, str>,
    pub title: Cow<'a, str>,
}

impl<'a> From<CmarkEvent<'a>> for Event<'a> {
    fn from(evt: CmarkEvent<'a>) -> Self {
        match evt {
            CmarkEvent::Start(tag) => Event::Start(tag.into()),
            CmarkEvent::End(tag) => Event::End(tag.into()),
            CmarkEvent::Text(text) => Event::Text(text),
            CmarkEvent::Html(html) => Event::Html(html),
            CmarkEvent::InlineHtml(html) => Event::InlineHtml(html),
            CmarkEvent::FootnoteReference(label) => Event::FootnoteReference(FootnoteReference {
                label,
            }),
            CmarkEvent::SoftBreak => Event::SoftBreak,
            CmarkEvent::HardBreak => Event::HardBreak,
        }
    }
}

impl<'a> From<CmarkTag<'a>> for Tag<'a> {
    fn from(tag: CmarkTag<'a>) -> Self {
        match tag {
            CmarkTag::Paragraph => Tag::Paragraph,
            CmarkTag::Rule => Tag::Rule,
            CmarkTag::Header(level) => Tag::Header(Header { level }),
            CmarkTag::BlockQuote => Tag::BlockQuote,
            CmarkTag::CodeBlock(language) => Tag::CodeBlock(CodeBlock { language }),
            CmarkTag::List(start_number) if start_number.is_none() => Tag::List,
            CmarkTag::List(start_number) => Tag::Enumerate(Enumerate {
                start_number: start_number.unwrap()
            }),
            CmarkTag::Item => Tag::Item,
            CmarkTag::FootnoteDefinition(label) => Tag::FootnoteDefinition(FootnoteDefinition {
                label
            }),
            CmarkTag::Table(alignment) => Tag::Table(Table { alignment }),
            CmarkTag::TableHead => Tag::TableHead,
            CmarkTag::TableRow => Tag::TableRow,
            CmarkTag::TableCell => Tag::TableCell,
            CmarkTag::Emphasis => Tag::Emphasis,
            CmarkTag::Strong => Tag::Strong,
            CmarkTag::Code => Tag::Code,
            CmarkTag::Link(typ, dst, title) => Tag::Link(Link { typ, dst, title }),
            CmarkTag::Image(typ, dst, title) => Tag::Image(Link { typ, dst, title }),
        }
    }
}
