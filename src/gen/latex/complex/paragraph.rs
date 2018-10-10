use std::io::{Result, Write};

use crate::gen::{CodeGenUnit, PrimitiveGenerator, Backend};
use crate::config::Config;
use crate::parser::{Event, Tag};

#[derive(Debug)]
pub struct ParagraphGen;

impl<'a> CodeGenUnit<'a, ()> for ParagraphGen {
    fn new(_cfg: &'a Config, (): (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(ParagraphGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        // TODO: improve latex readability (e.g. no newline between list items)
        // TODO: fix too many linebreaks (e.g. after placeholditimage)
        match peek {
            Some(Event::Text(_))
            | Some(Event::Html(_))
            | Some(Event::InlineHtml(_))
            | Some(Event::Start(Tag::Paragraph))
            // those shouldn't occur after a par, but better safe than sorry
            | Some(Event::Start(Tag::InlineEmphasis))
            | Some(Event::Start(Tag::InlineStrong))
            | Some(Event::Start(Tag::InlineCode))
            | Some(Event::Start(Tag::Image(..))) => writeln!(gen.get_out(), "\\mbox{{}}\\\\\n\\mbox{{}}\\\\"),
            _ => writeln!(gen.get_out()),
        }
    }
}
