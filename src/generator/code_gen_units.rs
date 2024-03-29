use std::io::Write;
use diagnostic::Spanned;

use crate::backend::{Backend, StatefulCodeGenUnit};
use crate::config::Config;
use crate::Diagnostics;
use crate::error::Result;
use crate::generator::event::{Event, Tag};
use crate::generator::Generator;
use crate::resolve::Context;

#[derive(Debug)]
pub enum StackElement<'a, B: Backend<'a>> {
    Paragraph(B::Paragraph),
    Header(B::Header),
    BlockQuote(B::BlockQuote),
    CodeBlock(B::CodeBlock),
    List(B::List),
    Enumerate(B::Enumerate),
    Item(B::Item),
    FootnoteDefinition(B::FootnoteDefinition),
    Url(B::UrlWithContent),
    InterLink(B::InterLinkWithContent),
    Figure(B::Figure),
    TableFigure(B::TableFigure),
    Table(B::Table),
    TableHead(B::TableHead),
    TableRow(B::TableRow),
    TableCell(B::TableCell),
    InlineEmphasis(B::InlineEmphasis),
    InlineStrong(B::InlineStrong),
    InlineStrikethrough(B::InlineStrikethrough),
    InlineCode(B::InlineCode),
    InlineMath(B::InlineMath),
    Equation(B::Equation),
    NumberedEquation(B::NumberedEquation),
    Graphviz(B::Graphviz),

    // resolve context
    Context(Context, &'a Diagnostics),
}

use self::StackElement::*;

#[rustfmt::skip]
impl<'a, B: Backend<'a>> StackElement<'a, B> {
    pub fn new(cfg: &'a Config, tag: Spanned<Tag<'a>>, gen: &mut Generator<'a, B, impl Write>) -> Result<Self> {
        let Spanned { value: tag, span } = tag;
        match tag {
            Tag::Paragraph => Ok(Paragraph(B::Paragraph::new(cfg, Spanned::new((), span), gen)?)),
            Tag::Header(header) => Ok(Header(B::Header::new(cfg, Spanned::new(header, span), gen)?)),
            Tag::BlockQuote => Ok(BlockQuote(B::BlockQuote::new(cfg, Spanned::new((), span), gen)?)),
            Tag::CodeBlock(cb) => Ok(CodeBlock(B::CodeBlock::new(cfg, Spanned::new(cb, span), gen)?)),
            Tag::List => Ok(List(B::List::new(cfg, Spanned::new((), span), gen)?)),
            Tag::Enumerate(enumerate) => Ok(Enumerate(B::Enumerate::new(cfg, Spanned::new(enumerate, span), gen)?)),
            Tag::Item => Ok(Item(B::Item::new(cfg, Spanned::new((), span), gen)?)),
            Tag::FootnoteDefinition(fnote) => Ok(FootnoteDefinition(B::FootnoteDefinition::new(cfg, Spanned::new(fnote, span), gen)?)),
            Tag::Url(url) => Ok(Url(B::UrlWithContent::new(cfg, Spanned::new(url, span), gen)?)),
            Tag::InterLink(interlink) => Ok(InterLink(B::InterLinkWithContent::new(cfg, Spanned::new(interlink, span), gen)?)),
            Tag::Figure(figure) => Ok(Figure(B::Figure::new(cfg, Spanned::new(figure, span), gen)?)),
            Tag::TableFigure(figure) => Ok(TableFigure(B::TableFigure::new(cfg, Spanned::new(figure, span), gen)?)),
            Tag::Table(table) => Ok(Table(B::Table::new(cfg, Spanned::new(table, span), gen)?)),
            Tag::TableHead => Ok(TableHead(B::TableHead::new(cfg, Spanned::new((), span), gen)?)),
            Tag::TableRow => Ok(TableRow(B::TableRow::new(cfg, Spanned::new((), span), gen)?)),
            Tag::TableCell => Ok(TableCell(B::TableCell::new(cfg, Spanned::new((), span), gen)?)),
            Tag::InlineEmphasis => Ok(InlineEmphasis(B::InlineEmphasis::new(cfg, Spanned::new((), span), gen)?)),
            Tag::InlineStrong => Ok(InlineStrong(B::InlineStrong::new(cfg, Spanned::new((), span), gen)?)),
            Tag::InlineStrikethrough => Ok(InlineStrikethrough(B::InlineStrikethrough::new(cfg, Spanned::new((), span), gen)?)),
            Tag::InlineCode => Ok(InlineCode(B::InlineCode::new(cfg, Spanned::new((), span), gen)?)),
            Tag::InlineMath => Ok(InlineMath(B::InlineMath::new(cfg, Spanned::new((), span), gen)?)),
            Tag::Equation(equation) => Ok(Equation(B::Equation::new(cfg, Spanned::new(equation, span), gen)?)),
            Tag::NumberedEquation(equation) => Ok(NumberedEquation(B::NumberedEquation::new(cfg, Spanned::new(equation, span), gen)?)),
            Tag::Graphviz(graphviz) => Ok(Graphviz(B::Graphviz::new(cfg, Spanned::new(graphviz, span), gen)?)),
        }
    }

    pub fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        match self {
            Paragraph(s) => s.output_redirect(),
            Header(s) => s.output_redirect(),
            BlockQuote(s) => s.output_redirect(),
            CodeBlock(s) => s.output_redirect(),
            List(s) => s.output_redirect(),
            Enumerate(s) => s.output_redirect(),
            Item(s) => s.output_redirect(),
            FootnoteDefinition(s) => s.output_redirect(),
            Url(s) => s.output_redirect(),
            InterLink(s) => s.output_redirect(),
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

    pub fn finish<'b>(self, tag: Tag<'a>, gen: &mut Generator<'a, B, impl Write>, peek: Option<Spanned<&Event<'a>>>) -> Result<()> {
        match (self, tag) {
            (Paragraph(s), Tag::Paragraph) => s.finish(gen, peek),
            (Header(s), Tag::Header(_)) => s.finish(gen, peek),
            (BlockQuote(s), Tag::BlockQuote) => s.finish(gen, peek),
            (CodeBlock(s), Tag::CodeBlock(_)) => s.finish(gen, peek),
            (List(s), Tag::List) => s.finish(gen, peek),
            (Enumerate(s), Tag::Enumerate(_)) => s.finish(gen, peek),
            (Item(s), Tag::Item) => s.finish(gen, peek),
            (FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(gen, peek),
            (Url(s), Tag::Url(_)) => s.finish(gen, peek),
            (InterLink(s), Tag::InterLink(_)) => s.finish(gen, peek),
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
