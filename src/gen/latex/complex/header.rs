use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct Header {
    label: String,
}

impl<'a> State<'a> for Header {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        let level = match tag {
            Tag::Header(level) => level,
            _ => unreachable!(),
        };
        write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(Header {
            label: String::with_capacity(100),
        })
    }

    fn intercept_event<'b>(&mut self, stack: &mut Stack<'a, 'b, impl Document<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match &e {
            Event::Text(text) => self.label.extend(text.chars().map(|c| match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
                _ => '-',
            })),
            _ => (),
        }
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{sec:{}}}\n", self.label)?;
        Ok(())
    }
}
