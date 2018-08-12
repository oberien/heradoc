use std::io::{Write, Result};

use pulldown_cmark::Event;

#[macro_use]
mod macros;
mod peek;
mod article;
mod primitives;

use self::peek::Peek;
use self::article::Article;

pub trait Generator<'a> {
    fn gen(&mut self, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()>;
    fn visit_event(&mut self, event: Event<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()>;
}

pub fn generate<'a>(events: impl IntoIterator<Item = Event<'a>>, mut out: impl Write) -> Result<()> {
    Article::new().gen(&mut State {
        events: &mut events.into_iter().peekable(),
        out: &mut out,
        stack: Vec::new(),
    })
}

pub enum Container {
    Paragraph,
    Header,
    BlockQuote,
    CodeBlock,
    List,
    FootnoteDefinition,
    Table,
    InlineEmphasis,
    InlineStrong,
    InlineCode,
    Link,
    Image,
}

pub struct State<'a, P: Peek<Item = Event<'a>>, W: Write> {
    events: P,
    out: W,
    stack: Vec<Container>,
}

