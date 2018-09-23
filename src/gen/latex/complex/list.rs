use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{self, State, States, Generator, Document};
use crate::config::Config;

#[derive(Debug)]
pub struct List {
    start: Option<usize>,
}

impl<'a> State<'a> for List {
    fn new(cfg: &'a Config, tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        let start = match tag {
            Tag::List(start) => start,
            _ => unreachable!("List::new must be called with Tag::List"),
        };

        if let Some(start) = start {
            let start = start as i32 - 1;
            let enumerate_depth = 1 + gen.iter_stack().filter(|state| state.is_enumerate_list()).count();
            writeln!(gen.get_out(), "\\begin{{enumerate}}")?;
            writeln!(gen.get_out(), "\\setcounter{{enum{}}}{{{}}}", "i".repeat(enumerate_depth), start)?;
        } else {
            writeln!(gen.get_out(), "\\begin{{itemize}}")?;
        }

        Ok(List {
            start,
        })
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        if self.start.is_some() {
            writeln!(gen.get_out(), "\\end{{enumerate}}")?;
        } else {
            writeln!(gen.get_out(), "\\end{{itemize}}")?;
        }
        Ok(())
    }
}

impl<'a> gen::List<'a> for List {
    fn is_enumerate(&self) -> bool {
        self.start.is_some()
    }
}

#[derive(Debug)]
pub struct Item;

impl<'a> State<'a> for Item {
    fn new(cfg: &'a Config, tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\item ")?;
        Ok(Item)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out())?;
        Ok(())
    }
}
