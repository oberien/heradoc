use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;
use std::sync::{Arc, Mutex};

use typed_arena::Arena;
use codespan_reporting::termcolor::StandardStream;

use crate::config::Config;
use crate::error::{FatalResult, Result};
use crate::frontend::range::WithRange;
use crate::diagnostics::Diagnostics;
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
    Svg,
    InterLink,
    Pdf,
    Table,
    TaskListMarker,
    Url,
};
use crate::generator::{Generator, Stack};

pub mod latex;
pub mod ffmpeg;

pub fn generate<'a>(
    cfg: &'a Config, backend: impl Backend<'a>, arena: &'a Arena<String>, markdown: String,
    out: impl Write, stderr: Arc<Mutex<StandardStream>>,
) -> FatalResult<()> {
    let mut gen = Generator::new(cfg, backend, out, arena, stderr);
    gen.generate(markdown)?;
    Ok(())
}

pub fn generate_rust_docs<'a>(
    cfg: &'a Config, backend: impl Backend<'a>, arena: &'a Arena<String>, what_cargo: String,
    out: impl Write, stderr: Arc<Mutex<StandardStream>>,
) -> FatalResult<()> {
    todo!()
}

#[rustfmt::skip]
pub trait Backend<'a>: Sized + Debug {
    // MediumCodeGenUnits are used for leaf-events, which don't contain any further events.
    // StatefulCodeGenUnits are used for tags, which have a start and an end and can contain further events.
    type Text: StatefulCodeGenUnit<'a, Self, Cow<'a, str>>;
    type Latex: StatefulCodeGenUnit<'a, Self, Cow<'a, str>>;
    type FootnoteReference: StatefulCodeGenUnit<'a, Self, FootnoteReference<'a>>;
    type BiberReferences: StatefulCodeGenUnit<'a, Self, Vec<BiberReference<'a>>>;
    type Url: StatefulCodeGenUnit<'a, Self, Url<'a>>;
    type InterLink: StatefulCodeGenUnit<'a, Self, InterLink<'a>>;
    type Image: StatefulCodeGenUnit<'a, Self, Image<'a>>;
    type Svg: StatefulCodeGenUnit<'a, Self, Svg<'a>>;
    type Label: StatefulCodeGenUnit<'a, Self, Cow<'a, str>>;
    type Pdf: StatefulCodeGenUnit<'a, Self, Pdf>;
    type SoftBreak: StatefulCodeGenUnit<'a, Self, ()>;
    type HardBreak: StatefulCodeGenUnit<'a, Self, ()>;
    type PageBreak: StatefulCodeGenUnit<'a, Self, ()>;
    type TaskListMarker: StatefulCodeGenUnit<'a, Self, TaskListMarker>;
    type TableOfContents: StatefulCodeGenUnit<'a, Self, ()>;
    type Bibliography: StatefulCodeGenUnit<'a, Self, ()>;
    type ListOfTables: StatefulCodeGenUnit<'a, Self, ()>;
    type ListOfFigures: StatefulCodeGenUnit<'a, Self, ()>;
    type ListOfListings: StatefulCodeGenUnit<'a, Self, ()>;
    type Appendix: StatefulCodeGenUnit<'a, Self, ()>;

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
    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &Diagnostics<'a>) -> FatalResult<()>;
    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &Diagnostics<'a>) -> FatalResult<()>;
}

/// A [`CodeGenUnit`] is used to generate the code for an event which can contain other events,
/// namely for all tags.
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

/// Similar to [`CodeGenUnit`], but it is specialized for a single [`Backend`] implementation.
/// As such, it can access the backend and store, retrieve and modify data in the backend.
/// This is for example used for latex-beamer, where we need to keep track of the headings,
/// because a new heading can close the frame of an old heading if there is one.
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

/// A [`SimpleCodeGenUnit`] can be used to implement "leaf-events", events which don't contain any further
/// events. It is context free and gets the struct and the out-writer.
pub trait SimpleCodeGenUnit<T>: Debug + Default {
    fn gen(data: WithRange<T>, out: &mut impl Write) -> Result<()>;
}

/// Similar to a [`SimpleCodeGenUnit`], but a [`MediumCodeGenUnit`] gets context information by
/// being passed the stack. The out-writer can be gotten from `stack.get_out()`.
pub trait MediumCodeGenUnit<T>: Debug + Default {
    fn gen<'a, 'b>(
        data: WithRange<T>, config: &Config, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>,
    ) -> Result<()>;
}

// default implementation of Medium… for Simple… such that we can use Medium… everywhere
impl<C: SimpleCodeGenUnit<T>, T> MediumCodeGenUnit<T> for C {
    fn gen<'a, 'b>(
        data: WithRange<T>, _config: &Config, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>,
    ) -> Result<()> {
        C::gen(data, &mut stack.get_out())
    }
}

// default implementation of CodeGenUnit for Medium... such that we can use Stateful everywhere
impl<'a, T, C: MediumCodeGenUnit<T>> CodeGenUnit<'a, T> for C {
    fn new(cfg: &'a Config, data: WithRange<T>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        C::gen(data, cfg, &mut gen.stack())?;
        Ok(C::default())
    }

    fn finish(self, _gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<WithRange<&Event<'a>>>) -> Result<()> {
        Ok(())
    }
}

// default impl Stateful… for CodeGenUnit such that we can use Stateful… everywhere
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

