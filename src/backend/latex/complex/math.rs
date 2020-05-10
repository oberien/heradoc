use std::borrow::Cow;
use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, MathBlock, MathBlockKind};
use crate::generator::Generator;

#[derive(Debug)]
pub struct InlineMathGen;

impl<'a> CodeGenUnit<'a, ()> for InlineMathGen {
    fn new(
        _cfg: &Config, _: WithRange<()>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        write!(gen.get_out(), "\\begin{{math}}")?;
        Ok(InlineMathGen)
    }

    fn finish(
        self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        write!(gen.get_out(), "\\end{{math}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MathBlockGen<'a> {
    _label: Option<WithRange<Cow<'a, str>>>,
    kind: MathBlockKind,
}

impl<'a> CodeGenUnit<'a, MathBlock<'a>> for MathBlockGen<'a> {
    fn new(
        _cfg: &Config, eq: WithRange<MathBlock<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(MathBlock { kind, label, caption }, _range) = eq;
        let out = gen.get_out();

        match kind {
            MathBlockKind::Equation => writeln!(out, "\\begin{{align*}}")?,
            MathBlockKind::NumberedEquation => writeln!(out, "\\begin{{align}}")?,
        }

        Ok(MathBlockGen { _label: label, kind })
    }

    fn finish(
        self, gen: &'_ mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let out = gen.get_out();
        match self.kind {
            MathBlockKind::Equation => writeln!(out, "\\end{{align*}}")?,
            MathBlockKind::NumberedEquation => writeln!(out, "\\end{{align}}")?,
        }
        Ok(())
    }
}
