use std::borrow::Cow;
use std::io::Write;
use std::ops::Range;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::Generator;
use crate::generator::event::{Event, InterLink, Url};
use crate::error::Result;

#[derive(Debug)]
pub struct UrlWithContentGen<'a> {
    title: Option<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Url<'a>> for UrlWithContentGen<'a> {
    fn new(
        _cfg: &'a Config, url: Url<'a>, _range: Range<usize>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Url { destination, title } = url;
        let out = gen.get_out();

        if title.is_some() {
            write!(out, "\\pdftooltip{{\\href{{{}}}{{", destination)?;
        } else {
            write!(out, "\\href{{{}}}{{", destination)?;
        }
        Ok(UrlWithContentGen { title })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<(&Event<'a>, Range<usize>)>,
    ) -> Result<()> {
        let out = gen.get_out();

        match self.title {
            None => write!(out, "}}")?,
            Some(title) => write!(out, "}}}}{{{}}}", title)?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct InterLinkWithContentGen;

impl<'a> CodeGenUnit<'a, InterLink<'a>> for InterLinkWithContentGen {
    fn new(
        _cfg: &'a Config, interlink: InterLink<'a>, _range: Range<usize>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let InterLink { label, uppercase: _ } = interlink;
        write!(gen.get_out(), "\\hyperref[{}]{{", label)?;
        Ok(InterLinkWithContentGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<(&Event<'a>, Range<usize>)>,
    ) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}
