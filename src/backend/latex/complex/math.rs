use std::io::{Result, Write};

use crate::config::Config;
use crate::backend::{Backend, CodeGenUnit};
use crate::generator::PrimitiveGenerator;
use crate::generator::event::Event;

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
pub struct EquationGen;

impl<'a> CodeGenUnit<'a, ()> for EquationGen {
    fn new(_cfg: &Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        writeln!(gen.get_out(), "\\begin{{align*}}")?;
        Ok(EquationGen)
    }

    fn finish(self, gen: &'_ mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "\\end{{align*}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NumberedEquationGen;

impl<'a> CodeGenUnit<'a, ()> for NumberedEquationGen {
    fn new(_cfg: &Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\begin{{align}}")?;
        Ok(NumberedEquationGen)
    }

    fn finish(self, gen: &'_ mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "\\end{{align}}")?;
        Ok(())
    }
}
