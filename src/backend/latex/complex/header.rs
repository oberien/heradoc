use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::{PrimitiveGenerator, Stack};
use crate::config::Config;
use crate::generator::event::{Event, Header};

#[derive(Debug)]
pub struct HeaderGen {
    label: String,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for HeaderGen {
    fn new(_cfg: &'a Config, header: Header<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Header { level } = header;
        write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(HeaderGen {
            label: String::with_capacity(100),
        })
    }

    fn intercept_event<'b>(&mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match &e {
            Event::Text(text) => self.label.extend(text.chars().map(|c| match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
                _ => '-',
            })),
            _ => (),
        }
        Ok(Some(e))
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{sec:{}}}\n", self.label)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BookHeaderGen {
    label: String,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for BookHeaderGen {
    fn new(_cfg: &'a Config, header: Header<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Header { level } = header;
        if level == 1 {
            write!(gen.get_out(), "\\chapter{{")?;
        } else {
            write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 2))?;
        }
        Ok(BookHeaderGen {
            label: String::with_capacity(100),
        })
    }

    fn intercept_event<'b>(&mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match &e {
            Event::Text(text) => self.label.extend(text.chars().map(|c| match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
                _ => '-',
            })),
            _ => (),
        }
        Ok(Some(e))
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{sec:{}}}\n", self.label)?;
        Ok(())
    }
}
