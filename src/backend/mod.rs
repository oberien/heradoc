use std::io::{Write, Result};
use std::fmt::Debug;
use std::borrow::Cow;

use typed_arena::Arena;

use crate::frontend::{Event, Header, CodeBlock, Enumerate, FootnoteDefinition, FootnoteReference, Table, Image, Graphviz, Link};
use crate::generator::{Generator, PrimitiveGenerator, Stack};

pub mod latex;
mod code_gen_units;

pub use self::code_gen_units::CodeGenUnits;

use crate::config::Config;

pub fn generate<'a>(cfg: &'a Config, doc: impl Backend<'a>, arena: &'a Arena<String>, markdown: String, out: impl Write) -> Result<()> {
    let mut gen = Generator::new(cfg, doc, out, arena);
    let events = gen.get_events(markdown);
    gen.generate(events)?;
    Ok(())
}

pub trait Backend<'a>: Debug {
    type Text: SimpleCodeGenUnit<Cow<'a, str>>;
    type FootnoteReference: SimpleCodeGenUnit<FootnoteReference<'a>>;
    type Link: SimpleCodeGenUnit<Link<'a>>;
    type SoftBreak: SimpleCodeGenUnit<()>;
    type HardBreak: SimpleCodeGenUnit<()>;

    type Paragraph: CodeGenUnit<'a, ()>;
    type Rule: CodeGenUnit<'a, ()>;
    type Header: CodeGenUnit<'a, Header>;
    type BlockQuote: CodeGenUnit<'a, ()>;
    type CodeBlock: CodeGenUnit<'a, CodeBlock<'a>>;
    type List: CodeGenUnit<'a, ()>;
    type Enumerate: CodeGenUnit<'a, Enumerate>;
    type Item: CodeGenUnit<'a, ()>;
    type FootnoteDefinition: CodeGenUnit<'a, FootnoteDefinition<'a>>;

    type Table: CodeGenUnit<'a, Table>;
    type TableHead: CodeGenUnit<'a, ()>;
    type TableRow: CodeGenUnit<'a, ()>;
    type TableCell: CodeGenUnit<'a, ()>;

    type InlineEmphasis: CodeGenUnit<'a, ()>;
    type InlineStrong: CodeGenUnit<'a, ()>;
    type InlineCode: CodeGenUnit<'a, ()>;
    type InlineMath: CodeGenUnit<'a, ()>;

    type Image: CodeGenUnit<'a, Image<'a>>;

    type Equation: CodeGenUnit<'a, ()>;
    type NumberedEquation: CodeGenUnit<'a, ()>;
    type Graphviz: CodeGenUnit<'a, Graphviz<'a>>;

    fn new() -> Self;
    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()>;
    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()>;
}

pub trait CodeGenUnit<'a, T>: Sized + Debug {
    fn new(cfg: &'a Config, tag: T, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self>;
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        None
    }
    fn intercept_event<'b>(&mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }
    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()>;
}

pub trait SimpleCodeGenUnit<T> {
    fn gen(data: T, out: &mut impl Write) -> Result<()>;
}

