use std::io::{Write, Result};

use pulldown_cmark::{Tag, Event};

use gen::{Document, Generator, Stack, State};

#[derive(Debug)]
pub enum States<'a, D: Document<'a>> {
    Paragraph(D::Paragraph),
    Rule(D::Rule),
    Header(D::Header),
    BlockQuote(D::BlockQuote),
    CodeBlock(D::CodeBlock),
    List(D::List),
    Item(D::Item),
    FootnoteDefinition(D::FootnoteDefinition),
    Table(D::Table),
    TableHead(D::TableHead),
    TableRow(D::TableRow),
    TableCell(D::TableCell),
    InlineEmphasis(D::InlineEmphasis),
    InlineStrong(D::InlineStrong),
    InlineCode(D::InlineCode),
    Link(D::Link),
    Image(D::Image),
}

impl<'a, D: Document<'a>> States<'a, D> {
    pub fn new(tag: Tag<'a>, gen: &mut Generator<'a, D, impl Write>) -> Result<Self> {
        match &tag {
            Tag::Paragraph => Ok(States::Paragraph(D::Paragraph::new(tag, gen)?)),
            Tag::Rule => Ok(States::Rule(D::Rule::new(tag, gen)?)),
            Tag::Header(_) => Ok(States::Header(D::Header::new(tag, gen)?)),
            Tag::BlockQuote => Ok(States::BlockQuote(D::BlockQuote::new(tag, gen)?)),
            Tag::CodeBlock(_) => Ok(States::CodeBlock(D::CodeBlock::new(tag, gen)?)),
            Tag::List(_) => Ok(States::List(D::List::new(tag, gen)?)),
            Tag::Item => Ok(States::Item(D::Item::new(tag, gen)?)),
            Tag::FootnoteDefinition(_) => Ok(States::FootnoteDefinition(D::FootnoteDefinition::new(tag, gen)?)),
            Tag::Table(_) => Ok(States::Table(D::Table::new(tag, gen)?)),
            Tag::TableHead => Ok(States::TableHead(D::TableHead::new(tag, gen)?)),
            Tag::TableRow => Ok(States::TableRow(D::TableRow::new(tag, gen)?)),
            Tag::TableCell => Ok(States::TableCell(D::TableCell::new(tag, gen)?)),
            Tag::Emphasis => Ok(States::InlineEmphasis(D::InlineEmphasis::new(tag, gen)?)),
            Tag::Strong => Ok(States::InlineStrong(D::InlineStrong::new(tag, gen)?)),
            Tag::Code => Ok(States::InlineCode(D::InlineCode::new(tag, gen)?)),
            Tag::Link(..) => Ok(States::Link(D::Link::new(tag, gen)?)),
            Tag::Image(..) => Ok(States::Image(D::Image::new(tag, gen)?)),
        }
    }

    pub fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        match self {
            States::Paragraph(s) => s.output_redirect(),
            States::Rule(s) => s.output_redirect(),
            States::Header(s) => s.output_redirect(),
            States::BlockQuote(s) => s.output_redirect(),
            States::CodeBlock(s) => s.output_redirect(),
            States::List(s) => s.output_redirect(),
            States::Item(s) => s.output_redirect(),
            States::FootnoteDefinition(s) => s.output_redirect(),
            States::Table(s) => s.output_redirect(),
            States::TableHead(s) => s.output_redirect(),
            States::TableRow(s) => s.output_redirect(),
            States::TableCell(s) => s.output_redirect(),
            States::InlineEmphasis(s) => s.output_redirect(),
            States::InlineStrong(s) => s.output_redirect(),
            States::InlineCode(s) => s.output_redirect(),
            States::Link(s) => s.output_redirect(),
            States::Image(s) => s.output_redirect(),
        }
    }

    pub fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Document<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match self {
            States::Paragraph(s) => s.intercept_event(stack, e),
            States::Rule(s) => s.intercept_event(stack, e),
            States::Header(s) => s.intercept_event(stack, e),
            States::BlockQuote(s) => s.intercept_event(stack, e),
            States::CodeBlock(s) => s.intercept_event(stack, e),
            States::List(s) => s.intercept_event(stack, e),
            States::Item(s) => s.intercept_event(stack, e),
            States::FootnoteDefinition(s) => s.intercept_event(stack, e),
            States::Table(s) => s.intercept_event(stack, e),
            States::TableHead(s) => s.intercept_event(stack, e),
            States::TableRow(s) => s.intercept_event(stack, e),
            States::TableCell(s) => s.intercept_event(stack, e),
            States::InlineEmphasis(s) => s.intercept_event(stack, e),
            States::InlineStrong(s) => s.intercept_event(stack, e),
            States::InlineCode(s) => s.intercept_event(stack, e),
            States::Link(s) => s.intercept_event(stack, e),
            States::Image(s) => s.intercept_event(stack, e),
        }
    }

    pub fn finish<'b>(self, tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        match (self, tag) {
            (States::Paragraph(s), Tag::Paragraph) => s.finish(gen, peek),
            (States::Rule(s), Tag::Rule) => s.finish(gen, peek),
            (States::Header(s), Tag::Header(_)) => s.finish(gen, peek),
            (States::BlockQuote(s), Tag::BlockQuote) => s.finish(gen, peek),
            (States::CodeBlock(s), Tag::CodeBlock(_)) => s.finish(gen, peek),
            (States::List(s), Tag::List(_)) => s.finish(gen, peek),
            (States::Item(s), Tag::Item) => s.finish(gen, peek),
            (States::FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(gen, peek),
            (States::Table(s), Tag::Table(_)) => s.finish(gen, peek),
            (States::TableHead(s), Tag::TableHead) => s.finish(gen, peek),
            (States::TableRow(s), Tag::TableRow) => s.finish(gen, peek),
            (States::TableCell(s), Tag::TableCell) => s.finish(gen, peek),
            (States::InlineEmphasis(s), Tag::Emphasis) => s.finish(gen, peek),
            (States::InlineStrong(s), Tag::Strong) => s.finish(gen, peek),
            (States::InlineCode(s), Tag::Code) => s.finish(gen, peek),
            (States::Link(s), Tag::Link(..)) => s.finish(gen, peek),
            (States::Image(s), Tag::Image(..)) => s.finish(gen, peek),
            (state, tag) => unreachable!("invalid end tag {:?}, expected {:?}", tag, state),
        }
    }

    pub fn is_code_block(&self) -> bool {
        match self {
            States::CodeBlock(_) => true,
            _ => false
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            States::List(_) => true,
            _ => false,
        }
    }

    pub fn is_inline(&self) -> bool {
        self.is_inline_emphasis() || self.is_inline_strong() || self.is_inline_code()
    }

    pub fn is_inline_emphasis(&self) -> bool {
        match self {
            States::InlineEmphasis(_) => true,
            _ => false
        }
    }

    pub fn is_inline_strong(&self) -> bool {
        match self {
            States::InlineStrong(_) => true,
            _ => false
        }
    }

    pub fn is_inline_code(&self) -> bool {
        match self {
            States::InlineCode(_) => true,
            _ => false
        }
    }
}

