use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;
use crate::generator::event::Event;

#[derive(Debug)]
pub struct InlineEmphasisGen;

impl<'a> CodeGenUnit<'a, ()> for InlineEmphasisGen {
    fn new(_cfg: &'a Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\emph{{")?;
        Ok(InlineEmphasisGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrongGen;

impl<'a> CodeGenUnit<'a, ()> for InlineStrongGen {
    fn new(_cfg: &'a Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\textbf{{")?;
        Ok(InlineStrongGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrikethroughGen;

impl<'a> CodeGenUnit<'a, ()> for InlineStrikethroughGen {
    fn new(_cfg: &'a Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\sout{{")?;
        Ok(InlineStrikethroughGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineCodeGen;

impl<'a> CodeGenUnit<'a, ()> for InlineCodeGen {
    fn new(_cfg: &'a Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\texttt{{")?;
        Ok(InlineCodeGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}
