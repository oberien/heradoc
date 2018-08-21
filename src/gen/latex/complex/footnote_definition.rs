use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct FootnoteDefinition;

impl<'a> State<'a> for FootnoteDefinition {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        let fnote = match tag {
            Tag::FootnoteDefinition(fnote) => fnote,
            _ => unreachable!(),
        };
        // TODO: Add pass to get all definitions to put definition on the same site as the first reference
        write!(gen.get_out(), "\\footnotetext{{\\label{{fnote:{}}}", fnote)?;
        Ok(FootnoteDefinition)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}")?;
        Ok(())
    }
}
