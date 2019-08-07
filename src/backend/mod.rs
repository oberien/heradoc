use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;
use std::sync::{Arc, Mutex};

use typed_arena::Arena;
use codespan_reporting::termcolor::StandardStream;

use crate::config::Config;
use crate::error::{FatalResult, Result};
use crate::frontend::range::WithRange;
use crate::generator::event::{
    BiberReference,
    CodeBlock,
    Enumerate,
    Equation,
    Event,
    Figure,
    FootnoteDefinition,
    FootnoteReference,
    Graphviz,
    Header,
    Image,
    InterLink,
    Pdf,
    Table,
    TaskListMarker,
    Url,
};
use crate::generator::{Generator, Stack};

pub mod latex;

pub fn generate<'a>(
    cfg: &'a Config, doc: impl Backend<'a>, arena: &'a Arena<String>, markdown: String,
    out: impl Write, stderr: Arc<Mutex<StandardStream>>,
) -> FatalResult<()> {
    let mut gen = Generator::new(cfg, doc, out, arena, stderr);
    gen.generate(markdown)?;
    Ok(())
}

#[rustfmt::skip]
pub trait Backend<'a>: Sized + Debug {
    type Text: MediumCodeGenUnit<Cow<'a, str>>;
    type Latex: MediumCodeGenUnit<Cow<'a, str>>;
    type FootnoteReference: MediumCodeGenUnit<FootnoteReference<'a>>;
    type BiberReferences: MediumCodeGenUnit<Vec<BiberReference<'a>>>;
    type Url: MediumCodeGenUnit<Url<'a>>;
    type InterLink: MediumCodeGenUnit<InterLink<'a>>;
    type Image: MediumCodeGenUnit<Image<'a>>;
    type Label: MediumCodeGenUnit<Cow<'a, str>>;
    type Pdf: MediumCodeGenUnit<Pdf>;
    type SoftBreak: MediumCodeGenUnit<()>;
    type HardBreak: MediumCodeGenUnit<()>;
    type TaskListMarker: MediumCodeGenUnit<TaskListMarker>;
    type TableOfContents: MediumCodeGenUnit<()>;
    type Bibliography: MediumCodeGenUnit<()>;
    type ListOfTables: MediumCodeGenUnit<()>;
    type ListOfFigures: MediumCodeGenUnit<()>;
    type ListOfListings: MediumCodeGenUnit<()>;
    type Appendix: MediumCodeGenUnit<()>;

    type Paragraph: StatefulCodeGenUnit<'a, Self, ()>;
    type Rule: StatefulCodeGenUnit<'a, Self, ()>;
    type Header: StatefulCodeGenUnit<'a, Self, Header<'a>>;
    type BlockQuote: StatefulCodeGenUnit<'a, Self, ()>;
    type CodeBlock: StatefulCodeGenUnit<'a, Self, CodeBlock<'a>>;
    type List: StatefulCodeGenUnit<'a, Self, ()>;
    type Enumerate: StatefulCodeGenUnit<'a, Self, Enumerate>;
    type Item: StatefulCodeGenUnit<'a, Self, ()>;
    type FootnoteDefinition: StatefulCodeGenUnit<'a, Self, FootnoteDefinition<'a>>;
    type UrlWithContent: StatefulCodeGenUnit<'a, Self, Url<'a>>;
    type InterLinkWithContent: StatefulCodeGenUnit<'a, Self, InterLink<'a>>;
    type HtmlBlock: StatefulCodeGenUnit<'a, Self, ()>;
    type Figure: StatefulCodeGenUnit<'a, Self, Figure<'a>>;

    type TableFigure: StatefulCodeGenUnit<'a, Self, Figure<'a>>;
    type Table: StatefulCodeGenUnit<'a, Self, Table<'a>>;
    type TableHead: StatefulCodeGenUnit<'a, Self, ()>;
    type TableRow: StatefulCodeGenUnit<'a, Self, ()>;
    type TableCell: StatefulCodeGenUnit<'a, Self, ()>;

    type InlineEmphasis: StatefulCodeGenUnit<'a, Self, ()>;
    type InlineStrong: StatefulCodeGenUnit<'a, Self, ()>;
    type InlineStrikethrough: StatefulCodeGenUnit<'a, Self, ()>;
    type InlineCode: StatefulCodeGenUnit<'a, Self, ()>;
    type InlineMath: StatefulCodeGenUnit<'a, Self, ()>;

    type Equation: StatefulCodeGenUnit<'a, Self, Equation<'a>>;
    type NumberedEquation: StatefulCodeGenUnit<'a, Self, Equation<'a>>;
    type Graphviz: StatefulCodeGenUnit<'a, Self, Graphviz<'a>>;

    fn new() -> Self;
    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, stderr: Arc<Mutex<StandardStream>>) -> FatalResult<()>;
    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write, stderr: Arc<Mutex<StandardStream>>) -> FatalResult<()>;
}

pub trait CodeGenUnit<'a, T>: Sized + Debug {
    fn new(
        cfg: &'a Config, tag: WithRange<T>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self>;
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        None
    }
    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()>;
}

pub trait StatefulCodeGenUnit<'a, B: Backend<'a>, T>: Sized + Debug {
    fn new(
        cfg: &'a Config, tag: WithRange<T>,
        gen: &mut Generator<'a, B, impl Write>,
    ) -> Result<Self>;
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        None
    }
    fn finish(
        self, gen: &mut Generator<'a, B, impl Write>,
        peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()>;
}

impl<'a, B: Backend<'a>, T, C: CodeGenUnit<'a, T>> StatefulCodeGenUnit<'a, B, T> for C {
    fn new(
        cfg: &'a Config, tag: WithRange<T>,
        gen: &mut Generator<'a, B, impl Write>,
    ) -> Result<Self> {
        C::new(cfg, tag, gen)
    }
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        C::output_redirect(self)
    }
    fn finish(
        self, gen: &mut Generator<'a, B, impl Write>,
        peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        C::finish(self, gen, peek)
    }
}

pub trait SimpleCodeGenUnit<T> {
    fn gen(data: WithRange<T>, out: &mut impl Write) -> Result<()>;
}

pub trait MediumCodeGenUnit<T> {
    fn gen<'a, 'b>(
        data: WithRange<T>, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>,
    ) -> Result<()>;
}

impl<C: SimpleCodeGenUnit<T>, T> MediumCodeGenUnit<T> for C {
    fn gen<'a, 'b>(
        data: WithRange<T>, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>,
    ) -> Result<()> {
        C::gen(data, &mut stack.get_out())
    }
}
