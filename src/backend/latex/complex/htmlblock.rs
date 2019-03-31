use std::io::{Result, Write};

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::event::Event;
use crate::generator::Generator;

#[derive(Debug)]
pub struct HtmlBlockGen;

// TODO: Not sure what to do here. Blind passthrough (current)? Panic? Error? Warn?
impl<'a> CodeGenUnit<'a, ()> for HtmlBlockGen {
    fn new(
        _cfg: &'a Config, (): (), _gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        Ok(HtmlBlockGen)
    }

    fn finish(
        self, _gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>,
    ) -> Result<()> {
        Ok(())
    }
}
