use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, FootnoteDefinition};
use crate::generator::Generator;

#[derive(Debug)]
pub struct FootnoteDefinitionGen;

impl<'a> CodeGenUnit<'a, FootnoteDefinition<'a>> for FootnoteDefinitionGen {
    fn new(
        _cfg: &'a Config, fnote: WithRange<FootnoteDefinition<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(FootnoteDefinition { label }, _range) = fnote;
        // TODO: Add pass to get all definitions to put definition on the same site as the first
        // reference
        write!(gen.get_out(), "\\footnotetext{{\\label{{fnote:{}}}", label)?;
        Ok(FootnoteDefinitionGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}")?;
        Ok(())
    }
}
