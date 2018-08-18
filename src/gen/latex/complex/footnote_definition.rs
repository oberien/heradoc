use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct FootnoteDefinition;

impl<'a> State<'a> for FootnoteDefinition {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a>>], out: &mut impl Write) -> Result<Self> {
        let fnote = match tag {
            Tag::FootnoteDefinition(fnote) => fnote,
            _ => unreachable!(),
        };
        // TODO: Add pass to get all definitions to put definition on the same site as the first reference
        write!(out, "\\footnotetext{{\\label{{fnote:{}}}", fnote)?;
        Ok(FootnoteDefinition)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "}}")?;
        Ok(())
    }
}
