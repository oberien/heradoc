use std::io::Write;
use crate::backend::StatefulCodeGenUnit;
use crate::{Beamer, Config};
use crate::frontend::range::{SourceRange, WithRange};
use crate::generator::Generator;
use crate::error::Result;
use crate::generator::event::Event;

#[derive(Debug)]
pub struct BeamerPageBreakGen<'a> {
    cfg: &'a Config,
    range: SourceRange,
}

impl<'a> StatefulCodeGenUnit<'a, Beamer, ()> for BeamerPageBreakGen<'a> {
    fn new(
        cfg: &'a Config, WithRange(_, range): WithRange<()>,
        _gen: &mut Generator<'a, Beamer, impl Write>,
    ) -> Result<Self> {
        Ok(BeamerPageBreakGen { cfg, range })
    }

    fn finish(
        self, gen: &mut Generator<'a, Beamer, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let BeamerPageBreakGen { cfg, range } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        backend.close_until(2, &mut out, range, diagnostics)?;
        backend.open_until(2, cfg, &mut out, range, diagnostics)?;
        Ok(())
    }
}
