use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use super::{State, States, Generator};

#[derive(Debug)]
pub struct Header<'a> {
    level: i32,
    label: String,
    events: Vec<Event<'a>>,
}

impl<'a> State<'a> for Header<'a> {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        let level = match tag {
            Tag::Header(level) => level,
            _ => unreachable!(),
        };
        Ok(Header {
            level,
            label: String::with_capacity(100),
            events: Vec::new(),
        })
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        match &e {
            Event::Text(text) => self.label.extend(text.chars().map(|c| match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
                _ => '-',
            })),
            _ => (),
        }
        self.events.push(e);
        Ok(None)
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "\\{}section{{", "sub".repeat(self.level as usize - 1))?;
        for event in self.events {
            gen.visit_event(event, None, out)?;
        }
        writeln!(out, "}}\\label{{{}}}\n", self.label)?;
        Ok(())
    }
}
