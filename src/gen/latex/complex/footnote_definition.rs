use std::io::{Result, Write};

use crate::gen::{CodeGenUnit, CodeGenUnits, Generator, Backend};
use crate::config::Config;

use crate::parser::{Event, FootnoteDefinition};

#[derive(Debug)]
pub struct FootnoteDefinitionGen;

impl<'a> CodeGenUnit<'a, FootnoteDefinition<'a>> for FootnoteDefinitionGen {
    fn new(cfg: &'a Config, fnote: FootnoteDefinition<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        // TODO: Add pass to get all definitions to put definition on the same site as the first reference
        write!(gen.get_out(), "\\footnotetext{{\\label{{fnote:{}}}", fnote.label)?;
        Ok(FootnoteDefinitionGen)
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}")?;
        Ok(())
    }
}
