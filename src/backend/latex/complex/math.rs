use std::io::Write;

use crate::backend::latex::InlineEnvironment;
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
    inline_fig: InlineEnvironment<'a>,
    kind: MathBlockKind,
}

impl<'a> CodeGenUnit<'a, MathBlock<'a>> for MathBlockGen<'a> {
    fn new(
        _cfg: &Config, eq: WithRange<MathBlock<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(MathBlock { kind, label, caption }, _range) = eq;
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        let out = gen.get_out();
        inline_fig.write_begin(&mut *out)?;

        match kind {
            MathBlockKind::Equation => writeln!(out, "\\begin{{align*}}")?,
            MathBlockKind::NumberedEquation => writeln!(out, "\\begin{{align}}")?,
        }

        Ok(MathBlockGen { inline_fig, kind })
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
        self.inline_fig.write_end(out)?;
        Ok(())
    }
}
