use std::io::{Result, Write};
use std::fmt::Debug;

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct InlineEmphasis;

impl<'a> State<'a> for InlineEmphasis {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\emph{{")?;
        Ok(InlineEmphasis)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrong;

impl<'a> State<'a> for InlineStrong {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\textbf{{")?;
        Ok(InlineStrong)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineCode;

impl<'a> State<'a> for InlineCode {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\texttt{{")?;
        Ok(InlineCode)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "}}")?;
        Ok(())
    }
}
