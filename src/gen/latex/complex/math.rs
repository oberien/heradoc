use std::io::{Result, Write};

use crate::config::Config;
use crate::gen::{Generator, Backend, CodeGenUnit};
use crate::parser::Event;

#[derive(Debug)]
pub struct InlineMathGen;

impl<'a> CodeGenUnit<'a, ()> for InlineMathGen {
    fn new(cfg: &Config, tag: (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\begin{{math}}")?;
        Ok(InlineMathGen)
    }

    fn finish(self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "\\end{{math}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EquationGen;

impl<'a> CodeGenUnit<'a, ()> for EquationGen {
    fn new(cfg: &Config, tag: (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        writeln!(gen.get_out(), "\\begin{{align*}}")?;
        Ok(EquationGen)
    }

    fn finish(self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "\\end{{align*}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NumberedEquationGen;

impl<'a> CodeGenUnit<'a, ()> for NumberedEquationGen {
    fn new(cfg: &Config, tag: (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\begin{{align}}")?;
        Ok(NumberedEquationGen)
    }

    fn finish(self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "\\end{{align}}")?;
        Ok(())
    }
}
