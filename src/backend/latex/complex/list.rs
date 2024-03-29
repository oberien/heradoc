use std::io::Write;
use diagnostic::Spanned;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::generator::event::{Enumerate, Event};
use crate::generator::Generator;

#[derive(Debug)]
pub struct ListGen;

impl<'a> CodeGenUnit<'a, ()> for ListGen {
    fn new(
        cfg: &'a Config, _: Spanned<()>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        writeln!(gen.get_out(), "\\begin{{itemize}}")?;
        if cfg.tightlist {
            writeln!(gen.get_out(), "\\setlength{{\\itemsep}}{{0pt}}\\setlength{{\\parskip}}{{0pt}}")?;
        }
        Ok(ListGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{itemize}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct EnumerateGen;

impl<'a> CodeGenUnit<'a, Enumerate> for EnumerateGen {
    fn new(
        _cfg: &'a Config, enumerate: Spanned<Enumerate>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: Enumerate { start_number }, .. } = enumerate;
        assert!(std::mem::size_of::<usize>() >= 4);
        assert!(start_number < i32::MAX as u64);
        let start = start_number as i32 - 1;
        let enumerate_depth = 1 + gen.iter_stack().filter(|state| state.is_enumerate()).count();
        writeln!(gen.get_out(), "\\begin{{enumerate}}")?;
        writeln!(
            gen.get_out(),
            "\\setcounter{{enum{}}}{{{}}}",
            "i".repeat(enumerate_depth),
            start
        )?;
        Ok(EnumerateGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{enumerate}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ItemGen;

impl<'a> CodeGenUnit<'a, ()> for ItemGen {
    fn new(
        _cfg: &'a Config, _: Spanned<()>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        write!(gen.get_out(), "\\item ")?;
        Ok(ItemGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out())?;
        Ok(())
    }
}
