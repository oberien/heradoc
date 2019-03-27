use std::io::{Result, Write};

use crate::backend::latex::InlineEnvironment;
use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::event::{Equation, Event};
use crate::generator::Generator;

#[derive(Debug)]
pub struct InlineMathGen;

impl<'a> CodeGenUnit<'a, ()> for InlineMathGen {
    fn new(
        _cfg: &Config, _tag: (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        write!(gen.get_out(), "\\begin{{math}}")?;
        Ok(InlineMathGen)
    }

    fn finish(
        self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>,
    ) -> Result<()> {
        write!(gen.get_out(), "\\end{{math}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EquationGen<'a> {
    inline_fig: InlineEnvironment<'a>,
}

impl<'a> CodeGenUnit<'a, Equation<'a>> for EquationGen<'a> {
    fn new(
        _cfg: &Config, eq: Equation<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Equation { label, caption } = eq;
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        let out = gen.get_out();
        inline_fig.write_begin(&mut *out)?;

        writeln!(out, "\\begin{{align*}}")?;

        Ok(EquationGen { inline_fig })
    }

    fn finish(
        self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>,
    ) -> Result<()> {
        let out = gen.get_out();
        write!(out, "\\end{{align*}}")?;
        self.inline_fig.write_end(out)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NumberedEquationGen<'a> {
    inline_fig: InlineEnvironment<'a>,
}

impl<'a> CodeGenUnit<'a, Equation<'a>> for NumberedEquationGen<'a> {
    fn new(
        _cfg: &Config, eq: Equation<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Equation { label, caption } = eq;
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        let out = gen.get_out();
        inline_fig.write_begin(&mut *out)?;

        writeln!(out, "\\begin{{align}}")?;
        Ok(NumberedEquationGen { inline_fig })
    }

    fn finish(
        self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>,
    ) -> Result<()> {
        let out = gen.get_out();
        writeln!(out, "\\end{{align}}")?;
        self.inline_fig.write_end(out)?;
        Ok(())
    }
}
