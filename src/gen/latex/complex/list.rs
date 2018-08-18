use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct List {
    start: Option<usize>,
}

impl<'a> State<'a> for List {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        let start = match tag {
            Tag::List(start) => start,
            _ => unreachable!("List::new must be called with Tag::List"),
        };

        if let Some(start) = start {
            let start = start as i32 - 1;
            let enumerate_depth = 1 + stack.iter().filter(|state| state.is_list()).count();
            writeln!(stack.get_out(), "\\begin{{enumerate}}")?;
            writeln!(stack.get_out(), "\\setcounter{{enum{}}}{{{}}}", "i".repeat(enumerate_depth), start)?;
        } else {
            writeln!(stack.get_out(), "\\begin{{itemize}}")?;
        }

        Ok(List {
            start,
        })
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        if self.start.is_some() {
            writeln!(stack.get_out(), "\\end{{enumerate}}")?;
        } else {
            writeln!(stack.get_out(), "\\end{{itemize}}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Item;

impl<'a> State<'a> for Item {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(stack.get_out(), "\\item ")?;
        Ok(Item)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out())?;
        Ok(())
    }
}
