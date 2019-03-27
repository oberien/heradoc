use std::io::{Result, Write};
use std::borrow::Cow;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::event::{Url, InterLink, Event};
use crate::generator::PrimitiveGenerator;

#[derive(Debug)]
pub struct UrlWithContentGen<'a> {
    title: Option<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Url<'a>> for UrlWithContentGen<'a> {
    fn new(
        _cfg: &'a Config, url: Url<'a>,
        gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>,
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
        self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<&Event<'a>>,
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
        _cfg: &'a Config, interlink: InterLink<'a>,
        gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let InterLink { label, uppercase: _ } = interlink;
        write!(gen.get_out(), "\\hyperref[{}]{{", label)?;
        Ok(InterLinkWithContentGen)
    }

    fn finish(
        self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<&Event<'a>>,
    ) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}
