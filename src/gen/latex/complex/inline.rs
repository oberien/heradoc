use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct InlineEmphasis;

impl<'a> State<'a> for InlineEmphasis {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\emph{{")?;
        Ok(InlineEmphasis)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrong;

impl<'a> State<'a> for InlineStrong {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\textbf{{")?;
        Ok(InlineStrong)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineCode;

impl<'a> State<'a> for InlineCode {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\texttt{{")?;
        Ok(InlineCode)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}
