use std::io::Write;
use std::ops::Range;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::generator::event::Event;
use crate::generator::Generator;

#[derive(Debug)]
pub struct BlockQuoteGen {
    quote: Vec<u8>,
}

impl<'a> CodeGenUnit<'a, ()> for BlockQuoteGen {
    fn new(
        _cfg: &'a Config, _tag: (), _range: Range<usize>,
        _gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        Ok(BlockQuoteGen { quote: Vec::new() })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.quote)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<(&Event<'a>, Range<usize>)>,
    ) -> Result<()> {
        let out = gen.get_out();
        let quote = String::from_utf8(self.quote).expect("invalid UTF8");
        let mut quote = quote.as_str();

        // check if last line of quote is source of quote
        let mut source = None;
        if let Some(pos) = quote.trim_end().rfind('\n') {
            let src = &quote[pos + 1..];
            if src.starts_with("--") {
                let src = src.trim_start_matches('-');
                source = Some(src.trim());
                quote = &quote[..pos + 1];
            }
        }
        if let Some(source) = source {
            writeln!(out, "\\begin{{aquote}}{{{}}}", source)?;
        } else {
            writeln!(out, "\\begin{{quote}}")?;
        }
        write!(out, "{}", quote)?;
        if source.is_some() {
            writeln!(out, "\\end{{aquote}}")?;
        } else {
            writeln!(out, "\\end{{quote}}")?;
        }
        Ok(())
    }
}
