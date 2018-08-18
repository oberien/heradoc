use std::io::{Result, Write};
use std::fmt::Debug;

use pulldown_cmark::{Tag, Event, Alignment};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct Table;

impl<'a> State<'a> for Table {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
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

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\end{{tabular}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableHead;

impl<'a> State<'a> for TableHead {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
        Ok(TableHead)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\\\ \\thickhline")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableRow;

impl<'a> State<'a> for TableRow {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
        Ok(TableRow)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\\\ \\hline")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableCell;

impl<'a> State<'a> for TableCell {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a> + Debug>], out: &mut impl Write) -> Result<Self> {
        Ok(TableCell)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a> + Debug>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        if let Event::Start(Tag::TableCell) = peek.unwrap() {
            write!(out, "&")?;
        }
        Ok(())
    }
}
