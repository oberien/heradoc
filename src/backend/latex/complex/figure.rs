use std::io::{Result, Write};
use std::borrow::Cow;

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;

use crate::generator::event::{Event, Figure};

#[derive(Debug)]
pub struct FigureGen<'a> {
    label: Option<Cow<'a, str>>,
    caption: Option<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Figure<'a>> for FigureGen<'a> {
    fn new(_cfg: &'a Config, figure: Figure<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Figure { label, caption } = figure;
        write!(gen.get_out(), "\\begin{{figure}}")?;
        Ok(FigureGen { label, caption })
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
        match self.caption {
            Some(caption) => writeln!(out, "\\caption{{{}}}", caption)?,
            None => writeln!(out, "\\caption{{}}")?,
        }
        if let Some(label) = self.label {
            writeln!(out, "\\label{{{}}}", label)?;
        }
        writeln!(out, "\\end{{figure}}")?;
        Ok(())
    }
}
