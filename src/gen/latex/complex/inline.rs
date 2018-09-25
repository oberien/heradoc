use std::io::{Result, Write};

use crate::gen::{CodeGenUnit, CodeGenUnits, Generator, Backend};
use crate::config::Config;
use crate::parser::Event;

#[derive(Debug)]
pub struct InlineEmphasisGen;

impl<'a> CodeGenUnit<'a, ()> for InlineEmphasisGen {
    fn new(cfg: &'a Config, (): (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\emph{{")?;
        Ok(InlineEmphasisGen)
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrongGen;

impl<'a> CodeGenUnit<'a, ()> for InlineStrongGen {
    fn new(cfg: &'a Config, (): (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\textbf{{")?;
        Ok(InlineStrongGen)
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineCodeGen;

impl<'a> CodeGenUnit<'a, ()> for InlineCodeGen {
    fn new(cfg: &'a Config, (): (), gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\texttt{{")?;
        Ok(InlineCodeGen)
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}
