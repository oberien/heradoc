use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;

use crate::generator::event::{Event, FootnoteDefinition};

#[derive(Debug)]
pub struct FootnoteDefinitionGen;

impl<'a> CodeGenUnit<'a, FootnoteDefinition<'a>> for FootnoteDefinitionGen {
    fn new(_cfg: &'a Config, fnote: FootnoteDefinition<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let FootnoteDefinition { label } = fnote;
        // TODO: Add pass to get all definitions to put definition on the same site as the first reference
        write!(gen.get_out(), "\\footnotetext{{\\label{{fnote:{}}}", label)?;
        Ok(FootnoteDefinitionGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}")?;
        Ok(())
    }
}
