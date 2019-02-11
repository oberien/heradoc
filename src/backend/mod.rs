use std::io::{Write, Result};
use std::fmt::Debug;
use std::borrow::Cow;

use typed_arena::Arena;

use crate::generator::event::{Event, Header, CodeBlock, Enumerate, FootnoteDefinition,
    FootnoteReference, Table, Image, Graphviz, Link, Pdf, Equation};
use crate::generator::{Generator, PrimitiveGenerator, Stack};

pub mod latex;

use crate::config::Config;

pub fn generate<'a>(cfg: &'a Config, doc: impl Backend<'a>, arena: &'a Arena<String>, markdown: String, out: impl Write) -> Result<()> {
    let mut gen = Generator::new(cfg, doc, out, arena);
    gen.generate(markdown)?;
    Ok(())
}

pub trait Backend<'a>: Debug {
    type Text: MediumCodeGenUnit<Cow<'a, str>>;
    type FootnoteReference: MediumCodeGenUnit<FootnoteReference<'a>>;
    type Link: MediumCodeGenUnit<Link<'a>>;
    type Image: MediumCodeGenUnit<Image<'a>>;
    type Label: MediumCodeGenUnit<Cow<'a, str>>;
    type Pdf: MediumCodeGenUnit<Pdf>;
    type SoftBreak: MediumCodeGenUnit<()>;
    type HardBreak: MediumCodeGenUnit<()>;
    type TableOfContents: MediumCodeGenUnit<()>;
    type Bibliography: MediumCodeGenUnit<()>;
    type ListOfTables: MediumCodeGenUnit<()>;
    type ListOfFigures: MediumCodeGenUnit<()>;
    type ListOfListings: MediumCodeGenUnit<()>;
    type Appendix: MediumCodeGenUnit<()>;

    type Paragraph: CodeGenUnit<'a, ()>;
    type Rule: CodeGenUnit<'a, ()>;
    type Header: CodeGenUnit<'a, Header<'a>>;
    type BlockQuote: CodeGenUnit<'a, ()>;
    type CodeBlock: CodeGenUnit<'a, CodeBlock<'a>>;
    type List: CodeGenUnit<'a, ()>;
    type Enumerate: CodeGenUnit<'a, Enumerate>;
    type Item: CodeGenUnit<'a, ()>;
    type FootnoteDefinition: CodeGenUnit<'a, FootnoteDefinition<'a>>;

    type Table: CodeGenUnit<'a, Table<'a>>;
    type TableHead: CodeGenUnit<'a, ()>;
    type TableRow: CodeGenUnit<'a, ()>;
    type TableCell: CodeGenUnit<'a, ()>;

    type InlineEmphasis: CodeGenUnit<'a, ()>;
    type InlineStrong: CodeGenUnit<'a, ()>;
    type InlineCode: CodeGenUnit<'a, ()>;
    type InlineMath: CodeGenUnit<'a, ()>;

    type Equation: CodeGenUnit<'a, Equation<'a>>;
    type NumberedEquation: CodeGenUnit<'a, Equation<'a>>;
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

pub trait MediumCodeGenUnit<T> {
    fn gen<'a, 'b>(data: T, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>) -> Result<()>;
}

impl<T: SimpleCodeGenUnit<D>, D> MediumCodeGenUnit<D> for T {
    fn gen<'a, 'b>(data: D, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>) -> Result<()> {
        T::gen(data, &mut stack.get_out())
    }
}