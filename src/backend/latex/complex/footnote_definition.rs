use std::io::Write;
use diagnostic::Spanned;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::generator::event::{Event, FootnoteDefinition};
use crate::generator::Generator;

#[derive(Debug)]
pub struct FootnoteDefinitionGen;

impl<'a> CodeGenUnit<'a, FootnoteDefinition<'a>> for FootnoteDefinitionGen {
    fn new(
        _cfg: &'a Config, fnote: Spanned<FootnoteDefinition<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: FootnoteDefinition { label }, .. } = fnote;
        // TODO: Add pass to get all definitions to put definition on the same site as the first
        // reference
        write!(gen.get_out(), "\\footnotetext{{\\label{{fnote:{}}}", label)?;
        Ok(FootnoteDefinitionGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}")?;
        Ok(())
    }
}
