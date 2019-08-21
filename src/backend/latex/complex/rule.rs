use std::io::Write;

use crate::backend::{Backend, CodeGenUnit, StatefulCodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::{WithRange, SourceRange};
use crate::generator::event::Event;
use crate::generator::Generator;
use crate::backend::latex::Beamer;

#[derive(Debug)]
pub struct RuleGen;

impl<'a> CodeGenUnit<'a, ()> for RuleGen {
    fn new(
        _cfg: &'a Config, _: WithRange<()>,
        _gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        Ok(RuleGen)
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

#[derive(Debug)]
pub struct BeamerRuleGen<'a> {
    cfg: &'a Config,
    range: SourceRange,
}

impl<'a> StatefulCodeGenUnit<'a, Beamer, ()> for BeamerRuleGen<'a> {
    fn new(
        cfg: &'a Config, WithRange(_, range): WithRange<()>,
        _gen: &mut Generator<'a, Beamer, impl Write>,
    ) -> Result<Self> {
        Ok(BeamerRuleGen { cfg, range })
    }

    fn finish(
        self, gen: &mut Generator<'a, Beamer, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let BeamerRuleGen { cfg, range } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        backend.close_until(2, &mut out, range, diagnostics)?;
        backend.open_until(2, cfg, &mut out, range, diagnostics)?;
        Ok(())
    }
}
