use std::io::{Result, Write};

use pulldown_cmark::{Event, Tag};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct Paragraph;

impl<'a> State<'a> for Paragraph {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(Paragraph)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        // TODO: improve readability (e.g. no newline between list items)
        match peek {
            Some(Event::Text(_))
            | Some(Event::Html(_))
            | Some(Event::InlineHtml(_))
            | Some(Event::Start(Tag::Paragraph))
            // those shouldn't occur after a par, but better safe than sorry
            | Some(Event::Start(Tag::Emphasis))
            | Some(Event::Start(Tag::Strong))
            | Some(Event::Start(Tag::Code))
            | Some(Event::Start(Tag::Link(..)))
            | Some(Event::Start(Tag::Image(..))) => writeln!(gen.get_out(), "\\\\\n\\\\"),
            _ => writeln!(gen.get_out()),
        }
    }
}
