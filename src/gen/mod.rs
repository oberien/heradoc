use std::io::{Write, Result};
use std::iter::Peekable;

use pulldown_cmark::{Event, Tag};

#[macro_use]
mod macros;
mod peek;
mod article;

mod paragraph;
mod rule;
mod header;
mod blockquote;
mod codeblock;
mod list;
mod footnote_definition;
mod table;
mod inline;
mod link;
mod image;

use self::peek::Peek;
pub use self::article::Article;

use self::paragraph::Paragraph;
use self::rule::Rule;
use self::header::Header;
use self::blockquote::BlockQuote;
use self::codeblock::CodeBlock;
use self::list::{List, Item};
use self::footnote_definition::FootnoteDefinition;
use self::table::{Table, TableHead, TableRow, TableCell};
use self::inline::{InlineEmphasis, InlineStrong, InlineCode};
use self::link::Link;
use self::image::Image;

pub struct Generator<'a> {
    stack: Vec<States<'a>>,
}

pub fn generate<'a>(mut doc: impl Document, events: impl IntoIterator<Item = Event<'a>>, mut out: impl Write) -> Result<()> {
    doc.gen_preamble(&mut out)?;
    Generator::new().generate(events, &mut out)?;
    doc.gen_epilogue(&mut out)?;
    Ok(())
}

pub trait Document {
    fn new() -> Self;
    fn gen_preamble(&mut self, out: &mut impl Write) -> Result<()>;
    fn gen_epilogue(&mut self, out: &mut impl Write) -> Result<()>;
}

pub trait State<'a>: Sized {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self>;
    // TODO: refactor intercept_event to pass generator to collect Vec<u8> while intercepting instead of collecting into a Vec<Event>
    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>>;
    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()>;
}

#[derive(Debug)]
pub enum States<'a> {
    Paragraph(Paragraph),
    Rule(Rule),
    Header(Header<'a>),
    BlockQuote(BlockQuote<'a>),
    CodeBlock(CodeBlock),
    List(List),
    Item(Item),
    FootnoteDefinition(FootnoteDefinition),
    Table(Table),
    TableHead(TableHead),
    TableRow(TableRow),
    TableCell(TableCell),
    InlineEmphasis(InlineEmphasis),
    InlineStrong(InlineStrong),
    InlineCode(InlineCode),
    Link(Link<'a>),
    Image(Image<'a>),
}

impl<'a> States<'a> {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        match &tag {
            Tag::Paragraph => Ok(States::Paragraph(Paragraph::new(tag, stack, out)?)),
            Tag::Rule => Ok(States::Rule(Rule::new(tag, stack, out)?)),
            Tag::Header(_) => Ok(States::Header(Header::new(tag, stack, out)?)),
            Tag::BlockQuote => Ok(States::BlockQuote(BlockQuote::new(tag, stack, out)?)),
            Tag::CodeBlock(_) => Ok(States::CodeBlock(CodeBlock::new(tag, stack, out)?)),
            Tag::List(_) => Ok(States::List(List::new(tag, stack, out)?)),
            Tag::Item => Ok(States::Item(Item::new(tag, stack, out)?)),
            Tag::FootnoteDefinition(_) => Ok(States::FootnoteDefinition(FootnoteDefinition::new(tag, stack, out)?)),
            Tag::Table(_) => Ok(States::Table(Table::new(tag, stack, out)?)),
            Tag::TableHead => Ok(States::TableHead(TableHead::new(tag, stack, out)?)),
            Tag::TableRow => Ok(States::TableRow(TableRow::new(tag, stack, out)?)),
            Tag::TableCell => Ok(States::TableCell(TableCell::new(tag, stack, out)?)),
            Tag::Emphasis => Ok(States::InlineEmphasis(InlineEmphasis::new(tag, stack, out)?)),
            Tag::Strong => Ok(States::InlineStrong(InlineStrong::new(tag, stack, out)?)),
            Tag::Code => Ok(States::InlineCode(InlineCode::new(tag, stack, out)?)),
            Tag::Link(..) => Ok(States::Link(Link::new(tag, stack, out)?)),
            Tag::Image(..) => Ok(States::Image(Image::new(tag, stack, out)?)),
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

    fn finish(self, tag: Tag<'a>, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
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

impl<'a> Generator<'a> {
    pub fn new() -> Self {
        Generator {
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
            Some(Event::Text(text)) => gen_text(&text, out)?,
            Some(Event::Html(html)) => unimplemented!(),
            Some(Event::InlineHtml(html)) => unimplemented!(),
            Some(Event::FootnoteReference(fnote)) => gen_footnote_reference(&fnote, out)?,
            Some(Event::SoftBreak) => gen_soft_break(out)?,
            Some(Event::HardBreak) => gen_hard_break(out)?,
        }

        Ok(())
    }
}

fn gen_text(text: &str, out: &mut impl Write) -> Result<()> {
    write!(out, "{}", text)?;
    Ok(())
}

pub fn gen_footnote_reference(fnote: &str, out: &mut impl Write) -> Result<()> {
    write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote)?;
    Ok(())
}

pub fn gen_soft_break(out: &mut impl Write) -> Result<()> {
    // soft breaks are only used to split up text in lines in the source file
    // so it's nothing we should translate, but for better readability keep them
    writeln!(out)?;
    Ok(())
}

pub fn gen_hard_break(out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\par")?;
    Ok(())
}

fn read_until<'a>(gen: &mut Generator<'a>, events: impl IntoIterator<Item = Event<'a>>, peek: Option<&Event<'a>>) -> Result<String> {
    let mut events = events.into_iter().peekable();
    let mut res = Vec::new();
    while let Some(event) = events.next() {
        let peek = events.peek().or(peek);
        gen.visit_event(event, peek, &mut res)?;
    }
    Ok(String::from_utf8(res).expect("invalid UTF8"))
}
