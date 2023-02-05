use std::io::Write;
use diagnostic::{Span, Spanned};
use crate::backend::StatefulCodeGenUnit;
use crate::{Beamer, Config};
use crate::generator::Generator;
use crate::error::Result;
use crate::generator::event::Event;

#[derive(Debug)]
pub struct BeamerPageBreakGen<'a> {
    cfg: &'a Config,
    span: Span,
}

impl<'a> StatefulCodeGenUnit<'a, Beamer, ()> for BeamerPageBreakGen<'a> {
    fn new(
        cfg: &'a Config, Spanned { span, .. }: Spanned<()>,
        _gen: &mut Generator<'a, Beamer, impl Write>,
    ) -> Result<Self> {
        Ok(BeamerPageBreakGen { cfg, span })
    }

    fn finish(
        self, gen: &mut Generator<'a, Beamer, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        let BeamerPageBreakGen { cfg, span } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        backend.close_until(2, &mut out, span, diagnostics)?;
        backend.open_until(2, cfg, &mut out, span, diagnostics)?;
        Ok(())
    }
}
