use std::io::{Write, Result};

use crate::gen::{Backend, Generator, Stack, CodeGenUnit};
use crate::config::Config;
use crate::parser::{Event, Tag};

#[derive(Debug)]
pub enum CodeGenUnits<'a, D: Backend<'a>> {
    Paragraph(D::Paragraph),
    Rule(D::Rule),
    Header(D::Header),
    BlockQuote(D::BlockQuote),
    CodeBlock(D::CodeBlock),
    List(D::List),
    Enumerate(D::Enumerate),
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

impl<'a, D: Backend<'a>> CodeGenUnits<'a, D> {
    pub fn new(cfg: &'a Config, tag: Tag<'a>, gen: &mut Generator<'a, D, impl Write>) -> Result<Self> {
        match tag {
            Tag::Paragraph => Ok(CodeGenUnits::Paragraph(D::Paragraph::new(cfg, (), gen)?)),
            Tag::Rule => Ok(CodeGenUnits::Rule(D::Rule::new(cfg, (), gen)?)),
            Tag::Header(header) => Ok(CodeGenUnits::Header(D::Header::new(cfg, header, gen)?)),
            Tag::BlockQuote => Ok(CodeGenUnits::BlockQuote(D::BlockQuote::new(cfg, (), gen)?)),
            Tag::CodeBlock(cb) => Ok(CodeGenUnits::CodeBlock(D::CodeBlock::new(cfg, cb, gen)?)),
            Tag::List => Ok(CodeGenUnits::List(D::List::new(cfg, (), gen)?)),
            Tag::Enumerate(enumerate) => Ok(CodeGenUnits::Enumerate(D::Enumerate::new(cfg, enumerate, gen)?)),
            Tag::Item => Ok(CodeGenUnits::Item(D::Item::new(cfg, (), gen)?)),
            Tag::FootnoteDefinition(fnote) => Ok(CodeGenUnits::FootnoteDefinition(D::FootnoteDefinition::new(cfg, fnote, gen)?)),
            Tag::Table(table) => Ok(CodeGenUnits::Table(D::Table::new(cfg, table, gen)?)),
            Tag::TableHead => Ok(CodeGenUnits::TableHead(D::TableHead::new(cfg, (), gen)?)),
            Tag::TableRow => Ok(CodeGenUnits::TableRow(D::TableRow::new(cfg, (), gen)?)),
            Tag::TableCell => Ok(CodeGenUnits::TableCell(D::TableCell::new(cfg, (), gen)?)),
            Tag::Emphasis => Ok(CodeGenUnits::InlineEmphasis(D::InlineEmphasis::new(cfg, (), gen)?)),
            Tag::Strong => Ok(CodeGenUnits::InlineStrong(D::InlineStrong::new(cfg, (), gen)?)),
            Tag::Code => Ok(CodeGenUnits::InlineCode(D::InlineCode::new(cfg, (), gen)?)),
            Tag::Link(link) => Ok(CodeGenUnits::Link(D::Link::new(cfg, link, gen)?)),
            Tag::Image(link) => Ok(CodeGenUnits::Image(D::Image::new(cfg, link, gen)?)),
        }
    }

    pub fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        match self {
            CodeGenUnits::Paragraph(s) => s.output_redirect(),
            CodeGenUnits::Rule(s) => s.output_redirect(),
            CodeGenUnits::Header(s) => s.output_redirect(),
            CodeGenUnits::BlockQuote(s) => s.output_redirect(),
            CodeGenUnits::CodeBlock(s) => s.output_redirect(),
            CodeGenUnits::List(s) => s.output_redirect(),
            CodeGenUnits::Enumerate(s) => s.output_redirect(),
            CodeGenUnits::Item(s) => s.output_redirect(),
            CodeGenUnits::FootnoteDefinition(s) => s.output_redirect(),
            CodeGenUnits::Table(s) => s.output_redirect(),
            CodeGenUnits::TableHead(s) => s.output_redirect(),
            CodeGenUnits::TableRow(s) => s.output_redirect(),
            CodeGenUnits::TableCell(s) => s.output_redirect(),
            CodeGenUnits::InlineEmphasis(s) => s.output_redirect(),
            CodeGenUnits::InlineStrong(s) => s.output_redirect(),
            CodeGenUnits::InlineCode(s) => s.output_redirect(),
            CodeGenUnits::Link(s) => s.output_redirect(),
            CodeGenUnits::Image(s) => s.output_redirect(),
        }
    }

    pub fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match self {
            CodeGenUnits::Paragraph(s) => s.intercept_event(stack, e),
            CodeGenUnits::Rule(s) => s.intercept_event(stack, e),
            CodeGenUnits::Header(s) => s.intercept_event(stack, e),
            CodeGenUnits::BlockQuote(s) => s.intercept_event(stack, e),
            CodeGenUnits::CodeBlock(s) => s.intercept_event(stack, e),
            CodeGenUnits::List(s) => s.intercept_event(stack, e),
            CodeGenUnits::Enumerate(s) => s.intercept_event(stack, e),
            CodeGenUnits::Item(s) => s.intercept_event(stack, e),
            CodeGenUnits::FootnoteDefinition(s) => s.intercept_event(stack, e),
            CodeGenUnits::Table(s) => s.intercept_event(stack, e),
            CodeGenUnits::TableHead(s) => s.intercept_event(stack, e),
            CodeGenUnits::TableRow(s) => s.intercept_event(stack, e),
            CodeGenUnits::TableCell(s) => s.intercept_event(stack, e),
            CodeGenUnits::InlineEmphasis(s) => s.intercept_event(stack, e),
            CodeGenUnits::InlineStrong(s) => s.intercept_event(stack, e),
            CodeGenUnits::InlineCode(s) => s.intercept_event(stack, e),
            CodeGenUnits::Link(s) => s.intercept_event(stack, e),
            CodeGenUnits::Image(s) => s.intercept_event(stack, e),
        }
    }

    pub fn finish<'b>(self, tag: Tag<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        match (self, tag) {
            (CodeGenUnits::Paragraph(s), Tag::Paragraph) => s.finish(gen, peek),
            (CodeGenUnits::Rule(s), Tag::Rule) => s.finish(gen, peek),
            (CodeGenUnits::Header(s), Tag::Header(_)) => s.finish(gen, peek),
            (CodeGenUnits::BlockQuote(s), Tag::BlockQuote) => s.finish(gen, peek),
            (CodeGenUnits::CodeBlock(s), Tag::CodeBlock(_)) => s.finish(gen, peek),
            (CodeGenUnits::List(s), Tag::List) => s.finish(gen, peek),
            (CodeGenUnits::Enumerate(s), Tag::Enumerate(_)) => s.finish(gen, peek),
            (CodeGenUnits::Item(s), Tag::Item) => s.finish(gen, peek),
            (CodeGenUnits::FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(gen, peek),
            (CodeGenUnits::Table(s), Tag::Table(_)) => s.finish(gen, peek),
            (CodeGenUnits::TableHead(s), Tag::TableHead) => s.finish(gen, peek),
            (CodeGenUnits::TableRow(s), Tag::TableRow) => s.finish(gen, peek),
            (CodeGenUnits::TableCell(s), Tag::TableCell) => s.finish(gen, peek),
            (CodeGenUnits::InlineEmphasis(s), Tag::Emphasis) => s.finish(gen, peek),
            (CodeGenUnits::InlineStrong(s), Tag::Strong) => s.finish(gen, peek),
            (CodeGenUnits::InlineCode(s), Tag::Code) => s.finish(gen, peek),
            (CodeGenUnits::Link(s), Tag::Link(..)) => s.finish(gen, peek),
            (CodeGenUnits::Image(s), Tag::Image(..)) => s.finish(gen, peek),
            (state, tag) => unreachable!("invalid end tag {:?}, expected {:?}", tag, state),
        }
    }

    pub fn is_code_block(&self) -> bool {
        match self {
            CodeGenUnits::CodeBlock(_) => true,
            _ => false
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            CodeGenUnits::List(_) => true,
            _ => false,
        }
    }

    pub fn is_enumerate(&self) -> bool {
        match self {
            CodeGenUnits::Enumerate(_) => true,
            _ => false
        }
    }

    pub fn is_inline(&self) -> bool {
        self.is_inline_emphasis() || self.is_inline_strong() || self.is_inline_code()
    }

    pub fn is_inline_emphasis(&self) -> bool {
        match self {
            CodeGenUnits::InlineEmphasis(_) => true,
            _ => false
        }
    }

    pub fn is_inline_strong(&self) -> bool {
        match self {
            CodeGenUnits::InlineStrong(_) => true,
            _ => false
        }
    }

    pub fn is_inline_code(&self) -> bool {
        match self {
            CodeGenUnits::InlineCode(_) => true,
            _ => false
        }
    }
}

