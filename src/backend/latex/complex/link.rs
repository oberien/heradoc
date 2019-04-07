use std::borrow::Cow;
use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, InterLink, Url};
use crate::generator::Generator;

#[derive(Debug)]
pub struct UrlWithContentGen<'a> {
    title: Option<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Url<'a>> for UrlWithContentGen<'a> {
    fn new(
        _cfg: &'a Config, url: WithRange<Url<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(Url { destination, title }, _range) = url;
        let out = gen.get_out();

        if title.is_some() {
            write!(out, "\\pdftooltip{{\\href{{{}}}{{", destination)?;
        } else {
            write!(out, "\\href{{{}}}{{", destination)?;
        }
        Ok(UrlWithContentGen { title })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
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
        _cfg: &'a Config, interlink: WithRange<InterLink<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(InterLink { label, uppercase: _ }, _range) = interlink;
        write!(gen.get_out(), "\\hyperref[{}]{{", label)?;
        Ok(InterLinkWithContentGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        write!(gen.get_out(), "}}")?;
        Ok(())
    }
}
