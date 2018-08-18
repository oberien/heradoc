use std::io::{Result, Write};

use super::{State, States, Generator};

use pulldown_cmark::{Tag, Event};

#[derive(Debug)]
pub struct Rule;

impl<'a> State<'a> for Rule {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        Ok(Rule)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        // TODO: check this
        unreachable!("rule shouldn't have anything between start and end")
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        // TODO: find out why text after the hrule is indented in the pdf
        writeln!(out)?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out, "\\hrule")?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out)?;
        Ok(())
    }
}
