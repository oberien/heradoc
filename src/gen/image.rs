use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::{Tag, Event};

use super::{State, States, Generator, read_until};

#[derive(Debug)]
pub struct Image<'a> {
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    events: Vec<Event<'a>>,
}

impl<'a> State<'a> for Image<'a> {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        let (dst, title) = match tag {
            Tag::Image(dst, title) => (dst, title),
            _ => unreachable!(),
        };

        writeln!(out, "\\begin{{figure}}")?;
        writeln!(out, "\\includegraphics{{{}}}", dst)?;

        Ok(Image {
            dst,
            title,
            events: Vec::new(),
        })
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        self.events.push(e);
        Ok(None)
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        let caption = read_until(gen, self.events, peek)?;

        if !caption.is_empty() {
            writeln!(out, "\\caption{{{}}}", caption)?;
        }
        if !self.title.is_empty() {
            writeln!(out, "\\label{{img:{}}}", self.title)?;
        }
        writeln!(out, "\\end{{figure}}")?;
        Ok(())
    }
}
