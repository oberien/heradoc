use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct Header {
    label: String,
}

impl<'a> State<'a> for Header {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        let level = match tag {
            Tag::Header(level) => level,
            _ => unreachable!(),
        };
        write!(stack.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(Header {
            label: String::with_capacity(100),
        })
    }

    fn intercept_event<'b>(&mut self, e: &Event<'a>, stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        match e {
            Event::Text(text) => self.label.extend(text.chars().map(|c| match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
                _ => '-',
            })),
            _ => (),
        }
        Ok(())
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out(), "}}\\label{{{}}}\n", self.label)?;
        Ok(())
    }
}
