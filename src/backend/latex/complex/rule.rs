use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::{PrimitiveGenerator, Stack};
use crate::config::Config;
use crate::generator::event::Event;

#[derive(Debug)]
pub struct RuleGen;

impl<'a> CodeGenUnit<'a, ()> for RuleGen {
    fn new(_cfg: &'a Config, _tag: (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(RuleGen)
    }


    fn intercept_event<'b>(&mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, _e: Event<'a>) -> Result<Option<Event<'a>>> {
        // TODO: check this
        unreachable!("rule shouldn't have anything between start and end")
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
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
