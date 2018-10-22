use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;
use crate::generator::event::{Event, Enumerate};

#[derive(Debug)]
pub struct ListGen;

impl<'a> CodeGenUnit<'a, ()> for ListGen {
    fn new(_cfg: &'a Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        writeln!(gen.get_out(), "\\begin{{itemize}}")?;
        Ok(ListGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{itemize}}")
    }
}

#[derive(Debug)]
pub struct EnumerateGen;

impl<'a> CodeGenUnit<'a, Enumerate> for EnumerateGen {
    fn new(_cfg: &'a Config, enumerate: Enumerate, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let start = enumerate.start_number as i32 - 1;
        let enumerate_depth = 1 + gen.iter_stack().filter(|state| state.is_enumerate()).count();
        writeln!(gen.get_out(), "\\begin{{enumerate}}")?;
        writeln!(gen.get_out(), "\\setcounter{{enum{}}}{{{}}}", "i".repeat(enumerate_depth), start)?;
        Ok(EnumerateGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{enumerate}}")
    }
}

#[derive(Debug)]
pub struct ItemGen;

impl<'a> CodeGenUnit<'a, ()> for ItemGen {
    fn new(_cfg: &'a Config, _tag: (), gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\item ")?;
        Ok(ItemGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out())?;
        Ok(())
    }
}
