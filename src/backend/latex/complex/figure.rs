use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Range;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::Generator;
use crate::generator::event::{Event, Figure};
use crate::error::Result;

#[derive(Debug)]
#[doc(hidden)]
pub struct Fig;
#[doc(hidden)]
pub trait Environment {
    fn to_str() -> &'static str;
}
impl Environment for Fig {
    fn to_str() -> &'static str {
        "figure"
    }
}
#[derive(Debug)]
#[doc(hidden)]
pub struct Table;
impl Environment for Table {
    fn to_str() -> &'static str {
        "table"
    }
}

pub type FigureGen<'a> = AnyFigureGen<'a, Fig>;
pub type TableFigureGen<'a> = AnyFigureGen<'a, Table>;

#[derive(Debug)]
#[doc(hidden)]
pub struct AnyFigureGen<'a, T: Environment> {
    label: Option<Cow<'a, str>>,
    caption: Option<Cow<'a, str>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Environment + Debug> CodeGenUnit<'a, Figure<'a>> for AnyFigureGen<'a, T> {
    fn new(
        _cfg: &'a Config, figure: Figure<'a>, _range: Range<usize>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Figure { label, caption } = figure;
        write!(gen.get_out(), "\\begin{{{}}}", T::to_str())?;
        Ok(AnyFigureGen { label, caption, _marker: PhantomData })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<(&Event<'a>, Range<usize>)>,
    ) -> Result<()> {
        let out = gen.get_out();
        match self.caption {
            Some(caption) => writeln!(out, "\\caption{{{}}}", caption)?,
            None => writeln!(out, "\\caption{{}}")?,
        }
        if let Some(label) = self.label {
            writeln!(out, "\\label{{{}}}", label)?;
        }
        writeln!(out, "\\end{{{}}}", T::to_str())?;
        Ok(())
    }
}
