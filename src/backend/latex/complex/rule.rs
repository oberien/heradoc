use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::Event;
use crate::generator::{Generator, Stack};

#[derive(Debug)]
pub struct RuleGen;

impl<'a> CodeGenUnit<'a, ()> for RuleGen {
    fn new(
        _cfg: &'a Config, _: WithRange<()>,
        _gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        Ok(RuleGen)
    }

    fn intercept_event<'b>(
        &mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, _e: Event<'a>,
    ) -> Result<Option<Event<'a>>> {
        // TODO: check this
        unreachable!("rule shouldn't have anything between start and end")
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let out = gen.get_out();
        writeln!(out)?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out, "\\hrule")?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out)?;
        Ok(())
    }
}
