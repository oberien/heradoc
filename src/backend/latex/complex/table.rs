use std::io::{Result, Write};

use pulldown_cmark::Alignment;

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;
use crate::generator::event::{Event, Tag, Table};

#[derive(Debug)]
pub struct TableGen;

impl<'a> CodeGenUnit<'a, Table<'a>> for TableGen {
    fn new(_cfg: &'a Config, table: Table<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Table { alignment } = table;
        let out = gen.get_out();

        // TODO: in-cell linebreaks
        // TODO: merging columns
        // TODO: merging rows
        // TODO: easier custom formatting
        write!(out, "\\begin{{tabular}}{{|")?;
        for align in alignment {
            match align {
                Alignment::None | Alignment::Left => write!(out, " l |")?,
                Alignment::Center => write!(out, " c |")?,
                Alignment::Right => write!(out, " r |")?,
            }
        }
        writeln!(out, "}}")?;
        writeln!(out, "\\hline")?;
        Ok(TableGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{tabular}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableHeadGen;

impl<'a> CodeGenUnit<'a, ()> for TableHeadGen {
    fn new(_cfg: &'a Config, _tag: (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(TableHeadGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "\\\\ \\thickhline")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableRowGen;

impl<'a> CodeGenUnit<'a, ()> for TableRowGen {
    fn new(_cfg: &'a Config, _tag: (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(TableRowGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "\\\\ \\hline")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableCellGen;

impl<'a> CodeGenUnit<'a, ()> for TableCellGen {
    fn new(_cfg: &'a Config, _tag: (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(TableCellGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        if let Event::Start(Tag::TableCell) = peek.unwrap() {
            write!(gen.get_out(), "&")?;
        }
        Ok(())
    }
}
