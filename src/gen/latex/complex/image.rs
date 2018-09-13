use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::{Tag, Event, LinkType};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct Image<'a> {
    typ: LinkType,
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    caption: Vec<u8>,
}

impl<'a> State<'a> for Image<'a> {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        let out = gen.get_out();
        let (typ, dst, title) = match tag {
            Tag::Image(typ, dst, title) => (typ, dst, title),
            _ => unreachable!(),
        };

        writeln!(out, "\\begin{{figure}}")?;
        writeln!(out, "\\includegraphics{{{}}}", dst)?;

        Ok(Image {
            typ,
            dst,
            title,
            caption: Vec::new(),
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.caption)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
        let caption = String::from_utf8(self.caption).expect("inavlid UTF8");

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
