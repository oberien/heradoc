use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Document, read_until};

#[derive(Debug)]
pub struct BlockQuote<'a> {
    events: Vec<Event<'a>>,
}

impl<'a> State<'a> for BlockQuote<'a> {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a>>], out: &mut impl Write) -> Result<Self> {
        Ok(BlockQuote {
            events: Vec::with_capacity(20),
        })
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        self.events.push(e);
        Ok(None)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        let quote = read_until(gen, self.events, peek)?;
        let mut quote = quote.as_str();

        // check if last line of quote is source of quote
        let mut source = None;
        if let Some(pos) = quote.trim_right().rfind("\n") {
            let src = &quote[pos+1..];
            if src.starts_with("--") {
                let src = src.trim_left_matches("-");
                source = Some(src.trim());
                quote = &quote[..pos+1];
            }
        }
        if let Some(source) = source {
            writeln!(out, "\\begin{{aquote}}{{{}}}", source)?;
        } else {
            writeln!(out, "\\begin{{quote}}")?;
        }
        write!(out, "{}", quote)?;
        if source.is_some() {
            writeln!(out, "\\end{{aquote}}")?;
        } else {
            writeln!(out, "\\end{{quote}}")?;
        }
        Ok(())
    }
}
