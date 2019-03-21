use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;
use crate::generator::event::Event;

#[derive(Debug)]
pub struct HtmlBlockGen;

// TODO: Not sure what to do here. Blind passthrough (current)? Panic? Error? Warn?
impl<'a> CodeGenUnit<'a, ()> for HtmlBlockGen {
    fn new(_cfg: &'a Config, (): (), _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        Ok(HtmlBlockGen)
    }

    fn finish(self, _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        Ok(())
    }
}
