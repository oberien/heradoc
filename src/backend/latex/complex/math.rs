use std::io::{Result, Write};
use std::borrow::Cow;

use crate::config::Config;
use crate::backend::{latex, Backend, CodeGenUnit};
use crate::generator::PrimitiveGenerator;
use crate::generator::event::{Event, Equation};

#[derive(Debug)]
pub struct InlineMathGen;

impl<'a> CodeGenUnit<'a, ()> for InlineMathGen {
    fn new(_cfg: &Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\begin{{math}}")?;
        Ok(InlineMathGen)
    }

    fn finish(self, gen: &'_ mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "\\end{{math}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EquationGen<'a> {
    label: Option<Cow<'a, str>>,
    caption: Option<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Equation<'a>> for EquationGen<'a> {
    fn new(_cfg: &Config, eq: Equation<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Equation { label, caption } = eq;
        let out = gen.get_out();
        latex::inline_figure_begin(&mut*out, &label, &caption)?;

        writeln!(out, "\\begin{{align*}}")?;

        Ok(EquationGen { label, caption })
    }

    fn finish(self, gen: &'_ mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
        write!(out, "\\end{{align*}}")?;
        latex::inline_figure_end(out, self.label, self.caption)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NumberedEquationGen<'a> {
    label: Option<Cow<'a, str>>,
    caption: Option<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Equation<'a>> for NumberedEquationGen<'a> {
    fn new(_cfg: &Config, eq: Equation<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Equation { label, caption } = eq;
        let out = gen.get_out();
        latex::inline_figure_begin(&mut*out, &label, &caption)?;

        writeln!(out, "\\begin{{align}}")?;
        Ok(NumberedEquationGen { label, caption })
    }

    fn finish(self, gen: &'_ mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
        writeln!(out, "\\end{{align}}")?;
        latex::inline_figure_end(out, self.label, self.caption)?;
        Ok(())
    }
}
