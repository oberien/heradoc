use std::io::Write;
use std::sync::Arc;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::diagnostics::Diagnostics;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, Tag};
use crate::generator::Generator;
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
    Url(D::UrlWithContent),
    InterLink(D::InterLinkWithContent),
    HtmlBlock(D::HtmlBlock),
    Figure(D::Figure),
    TableFigure(D::TableFigure),
    Table(D::Table),
    TableHead(D::TableHead),
    TableRow(D::TableRow),
    TableCell(D::TableCell),
    InlineEmphasis(D::InlineEmphasis),
    InlineStrong(D::InlineStrong),
    InlineStrikethrough(D::InlineStrikethrough),
    InlineCode(D::InlineCode),
    InlineMath(D::InlineMath),
    Equation(D::Equation),
    NumberedEquation(D::NumberedEquation),
    Graphviz(D::Graphviz),

    // resolve context
    Context(Context, Arc<Diagnostics<'a>>),
}

use self::StackElement::*;

#[rustfmt::skip]
impl<'a, D: Backend<'a>> StackElement<'a, D> {
    pub fn new(cfg: &'a Config, tag: WithRange<Tag<'a>>, gen: &mut Generator<'a, D, impl Write>) -> Result<Self> {
        let WithRange(tag, range) = tag;
        match tag {
            Tag::Paragraph => Ok(Paragraph(D::Paragraph::new(cfg, WithRange((), range), gen)?)),
            Tag::Rule => Ok(Rule(D::Rule::new(cfg, WithRange((), range), gen)?)),
            Tag::Header(header) => Ok(Header(D::Header::new(cfg, WithRange(header, range), gen)?)),
            Tag::BlockQuote => Ok(BlockQuote(D::BlockQuote::new(cfg, WithRange((), range), gen)?)),
            Tag::CodeBlock(cb) => Ok(CodeBlock(D::CodeBlock::new(cfg, WithRange(cb, range), gen)?)),
            Tag::List => Ok(List(D::List::new(cfg, WithRange((), range), gen)?)),
            Tag::Enumerate(enumerate) => Ok(Enumerate(D::Enumerate::new(cfg, WithRange(enumerate, range), gen)?)),
            Tag::Item => Ok(Item(D::Item::new(cfg, WithRange((), range), gen)?)),
            Tag::FootnoteDefinition(fnote) => Ok(FootnoteDefinition(D::FootnoteDefinition::new(cfg, WithRange(fnote, range), gen)?)),
            Tag::Url(url) => Ok(Url(D::UrlWithContent::new(cfg, WithRange(url, range), gen)?)),
            Tag::InterLink(interlink) => Ok(InterLink(D::InterLinkWithContent::new(cfg, WithRange(interlink, range), gen)?)),
            Tag::HtmlBlock => Ok(HtmlBlock(D::HtmlBlock::new(cfg, WithRange((), range), gen)?)),
            Tag::Figure(figure) => Ok(Figure(D::Figure::new(cfg, WithRange(figure, range), gen)?)),
            Tag::TableFigure(figure) => Ok(TableFigure(D::TableFigure::new(cfg, WithRange(figure, range), gen)?)),
            Tag::Table(table) => Ok(Table(D::Table::new(cfg, WithRange(table, range), gen)?)),
            Tag::TableHead => Ok(TableHead(D::TableHead::new(cfg, WithRange((), range), gen)?)),
            Tag::TableRow => Ok(TableRow(D::TableRow::new(cfg, WithRange((), range), gen)?)),
            Tag::TableCell => Ok(TableCell(D::TableCell::new(cfg, WithRange((), range), gen)?)),
            Tag::InlineEmphasis => Ok(InlineEmphasis(D::InlineEmphasis::new(cfg, WithRange((), range), gen)?)),
            Tag::InlineStrong => Ok(InlineStrong(D::InlineStrong::new(cfg, WithRange((), range), gen)?)),
            Tag::InlineStrikethrough => Ok(InlineStrikethrough(D::InlineStrikethrough::new(cfg, WithRange((), range), gen)?)),
            Tag::InlineCode => Ok(InlineCode(D::InlineCode::new(cfg, WithRange((), range), gen)?)),
            Tag::InlineMath => Ok(InlineMath(D::InlineMath::new(cfg, WithRange((), range), gen)?)),
            Tag::Equation(equation) => Ok(Equation(D::Equation::new(cfg, WithRange(equation, range), gen)?)),
            Tag::NumberedEquation(equation) => Ok(NumberedEquation(D::NumberedEquation::new(cfg, WithRange(equation, range), gen)?)),
            Tag::Graphviz(graphviz) => Ok(Graphviz(D::Graphviz::new(cfg, WithRange(graphviz, range), gen)?)),
        }
    }

    pub fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        match self {
            Paragraph(s) => s.output_redirect(),
            Rule(s) => s.output_redirect(),
            Header(s) => s.output_redirect(),
            BlockQuote(s) => s.output_redirect(),
            CodeBlock(s) => s.output_redirect(),
            List(s) => s.output_redirect(),
            Enumerate(s) => s.output_redirect(),
            Item(s) => s.output_redirect(),
            FootnoteDefinition(s) => s.output_redirect(),
            Url(s) => s.output_redirect(),
            InterLink(s) => s.output_redirect(),
            HtmlBlock(s) => s.output_redirect(),
            Figure(s) => s.output_redirect(),
            TableFigure(s) => s.output_redirect(),
            Table(s) => s.output_redirect(),
            TableHead(s) => s.output_redirect(),
            TableRow(s) => s.output_redirect(),
            TableCell(s) => s.output_redirect(),
            InlineEmphasis(s) => s.output_redirect(),
            InlineStrong(s) => s.output_redirect(),
            InlineStrikethrough(s) => s.output_redirect(),
            InlineCode(s) => s.output_redirect(),
            InlineMath(s) => s.output_redirect(),
            Equation(s) => s.output_redirect(),
            NumberedEquation(s) => s.output_redirect(),
            Graphviz(s) => s.output_redirect(),

            Context(..) => None,
        }
    }

    pub fn finish<'b>(self, tag: Tag<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<WithRange<&Event<'a>>>) -> Result<()> {
        match (self, tag) {
            (Paragraph(s), Tag::Paragraph) => s.finish(gen, peek),
            (Rule(s), Tag::Rule) => s.finish(gen, peek),
            (Header(s), Tag::Header(_)) => s.finish(gen, peek),
            (BlockQuote(s), Tag::BlockQuote) => s.finish(gen, peek),
            (CodeBlock(s), Tag::CodeBlock(_)) => s.finish(gen, peek),
            (List(s), Tag::List) => s.finish(gen, peek),
            (Enumerate(s), Tag::Enumerate(_)) => s.finish(gen, peek),
            (Item(s), Tag::Item) => s.finish(gen, peek),
            (FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(gen, peek),
            (Url(s), Tag::Url(_)) => s.finish(gen, peek),
            (InterLink(s), Tag::InterLink(_)) => s.finish(gen, peek),
            (HtmlBlock(s), Tag::HtmlBlock) => s.finish(gen, peek),
            (Figure(s), Tag::Figure(_)) => s.finish(gen, peek),
            (TableFigure(s), Tag::TableFigure(_)) => s.finish(gen, peek),
            (Table(s), Tag::Table(_)) => s.finish(gen, peek),
            (TableHead(s), Tag::TableHead) => s.finish(gen, peek),
            (TableRow(s), Tag::TableRow) => s.finish(gen, peek),
            (TableCell(s), Tag::TableCell) => s.finish(gen, peek),
            (InlineEmphasis(s), Tag::InlineEmphasis) => s.finish(gen, peek),
            (InlineStrong(s), Tag::InlineStrong) => s.finish(gen, peek),
            (InlineStrikethrough(s), Tag::InlineStrikethrough) => s.finish(gen, peek),
            (InlineCode(s), Tag::InlineCode) => s.finish(gen, peek),
            (InlineMath(s), Tag::InlineMath) => s.finish(gen, peek),
            (Equation(s), Tag::Equation(_)) => s.finish(gen, peek),
            (NumberedEquation(s), Tag::NumberedEquation(_)) => s.finish(gen, peek),
            (Graphviz(s), Tag::Graphviz(_)) => s.finish(gen, peek),
            (state, tag) => unreachable!("invalid end tag {:?}, expected {:?}", tag, state),
        }
    }

    // TODO: reomve allows
    #[allow(dead_code)]
    pub fn is_graphviz(&self) -> bool {
        match self {
            Graphviz(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_code_block(&self) -> bool {
        self.is_graphviz() || match self {
            CodeBlock(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_list(&self) -> bool {
        match self {
            List(_) => true,
            _ => false,
        }
    }

    pub fn is_enumerate(&self) -> bool {
        match self {
            Enumerate(_) => true,
            _ => false
        }
    }

    pub fn is_equation(&self) -> bool {
        match self {
            Equation(_) => true,
            _ => false,
        }
    }

    pub fn is_numbered_equation(&self) -> bool {
        match self {
            NumberedEquation(_) => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_code(&self) -> bool {
        self.is_code_block() || self.is_inline_code() || self.is_graphviz()
    }

    pub fn is_math(&self) -> bool {
        self.is_equation() || self.is_numbered_equation() || self.is_inline_math()
    }

    #[allow(dead_code)]
    pub fn is_inline(&self) -> bool {
        self.is_inline_emphasis() || self.is_inline_strong() || self.is_inline_code()
            || self.is_inline_math()
    }

    #[allow(dead_code)]
    pub fn is_inline_emphasis(&self) -> bool {
        match self {
            InlineEmphasis(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_inline_strong(&self) -> bool {
        match self {
            InlineStrong(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_inline_code(&self) -> bool {
        match self {
            InlineCode(_) => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn is_inline_math(&self) -> bool {
        match self {
            InlineMath(_) => true,
            _ => false,
        }
    }

    pub fn is_table(&self) -> bool {
        match self {
            Table(_) => true,
            _ => false
        }
    }
}
