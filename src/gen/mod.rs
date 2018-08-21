use std::io::{Write, Result};
use std::iter::Peekable;
use std::fmt::Debug;

use pulldown_cmark::{Event, Tag};

pub mod latex;

pub fn generate<'a>(mut doc: impl Document<'a>, events: impl IntoIterator<Item = Event<'a>>, mut out: impl Write) -> Result<()> {
    Generator::new(doc, out).generate(events)?;
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
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self>;
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        None
    }
    fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Document<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }
    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()>;
}

pub trait Simple: Debug {
    fn gen_text(text: &str, out: &mut impl Write) -> Result<()>;
    fn gen_footnote_reference(fnote: &str, out: &mut impl Write) -> Result<()>;
    fn gen_soft_break(out: &mut impl Write) -> Result<()>;
    fn gen_hard_break(out: &mut impl Write) -> Result<()>;
}

pub struct Stack<'a: 'b, 'b, D: Document<'a> + 'b, W: Write + 'b> {
    default_out: &'b mut W,
    stack: &'b mut [States<'a, D>],
}

impl<'a: 'b, 'b, D: Document<'a> + 'b, W: Write> Stack<'a, 'b, D, W> {
    fn new(default_out: &'b mut W, stack: &'b mut [States<'a, D>]) -> Self {
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
            .unwrap_or(self.default_out)
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
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, D, impl Write>) -> Result<Self> {
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

    fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Document<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
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

    fn finish<'b>(self, tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
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

    fn is_list(&self) -> bool {
        match self {
            States::List(_) => true,
            _ => false,
        }
    }
}

pub struct Generator<'a, D: Document<'a>, W: Write> {
    doc: D,
    default_out: W,
    stack: Vec<States<'a, D>>,
}

impl<'a, D: Document<'a>, W: Write> Generator<'a, D, W> {
    pub fn new(doc: D, default_out: W) -> Self {
        Generator {
            doc,
            default_out,
            stack: Vec::new(),
        }
    }

    pub fn generate(mut self, events: impl IntoIterator<Item = Event<'a>>) -> Result<()> {
        self.doc.gen_preamble(&mut self.default_out)?;
        let mut events = events.into_iter().peekable();

        while let Some(event) = events.next() {
            self.visit_event(event, events.peek())?;
        }
        self.doc.gen_epilogue(&mut self.default_out)?;
        Ok(())
    }

    pub fn visit_event(&mut self, event: Event<'a>, peek: Option<&Event<'a>>) -> Result<()> {
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, self, peek)?;
            return Ok(());
        }

        let event = if !self.stack.is_empty() {
            let index = self.stack.len() - 1;
            let (stack, last) = self.stack.split_at_mut(index);
            last[0].intercept_event(&mut Stack::new(&mut self.default_out, stack), event)?
        } else {
            Some(event)
        };

        match event {
            None => (),
            Some(Event::End(_)) => unreachable!(),
            Some(Event::Start(tag)) => {
                let state = States::new(tag, self)?;
                self.stack.push(state);
            },
            Some(Event::Text(text)) => D::Simple::gen_text(&text, &mut self.get_out())?,
            Some(Event::Html(html)) => unimplemented!(),
            Some(Event::InlineHtml(html)) => unimplemented!(),
            Some(Event::FootnoteReference(fnote)) => D::Simple::gen_footnote_reference(&fnote, &mut self.get_out())?,
            Some(Event::SoftBreak) => D::Simple::gen_soft_break(&mut self.get_out())?,
            Some(Event::HardBreak) => D::Simple::gen_hard_break(&mut self.get_out())?,
        }

        Ok(())
    }

    pub fn iter_stack(&self) -> impl Iterator<Item = &States<'a, D>> {
        self.stack.iter()
    }

    pub fn get_out<'s: 'b, 'b>(&'s mut self) -> &'b mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(&mut self.default_out)
    }
}
