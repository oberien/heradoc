use std::io::{Write, Result};
use std::fmt::Debug;
use std::borrow::Cow;

use typed_arena::Arena;

use crate::parser::{Event, Header, CodeBlock, Enumerate, FootnoteDefinition, FootnoteReference, Table, Link, Graphviz};

pub mod latex;
mod code_gen_units;
mod generator;
mod concat;

pub use self::code_gen_units::CodeGenUnits;
pub use self::generator::Generator;

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

    type Link: CodeGenUnit<'a, Link<'a>>;
    type Image: CodeGenUnit<'a, Link<'a>>;

    type Equation: CodeGenUnit<'a, ()>;
    type NumberedEquation: CodeGenUnit<'a, ()>;
    type Graphviz: CodeGenUnit<'a, Graphviz<'a>>;

    fn new() -> Self;
    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()>;
    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()>;
}

pub trait CodeGenUnit<'a, T>: Sized + Debug {
    fn new(cfg: &'a Config, tag: T, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self>;
    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        None
    }
    fn intercept_event<'b>(&mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }
    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()>;
}

pub trait SimpleCodeGenUnit<T> {
    fn gen(data: T, out: &mut impl Write) -> Result<()>;
}

pub struct Stack<'a: 'b, 'b, D: Backend<'a> + 'b, W: Write + 'b> {
    default_out: &'b mut W,
    stack: &'b mut [CodeGenUnits<'a, D>],
}

impl<'a: 'b, 'b, D: Backend<'a> + 'b, W: Write> Stack<'a, 'b, D, W> {
    fn new(default_out: &'b mut W, stack: &'b mut [CodeGenUnits<'a, D>]) -> Self {
        Stack {
            default_out,
            stack,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &CodeGenUnits<'a, D>> {
        self.stack.iter()
    }

    // TODO
    #[allow(dead_code)]
    pub fn get_out(&mut self) -> &mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(self.default_out)
    }
}
