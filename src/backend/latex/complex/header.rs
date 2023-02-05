use std::borrow::Cow;
use std::io::Write;
use diagnostic::{Span, Spanned};

use crate::backend::{Backend, CodeGenUnit, StatefulCodeGenUnit, latex::Beamer};
use crate::config::Config;
use crate::error::Result;
use crate::generator::event::{Event, Header};
use crate::generator::Generator;

#[derive(Debug)]
pub struct HeaderGen<'a> {
    label: Spanned<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for HeaderGen<'a> {
    fn new(
        _cfg: &'a Config, header: Spanned<Header<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: Header { label, level }, .. } = header;
        write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(HeaderGen { label })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label.value)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BookHeaderGen<'a> {
    label: Spanned<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for BookHeaderGen<'a> {
    fn new(
        _cfg: &'a Config, header: Spanned<Header<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: Header { label, level }, .. } = header;
        assert!(level > 0, "Header level should be positive, but is {}", level);
        if level == 1 {
            write!(gen.get_out(), "\\chapter{{")?;
        } else {
            write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 2))?;
        }
        Ok(BookHeaderGen { label })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label.value)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BeamerHeaderGen<'a> {
    cfg: &'a Config,
    level: i32,
    label: Spanned<Cow<'a, str>>,
    span: Span,
}

impl<'a> StatefulCodeGenUnit<'a, Beamer, Header<'a>> for BeamerHeaderGen<'a> {
    fn new(
        cfg: &'a Config, header: Spanned<Header<'a>>,
        gen: &mut Generator<'a, Beamer, impl Write>,
    ) -> Result<Self> {
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        let Spanned { value: Header { label, level }, span } = header;

        // close old slide / beamerboxesrounded
        backend.close_until(level, &mut out, span, diagnostics)?;

        write!(out, "\\{}section{{", "sub".repeat(level as usize - 1))?;

        Ok(BeamerHeaderGen { cfg, level, label, span })
    }

    fn finish(
        self, gen: &mut Generator<'a, Beamer, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        let BeamerHeaderGen { cfg, level, label, span } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        writeln!(out, "}}\\label{{{}}}\n", label.value)?;

        backend.open_until(level, cfg, &mut out, span, diagnostics)?;
        Ok(())
    }
}
