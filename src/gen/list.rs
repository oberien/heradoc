use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use super::{State, States, Generator};

#[derive(Debug)]
pub struct List {
    start: Option<usize>,
}

impl<'a> State<'a> for List {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        let start = match tag {
            Tag::List(start) => start,
            _ => unreachable!("List::new must be called with Tag::List"),
        };

        if let Some(start) = start {
            let start = start as i32 - 1;
            let enumerate_depth = 1 + stack.iter().filter(|state| state.is_list()).count();
            writeln!(out, "\\begin{{enumerate}}")?;
            writeln!(out, "\\setcounter{{enum{}}}{{{}}}", "i".repeat(enumerate_depth), start)?;
        } else {
            writeln!(out, "\\begin{{itemize}}")?;
        }

        Ok(List {
            start,
        })
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        if self.start.is_some() {
            writeln!(out, "\\end{{enumerate}}")?;
        } else {
            writeln!(out, "\\end{{itemize}}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Item;

impl<'a> State<'a> for Item {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        write!(out, "\\item ")?;
        Ok(Item)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        writeln!(out)?;
        Ok(())
    }
}
