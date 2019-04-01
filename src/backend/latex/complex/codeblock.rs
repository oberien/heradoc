use std::io::Write;
use std::ops::Range;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::Generator;
use crate::generator::event::{CodeBlock, Event};
use crate::error::Result;

#[derive(Debug)]
pub struct CodeBlockGen;

impl<'a> CodeGenUnit<'a, CodeBlock<'a>> for CodeBlockGen {
    fn new(
        _cfg: &'a Config, code_block: CodeBlock<'a>, _range: Range<usize>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let CodeBlock { label, caption, language } = code_block;

        let out = gen.get_out();
        write!(out, "\\begin{{lstlisting}}[")?;
        if let Some((label, _)) = label {
            write!(out, "label={{{}}},", label)?;
        }
        if let Some((caption, _)) = caption {
            write!(out, "caption={{{}}},", caption)?;
        }
        if let Some((language, _)) = language {
            write!(out, "language={{{}}},", language)?;
        }
        writeln!(out, "]")?;

        Ok(CodeBlockGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<(&Event<'a>, Range<usize>)>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{lstlisting}}")?;
        Ok(())
    }
}
