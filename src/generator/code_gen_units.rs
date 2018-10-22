use std::io::{Write, Result};

use crate::backend::{Backend, CodeGenUnit};
use crate::generator::{PrimitiveGenerator, Stack};
use crate::config::Config;
use crate::generator::event::{Event, Tag};
use crate::resolve::Context;

#[derive(Debug)]
pub enum StackElement<'a, D: Backend<'a>> {
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
    InlineMath(D::InlineMath),
    Equation(D::Equation),
    NumberedEquation(D::NumberedEquation),
    Graphviz(D::Graphviz),

    // resolve context
    Context(Context),
}

impl<'a, D: Backend<'a>> StackElement<'a, D> {
    pub fn new(cfg: &'a Config, tag: Tag<'a>, gen: &mut PrimitiveGenerator<'a, D, impl Write>) -> Result<Self> {
        match tag {
            Tag::Paragraph => Ok(StackElement::Paragraph(D::Paragraph::new(cfg, (), gen)?)),
            Tag::Rule => Ok(StackElement::Rule(D::Rule::new(cfg, (), gen)?)),
            Tag::Header(header) => Ok(StackElement::Header(D::Header::new(cfg, header, gen)?)),
            Tag::BlockQuote => Ok(StackElement::BlockQuote(D::BlockQuote::new(cfg, (), gen)?)),
            Tag::CodeBlock(cb) => Ok(StackElement::CodeBlock(D::CodeBlock::new(cfg, cb, gen)?)),
            Tag::List => Ok(StackElement::List(D::List::new(cfg, (), gen)?)),
            Tag::Enumerate(enumerate) => Ok(StackElement::Enumerate(D::Enumerate::new(cfg, enumerate, gen)?)),
            Tag::Item => Ok(StackElement::Item(D::Item::new(cfg, (), gen)?)),
            Tag::FootnoteDefinition(fnote) => Ok(StackElement::FootnoteDefinition(D::FootnoteDefinition::new(cfg, fnote, gen)?)),
            Tag::Table(table) => Ok(StackElement::Table(D::Table::new(cfg, table, gen)?)),
            Tag::TableHead => Ok(StackElement::TableHead(D::TableHead::new(cfg, (), gen)?)),
            Tag::TableRow => Ok(StackElement::TableRow(D::TableRow::new(cfg, (), gen)?)),
            Tag::TableCell => Ok(StackElement::TableCell(D::TableCell::new(cfg, (), gen)?)),
            Tag::InlineEmphasis => Ok(StackElement::InlineEmphasis(D::InlineEmphasis::new(cfg, (), gen)?)),
            Tag::InlineStrong => Ok(StackElement::InlineStrong(D::InlineStrong::new(cfg, (), gen)?)),
            Tag::InlineCode => Ok(StackElement::InlineCode(D::InlineCode::new(cfg, (), gen)?)),
            Tag::InlineMath => Ok(StackElement::InlineMath(D::InlineMath::new(cfg, (), gen)?)),
            Tag::Equation => Ok(StackElement::Equation(D::Equation::new(cfg, (), gen)?)),
            Tag::NumberedEquation => Ok(StackElement::NumberedEquation(D::NumberedEquation::new(cfg, (), gen)?)),
            Tag::Graphviz(graphviz) => Ok(StackElement::Graphviz(D::Graphviz::new(cfg, graphviz, gen)?)),
        }
    }

    pub fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        match self {
            StackElement::Paragraph(s) => s.output_redirect(),
            StackElement::Rule(s) => s.output_redirect(),
            StackElement::Header(s) => s.output_redirect(),
            StackElement::BlockQuote(s) => s.output_redirect(),
            StackElement::CodeBlock(s) => s.output_redirect(),
            StackElement::List(s) => s.output_redirect(),
            StackElement::Enumerate(s) => s.output_redirect(),
            StackElement::Item(s) => s.output_redirect(),
            StackElement::FootnoteDefinition(s) => s.output_redirect(),
            StackElement::Table(s) => s.output_redirect(),
            StackElement::TableHead(s) => s.output_redirect(),
            StackElement::TableRow(s) => s.output_redirect(),
            StackElement::TableCell(s) => s.output_redirect(),
            StackElement::InlineEmphasis(s) => s.output_redirect(),
            StackElement::InlineStrong(s) => s.output_redirect(),
            StackElement::InlineCode(s) => s.output_redirect(),
            StackElement::InlineMath(s) => s.output_redirect(),
            StackElement::Equation(s) => s.output_redirect(),
            StackElement::NumberedEquation(s) => s.output_redirect(),
            StackElement::Graphviz(s) => s.output_redirect(),

            StackElement::Context(_) => None,
        }
    }

    pub fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match self {
            StackElement::Paragraph(s) => s.intercept_event(stack, e),
            StackElement::Rule(s) => s.intercept_event(stack, e),
            StackElement::Header(s) => s.intercept_event(stack, e),
            StackElement::BlockQuote(s) => s.intercept_event(stack, e),
            StackElement::CodeBlock(s) => s.intercept_event(stack, e),
            StackElement::List(s) => s.intercept_event(stack, e),
            StackElement::Enumerate(s) => s.intercept_event(stack, e),
            StackElement::Item(s) => s.intercept_event(stack, e),
            StackElement::FootnoteDefinition(s) => s.intercept_event(stack, e),
            StackElement::Table(s) => s.intercept_event(stack, e),
            StackElement::TableHead(s) => s.intercept_event(stack, e),
            StackElement::TableRow(s) => s.intercept_event(stack, e),
            StackElement::TableCell(s) => s.intercept_event(stack, e),
            StackElement::InlineEmphasis(s) => s.intercept_event(stack, e),
            StackElement::InlineStrong(s) => s.intercept_event(stack, e),
            StackElement::InlineCode(s) => s.intercept_event(stack, e),
            StackElement::InlineMath(s) => s.intercept_event(stack, e),
            StackElement::Equation(s) => s.intercept_event(stack, e),
            StackElement::NumberedEquation(s) => s.intercept_event(stack, e),
            StackElement::Graphviz(s) => s.intercept_event(stack, e),

            StackElement::Context(_) => Ok(Some(e)),
        }
    }

    pub fn finish<'b>(self, tag: Tag<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        match (self, tag) {
            (StackElement::Paragraph(s), Tag::Paragraph) => s.finish(gen, peek),
            (StackElement::Rule(s), Tag::Rule) => s.finish(gen, peek),
            (StackElement::Header(s), Tag::Header(_)) => s.finish(gen, peek),
            (StackElement::BlockQuote(s), Tag::BlockQuote) => s.finish(gen, peek),
            (StackElement::CodeBlock(s), Tag::CodeBlock(_)) => s.finish(gen, peek),
            (StackElement::List(s), Tag::List) => s.finish(gen, peek),
            (StackElement::Enumerate(s), Tag::Enumerate(_)) => s.finish(gen, peek),
            (StackElement::Item(s), Tag::Item) => s.finish(gen, peek),
            (StackElement::FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(gen, peek),
            (StackElement::Table(s), Tag::Table(_)) => s.finish(gen, peek),
            (StackElement::TableHead(s), Tag::TableHead) => s.finish(gen, peek),
            (StackElement::TableRow(s), Tag::TableRow) => s.finish(gen, peek),
            (StackElement::TableCell(s), Tag::TableCell) => s.finish(gen, peek),
            (StackElement::InlineEmphasis(s), Tag::InlineEmphasis) => s.finish(gen, peek),
            (StackElement::InlineStrong(s), Tag::InlineStrong) => s.finish(gen, peek),
            (StackElement::InlineCode(s), Tag::InlineCode) => s.finish(gen, peek),
            (StackElement::InlineMath(s), Tag::InlineMath) => s.finish(gen, peek),
            (StackElement::Equation(s), Tag::Equation) => s.finish(gen, peek),
            (StackElement::NumberedEquation(s), Tag::NumberedEquation) => s.finish(gen, peek),
            (StackElement::Graphviz(s), Tag::Graphviz(_)) => s.finish(gen, peek),
            (state, tag) => unreachable!("invalid end tag {:?}, expected {:?}", tag, state),
        }
    }

    // TODO: reomve allows
    #[allow(dead_code)]
    pub fn is_code_block(&self) -> bool {
        match self {
            StackElement::CodeBlock(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_list(&self) -> bool {
        match self {
            StackElement::List(_) => true,
            _ => false,
        }
    }

    pub fn is_enumerate(&self) -> bool {
        match self {
            StackElement::Enumerate(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_inline(&self) -> bool {
        self.is_inline_emphasis() || self.is_inline_strong() || self.is_inline_code()
    }

    #[allow(dead_code)]
    pub fn is_inline_emphasis(&self) -> bool {
        match self {
            StackElement::InlineEmphasis(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_inline_strong(&self) -> bool {
        match self {
            StackElement::InlineStrong(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_inline_code(&self) -> bool {
        match self {
            StackElement::InlineCode(_) => true,
            _ => false
        }
    }
}

