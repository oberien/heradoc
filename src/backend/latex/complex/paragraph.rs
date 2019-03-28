use std::io::{Result, Write};

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::event::{Event, Tag};
use crate::generator::PrimitiveGenerator;

#[derive(Debug)]
pub struct ParagraphGen;

impl<'a> CodeGenUnit<'a, ()> for ParagraphGen {
    fn new(
        _cfg: &'a Config, _tag: (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        Ok(ParagraphGen)
    }

    fn finish(
        self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>,
        peek: Option<&Event<'a>>,
    ) -> Result<()> {
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
            | Some(Event::Image(_)) => writeln!(gen.get_out(), "\n")?,
            Some(Event::End(Tag::FootnoteDefinition(_))) => (),
            _ => writeln!(gen.get_out(), "\n")?,
        }
        Ok(())
    }
}
