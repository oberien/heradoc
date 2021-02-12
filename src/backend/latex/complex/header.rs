use std::borrow::Cow;
use std::io::Write;

use crate::backend::{Backend, CodeGenUnit, StatefulCodeGenUnit, latex::Beamer};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::frontend::range::{WithRange, SourceRange};
use crate::generator::event::{Event, Header};
use crate::generator::Generator;

#[derive(Debug)]
pub struct HeaderGen<'a> {
    label: WithRange<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for HeaderGen<'a> {
    fn new(
        _cfg: &'a Config, header: WithRange<Header<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(Header { label, level }, _range) = header;

        if level > 3 {
            gen.diagnostics()
                .error("Latex backed does not support header nesting more than 3 levels.")
                .emit();

            return Err(Error::Diagnostic);
        }

        write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(HeaderGen { label })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BookHeaderGen<'a> {
    label: WithRange<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for BookHeaderGen<'a> {
    fn new(
        _cfg: &'a Config, header: WithRange<Header<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(Header { label, level }, _range) = header;
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
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BeamerHeaderGen<'a> {
    cfg: &'a Config,
    level: i32,
    label: WithRange<Cow<'a, str>>,
    range: SourceRange,
}

impl<'a> StatefulCodeGenUnit<'a, Beamer, Header<'a>> for BeamerHeaderGen<'a> {
    fn new(
        cfg: &'a Config, header: WithRange<Header<'a>>,
        gen: &mut Generator<'a, Beamer, impl Write>,
    ) -> Result<Self> {
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        let WithRange(Header { label, level }, range) = header;

        // close old slide / beamerboxesrounded
        backend.close_until(level, &mut out, range, diagnostics)?;

        write!(out, "\\{}section{{", "sub".repeat(level as usize - 1))?;

        Ok(BeamerHeaderGen { cfg, level, label, range })
    }

    fn finish(
        self, gen: &mut Generator<'a, Beamer, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let BeamerHeaderGen { cfg, level, label, range } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        writeln!(out, "}}\\label{{{}}}\n", label.0)?;

        backend.open_until(level, cfg, &mut out, range, diagnostics)?;
        Ok(())
    }
}
