use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct FootnoteDefinition;

impl<'a> State<'a> for FootnoteDefinition {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        let fnote = match tag {
            Tag::FootnoteDefinition(fnote) => fnote,
            _ => unreachable!(),
        };
        // TODO: Add pass to get all definitions to put definition on the same site as the first reference
        write!(stack.get_out(), "\\footnotetext{{\\label{{fnote:{}}}", fnote)?;
        Ok(FootnoteDefinition)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out(), "}}")?;
        Ok(())
    }
}
