use std::io::{Result, Write};

use crate::gen::{CodeGenUnit, Generator, Stack, Backend};
use crate::config::Config;
use crate::parser::{Event, Header};

#[derive(Debug)]
pub struct HeaderGen {
    label: String,
}

impl<'a> CodeGenUnit<'a, Header> for HeaderGen {
    fn new(_cfg: &'a Config, header: Header, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        write!(gen.get_out(), "\\{}section{{", "sub".repeat(header.level as usize - 1))?;
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

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{sec:{}}}\n", self.label)?;
        Ok(())
    }
}
