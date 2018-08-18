use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use super::{State, States, Generator};

#[derive(Debug)]
pub struct InlineEmphasis;

impl<'a> State<'a> for InlineEmphasis {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\emph{{")?;
        Ok(InlineEmphasis)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrong;

impl<'a> State<'a> for InlineStrong {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\textbf{{")?;
        Ok(InlineStrong)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineCode;

impl<'a> State<'a> for InlineCode {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\texttt{{")?;
        Ok(InlineCode)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "}}")?;
        Ok(())
    }
}
