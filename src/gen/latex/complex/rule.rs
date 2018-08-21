use std::io::{Result, Write};

use pulldown_cmark::{Event, Tag};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct Rule;

impl<'a> State<'a> for Rule {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(Rule)
    }


    fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Document<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        // TODO: check this
        unreachable!("rule shouldn't have anything between start and end")
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
        // TODO: find out why text after the hrule is indented in the pdf
        writeln!(out)?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out, "\\hrule")?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out)?;
        Ok(())
    }
}
