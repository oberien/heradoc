use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::LinkType;

use crate::gen::{CodeGenUnit, CodeGenUnits, Generator, Backend};
use crate::config::Config;
use crate::parser::{Event, Link};

#[derive(Debug)]
pub struct ImageGen<'a> {
    typ: LinkType,
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    caption: Vec<u8>,
}

impl<'a> CodeGenUnit<'a, Link<'a>> for ImageGen<'a> {
    fn new(cfg: &'a Config, link: Link<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let out = gen.get_out();
        let Link { typ, dst, title } = link;

        writeln!(out, "\\begin{{figure}}")?;
        writeln!(out, "\\includegraphics{{{}}}", dst)?;

        Ok(ImageGen {
            typ,
            dst,
            title,
            caption: Vec::new(),
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.caption)
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
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
