use std::io::{Write, Result, Read};
use std::iter::Peekable;
use std::fmt::Debug;

use pulldown_cmark::{Event, Tag, Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};

pub mod latex;
mod states;
mod generator;
mod concat;

pub use self::states::States;
pub use self::generator::Generator;

use self::concat::Concat;

pub fn generate<'a>(mut doc: impl Document<'a>, events: impl IntoIterator<Item = Event<'a>>, mut out: impl Write) -> Result<()> {
    Generator::new(doc, out).generate(events)?;
    Ok(())
}

pub fn get_parser<'a>(buf: &'a mut String, mut src: impl Read) -> impl Iterator<Item = Event<'a>> {
    src.read_to_string(buf).expect("can't read input");

    let parser = Parser::new_with_broken_link_callback(
        buf,
        OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
        Some(&refsolve)
    );
    Concat(parser.peekable())
}

fn refsolve(a: &str, b: &str) -> Option<(String, String)> {
    println!("Unk: {:?} {:?}", a, b);
    if a.starts_with('@') {
        Some(("biblatex-link-dst".to_string(), "title".to_string()))
    } else {
        Some((a.to_string(), b.to_string()))
    }
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
