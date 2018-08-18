use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event, Alignment};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct Table;

impl<'a> State<'a> for Table {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        let out = stack.get_out();
        let align = match tag {
            Tag::Table(align) => align,
            _ => unreachable!(),
        };

        // TODO: in-cell linebreaks
        // TODO: merging columns
        // TODO: merging rows
        // TODO: easier custom formatting
        write!(out, "\\begin{{tabular}}{{|")?;
        for align in align {
            match align {
                Alignment::None | Alignment::Left => write!(out, " l |")?,
                Alignment::Center => write!(out, " c |")?,
                Alignment::Right => write!(out, " r |")?,
            }
        }
        writeln!(out, "}}")?;
        writeln!(out, "\\hline")?;
        Ok(Table)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out(), "\\end{{tabular}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableHead;

impl<'a> State<'a> for TableHead {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(TableHead)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out(), "\\\\ \\thickhline")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableRow;

impl<'a> State<'a> for TableRow {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(TableRow)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out(), "\\\\ \\hline")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableCell;

impl<'a> State<'a> for TableCell {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(TableCell)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        if let Event::Start(Tag::TableCell) = peek.unwrap() {
            write!(stack.get_out(), "&")?;
        }
        Ok(())
    }
}
