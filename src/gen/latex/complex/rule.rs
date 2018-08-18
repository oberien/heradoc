use std::io::{Result, Write};

use pulldown_cmark::{Event, Tag};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct Rule;

impl<'a> State<'a> for Rule {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(Rule)
    }


    fn intercept_event<'b>(&mut self, e: &Event<'a>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        // TODO: check this
        unreachable!("rule shouldn't have anything between start and end")
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        let out = stack.get_out();
        // TODO: find out why text after the hrule is indented in the pdf
        writeln!(out)?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out, "\\hrule")?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out)?;
        Ok(())
    }
}
