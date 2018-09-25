use std::io::{Result, Write};

use crate::gen::{CodeGenUnit, CodeGenUnits, Generator, Stack, Backend};
use crate::config::Config;
use crate::parser::Event;

#[derive(Debug)]
pub struct RuleGen;

impl<'a> CodeGenUnit<'a, ()> for RuleGen {
    fn new(cfg: &'a Config, (): (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(RuleGen)
    }


    fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        // TODO: check this
        unreachable!("rule shouldn't have anything between start and end")
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
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
