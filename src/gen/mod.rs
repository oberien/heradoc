use std::io::{Write, Result};
use std::iter::Peekable;
use std::fmt::Debug;

use pulldown_cmark::{Event, Tag};

pub mod latex;

pub struct Generator<'a, D: Document<'a>> {
    doc: D,
    stack: Vec<States<'a, D>>,
}

pub fn generate<'a>(mut doc: impl Document<'a>, events: impl IntoIterator<Item = Event<'a>>, mut out: impl Write) -> Result<()> {
    Generator::new(doc).generate(events, &mut out)?;
    Ok(())
}

pub trait Document<'a>: Debug {
    type Simple: Simple;
    type Paragraph: State<'a>;
    type Rule: State<'a>;
    type Header: State<'a>;
    type BlockQuote: State<'a>;
    type CodeBlock: State<'a>;
    type List: State<'a>;
    type Item: State<'a>;
    type FootnoteDefinition: State<'a>;
    type Table: State<'a>;
    type TableHead: State<'a>;
    type TableRow: State<'a>;
    type TableCell: State<'a>;
    type InlineEmphasis: State<'a>;
    type InlineStrong: State<'a>;
    type InlineCode: State<'a>;
    type Link: State<'a>;
    type Image: State<'a>;

    fn new() -> Self;
    fn gen_preamble(&mut self, out: &mut impl Write) -> Result<()>;
    fn gen_epilogue(&mut self, out: &mut impl Write) -> Result<()>;
}

pub trait State<'a>: Sized + Debug {
    fn new<'b>(tag: Tag<'a>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self>;
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        None
    }
    fn intercept_event<'b>(&mut self, e: &Event<'a>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        Ok(())
    }
    fn finish<'b>(self, peek: Option<&Event<'a>>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()>;
}

pub trait Simple: Debug {
    fn gen_text(text: &str, out: &mut impl Write) -> Result<()>;
    fn gen_footnote_reference(fnote: &str, out: &mut impl Write) -> Result<()>;
    fn gen_soft_break(out: &mut impl Write) -> Result<()>;
    fn gen_hard_break(out: &mut impl Write) -> Result<()>;
}

pub struct Stack<'a: 'b, 'b, D: Document<'a> + 'b, W: Write> {
    default_out: W,
    stack: &'b mut [States<'a, D>],
}

impl<'a: 'b, 'b, D: Document<'a> + 'b, W: Write> Stack<'a, 'b, D, W> {
    fn new(default_out: W, stack: &'b mut [States<'a, D>]) -> Self {
        Stack {
            default_out,
            stack,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &States<'a, D>> {
        self.stack.iter()
    }

    pub fn get_out(&mut self) -> &mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(&mut self.default_out)
    }
}

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
    fn new<'b>(tag: Tag<'a>, stack: Stack<'a, 'b, D, impl Write>) -> Result<Self> {
        match &tag {
            Tag::Paragraph => Ok(States::Paragraph(D::Paragraph::new(tag, stack)?)),
            Tag::Rule => Ok(States::Rule(D::Rule::new(tag, stack)?)),
            Tag::Header(_) => Ok(States::Header(D::Header::new(tag, stack)?)),
            Tag::BlockQuote => Ok(States::BlockQuote(D::BlockQuote::new(tag, stack)?)),
            Tag::CodeBlock(_) => Ok(States::CodeBlock(D::CodeBlock::new(tag, stack)?)),
            Tag::List(_) => Ok(States::List(D::List::new(tag, stack)?)),
            Tag::Item => Ok(States::Item(D::Item::new(tag, stack)?)),
            Tag::FootnoteDefinition(_) => Ok(States::FootnoteDefinition(D::FootnoteDefinition::new(tag, stack)?)),
            Tag::Table(_) => Ok(States::Table(D::Table::new(tag, stack)?)),
            Tag::TableHead => Ok(States::TableHead(D::TableHead::new(tag, stack)?)),
            Tag::TableRow => Ok(States::TableRow(D::TableRow::new(tag, stack)?)),
            Tag::TableCell => Ok(States::TableCell(D::TableCell::new(tag, stack)?)),
            Tag::Emphasis => Ok(States::InlineEmphasis(D::InlineEmphasis::new(tag, stack)?)),
            Tag::Strong => Ok(States::InlineStrong(D::InlineStrong::new(tag, stack)?)),
            Tag::Code => Ok(States::InlineCode(D::InlineCode::new(tag, stack)?)),
            Tag::Link(..) => Ok(States::Link(D::Link::new(tag, stack)?)),
            Tag::Image(..) => Ok(States::Image(D::Image::new(tag, stack)?)),
        }
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
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

    fn intercept_event<'b>(&mut self, e: &Event<'a>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        match self {
            States::Paragraph(s) => s.intercept_event(e, stack),
            States::Rule(s) => s.intercept_event(e, stack),
            States::Header(s) => s.intercept_event(e, stack),
            States::BlockQuote(s) => s.intercept_event(e, stack),
            States::CodeBlock(s) => s.intercept_event(e, stack),
            States::List(s) => s.intercept_event(e, stack),
            States::Item(s) => s.intercept_event(e, stack),
            States::FootnoteDefinition(s) => s.intercept_event(e, stack),
            States::Table(s) => s.intercept_event(e, stack),
            States::TableHead(s) => s.intercept_event(e, stack),
            States::TableRow(s) => s.intercept_event(e, stack),
            States::TableCell(s) => s.intercept_event(e, stack),
            States::InlineEmphasis(s) => s.intercept_event(e, stack),
            States::InlineStrong(s) => s.intercept_event(e, stack),
            States::InlineCode(s) => s.intercept_event(e, stack),
            States::Link(s) => s.intercept_event(e, stack),
            States::Image(s) => s.intercept_event(e, stack),
        }
    }

    fn finish<'b>(self, tag: Tag<'a>, peek: Option<&Event<'a>>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        match (self, tag) {
            (States::Paragraph(s), Tag::Paragraph) => s.finish(peek, stack),
            (States::Rule(s), Tag::Rule) => s.finish(peek, stack),
            (States::Header(s), Tag::Header(_)) => s.finish(peek, stack),
            (States::BlockQuote(s), Tag::BlockQuote) => s.finish(peek, stack),
            (States::CodeBlock(s), Tag::CodeBlock(_)) => s.finish(peek, stack),
            (States::List(s), Tag::List(_)) => s.finish(peek, stack),
            (States::Item(s), Tag::Item) => s.finish(peek, stack),
            (States::FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(peek, stack),
            (States::Table(s), Tag::Table(_)) => s.finish(peek, stack),
            (States::TableHead(s), Tag::TableHead) => s.finish(peek, stack),
            (States::TableRow(s), Tag::TableRow) => s.finish(peek, stack),
            (States::TableCell(s), Tag::TableCell) => s.finish(peek, stack),
            (States::InlineEmphasis(s), Tag::Emphasis) => s.finish(peek, stack),
            (States::InlineStrong(s), Tag::Strong) => s.finish(peek, stack),
            (States::InlineCode(s), Tag::Code) => s.finish(peek, stack),
            (States::Link(s), Tag::Link(..)) => s.finish(peek, stack),
            (States::Image(s), Tag::Image(..)) => s.finish(peek, stack),
            (state, tag) => unreachable!("invalid end tag {:?}, expected {:?}", tag, state),
        }
    }

    fn is_list(&self) -> bool {
        match self {
            States::List(_) => true,
            _ => false,
        }
    }
}

impl<'a, D: Document<'a>> Generator<'a, D> {
    pub fn new(doc: D) -> Self {
        Generator {
            doc,
            stack: Vec::new(),
        }
    }

    pub fn generate(mut self, events: impl IntoIterator<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
        self.doc.gen_preamble(out)?;
        let mut events = events.into_iter().peekable();

        while let Some(event) = events.next() {
            self.visit_event(event, events.peek(), out)?;
        }
        self.doc.gen_epilogue(out)?;
        Ok(())
    }

    fn visit_event(&mut self, event: Event<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, peek, Stack::new(out, &mut self.stack))?;
            return Ok(());
        }

        if !self.stack.is_empty() {
            let index = self.stack.len() - 1;
            let (stack, last) = self.stack.split_at_mut(index);
            last[0].intercept_event(&event, Stack::new(&mut *out, stack))?;
        }

        match event {
            Event::End(_) => unreachable!(),
            Event::Start(tag) => {
                let state = States::new(tag, Stack::new(&mut *out, &mut self.stack))?;
                self.stack.push(state);
            },
            Event::Text(text) => D::Simple::gen_text(&text, &mut self.get_out(out))?,
            Event::Html(html) => unimplemented!(),
            Event::InlineHtml(html) => unimplemented!(),
            Event::FootnoteReference(fnote) => D::Simple::gen_footnote_reference(&fnote, &mut self.get_out(out))?,
            Event::SoftBreak => D::Simple::gen_soft_break(&mut self.get_out(out))?,
            Event::HardBreak => D::Simple::gen_hard_break(&mut self.get_out(out))?,
        }

        Ok(())
    }

    fn get_out<'s: 'b, 'b>(&'s mut self, out: &'b mut dyn Write) -> &'b mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(out)
    }
}
