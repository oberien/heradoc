use std::io::{Write, Result};
use std::iter::Peekable;
use std::fmt::Debug;

use pulldown_cmark::{Event, Tag};

#[macro_use]
mod macros;
mod peek;
pub mod latex;

use self::peek::Peek;

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
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a>>], out: &mut impl Write) -> Result<Self>;
    // TODO: refactor intercept_event to pass generator to collect Vec<u8> while intercepting instead of collecting into a Vec<Event>
    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>>;
    fn finish(self, gen: &mut Generator<'a, impl Document<'a>>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()>;
}

pub trait Simple: Debug {
    fn gen_text(text: &str, out: &mut impl Write) -> Result<()>;
    fn gen_footnote_reference(fnote: &str, out: &mut impl Write) -> Result<()>;
    fn gen_soft_break(out: &mut impl Write) -> Result<()>;
    fn gen_hard_break(out: &mut impl Write) -> Result<()>;
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
    fn new(tag: Tag<'a>, stack: &[States<'a, D>], out: &mut impl Write) -> Result<Self> {
        match &tag {
            Tag::Paragraph => Ok(States::Paragraph(D::Paragraph::new(tag, stack, out)?)),
            Tag::Rule => Ok(States::Rule(D::Rule::new(tag, stack, out)?)),
            Tag::Header(_) => Ok(States::Header(D::Header::new(tag, stack, out)?)),
            Tag::BlockQuote => Ok(States::BlockQuote(D::BlockQuote::new(tag, stack, out)?)),
            Tag::CodeBlock(_) => Ok(States::CodeBlock(D::CodeBlock::new(tag, stack, out)?)),
            Tag::List(_) => Ok(States::List(D::List::new(tag, stack, out)?)),
            Tag::Item => Ok(States::Item(D::Item::new(tag, stack, out)?)),
            Tag::FootnoteDefinition(_) => Ok(States::FootnoteDefinition(D::FootnoteDefinition::new(tag, stack, out)?)),
            Tag::Table(_) => Ok(States::Table(D::Table::new(tag, stack, out)?)),
            Tag::TableHead => Ok(States::TableHead(D::TableHead::new(tag, stack, out)?)),
            Tag::TableRow => Ok(States::TableRow(D::TableRow::new(tag, stack, out)?)),
            Tag::TableCell => Ok(States::TableCell(D::TableCell::new(tag, stack, out)?)),
            Tag::Emphasis => Ok(States::InlineEmphasis(D::InlineEmphasis::new(tag, stack, out)?)),
            Tag::Strong => Ok(States::InlineStrong(D::InlineStrong::new(tag, stack, out)?)),
            Tag::Code => Ok(States::InlineCode(D::InlineCode::new(tag, stack, out)?)),
            Tag::Link(..) => Ok(States::Link(D::Link::new(tag, stack, out)?)),
            Tag::Image(..) => Ok(States::Image(D::Image::new(tag, stack, out)?)),
        }
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        match self {
            States::Paragraph(s) => s.intercept_event(e, out),
            States::Rule(s) => s.intercept_event(e, out),
            States::Header(s) => s.intercept_event(e, out),
            States::BlockQuote(s) => s.intercept_event(e, out),
            States::CodeBlock(s) => s.intercept_event(e, out),
            States::List(s) => s.intercept_event(e, out),
            States::Item(s) => s.intercept_event(e, out),
            States::FootnoteDefinition(s) => s.intercept_event(e, out),
            States::Table(s) => s.intercept_event(e, out),
            States::TableHead(s) => s.intercept_event(e, out),
            States::TableRow(s) => s.intercept_event(e, out),
            States::TableCell(s) => s.intercept_event(e, out),
            States::InlineEmphasis(s) => s.intercept_event(e, out),
            States::InlineStrong(s) => s.intercept_event(e, out),
            States::InlineCode(s) => s.intercept_event(e, out),
            States::Link(s) => s.intercept_event(e, out),
            States::Image(s) => s.intercept_event(e, out),
        }
    }

    fn finish(self, tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        match (self, tag) {
            (States::Paragraph(s), Tag::Paragraph) => s.finish(gen, peek, out),
            (States::Rule(s), Tag::Rule) => s.finish(gen, peek, out),
            (States::Header(s), Tag::Header(_)) => s.finish(gen, peek, out),
            (States::BlockQuote(s), Tag::BlockQuote) => s.finish(gen, peek, out),
            (States::CodeBlock(s), Tag::CodeBlock(_)) => s.finish(gen, peek, out),
            (States::List(s), Tag::List(_)) => s.finish(gen, peek, out),
            (States::Item(s), Tag::Item) => s.finish(gen, peek, out),
            (States::FootnoteDefinition(s), Tag::FootnoteDefinition(_)) => s.finish(gen, peek, out),
            (States::Table(s), Tag::Table(_)) => s.finish(gen, peek, out),
            (States::TableHead(s), Tag::TableHead) => s.finish(gen, peek, out),
            (States::TableRow(s), Tag::TableRow) => s.finish(gen, peek, out),
            (States::TableCell(s), Tag::TableCell) => s.finish(gen, peek, out),
            (States::InlineEmphasis(s), Tag::Emphasis) => s.finish(gen, peek, out),
            (States::InlineStrong(s), Tag::Strong) => s.finish(gen, peek, out),
            (States::InlineCode(s), Tag::Code) => s.finish(gen, peek, out),
            (States::Link(s), Tag::Link(..)) => s.finish(gen, peek, out),
            (States::Image(s), Tag::Image(..)) => s.finish(gen, peek, out),
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
        let mut events = events.into_iter().peekable();

        while let Some(event) = events.next() {
            self.visit_event(event, events.peek(), out)?;
        }
        Ok(())
    }

    fn visit_event(&mut self, event: Event<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, self, peek, out)?;
            return Ok(());
        }

        let event = match self.stack.last_mut() {
            Some(state) => state.intercept_event(event, out)?,
            None => Some(event),
        };

        match event {
            None => (),
            Some(Event::End(_)) => unreachable!(),
            Some(Event::Start(tag)) => {
                let state = States::new(tag, &self.stack, out)?;
                self.stack.push(state);
            },
            Some(Event::Text(text)) => D::Simple::gen_text(&text, out)?,
            Some(Event::Html(html)) => unimplemented!(),
            Some(Event::InlineHtml(html)) => unimplemented!(),
            Some(Event::FootnoteReference(fnote)) => D::Simple::gen_footnote_reference(&fnote, out)?,
            Some(Event::SoftBreak) => D::Simple::gen_soft_break(out)?,
            Some(Event::HardBreak) => D::Simple::gen_hard_break(out)?,
        }

        Ok(())
    }
}

fn read_until<'a>(gen: &mut Generator<'a, impl Document<'a>>, events: impl IntoIterator<Item = Event<'a>>, peek: Option<&Event<'a>>) -> Result<String> {
    let mut events = events.into_iter().peekable();
    let mut res = Vec::new();
    while let Some(event) = events.next() {
        let peek = events.peek().or(peek);
        gen.visit_event(event, peek, &mut res)?;
    }
    Ok(String::from_utf8(res).expect("invalid UTF8"))
}
