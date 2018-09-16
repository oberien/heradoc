use std::io::{Write, Result, Read};
use std::iter::Peekable;
use std::fmt::Debug;

use pulldown_cmark::{Event, Tag, Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use typed_arena::Arena;

pub mod latex;
mod states;
mod generator;
mod concat;

pub use self::states::States;
pub use self::generator::Generator;

use self::concat::Concat;
use crate::config::Config;

pub fn generate<'a>(cfg: &'a Config, mut doc: impl Document<'a>, arena: &'a Arena<String>, markdown: String, mut out: impl Write) -> Result<()> {
    let mut gen = Generator::new(cfg, doc, out, arena);
    let events = gen.get_events(markdown);
    gen.generate(events)?;
    Ok(())
}

pub trait Document<'a>: Debug {
    type Simple: Simple;
    type Paragraph: State<'a>;
    type Rule: State<'a>;
    type Header: State<'a>;
    type BlockQuote: State<'a>;
    type CodeBlock: State<'a>;
    type List: List<'a>;
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
    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()>;
    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()>;
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

pub trait List<'a>: State<'a> {
    fn is_enumerate(&self) -> bool;
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
