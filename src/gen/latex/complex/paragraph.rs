use std::io::{Result, Write};

use pulldown_cmark::{Event, Tag};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct Paragraph;

impl<'a> State<'a> for Paragraph {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(Paragraph)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
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
            | Some(Event::Start(Tag::Image(..))) => writeln!(stack.get_out(), "\\\\\n\\\\"),
            _ => writeln!(stack.get_out()),
        }
    }
}
