use std::io::{Result, Write};

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::event::{CodeBlock, Event};
use crate::generator::Generator;

#[derive(Debug)]
pub struct CodeBlockGen;

impl<'a> CodeGenUnit<'a, CodeBlock<'a>> for CodeBlockGen {
    fn new(
        _cfg: &'a Config, code_block: CodeBlock<'a>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let CodeBlock { label, caption, language } = code_block;

        let out = gen.get_out();
        write!(out, "\\begin{{lstlisting}}[")?;
        if let Some(label) = label {
            write!(out, "label={{{}}},", label)?;
        }
        if let Some(caption) = caption {
            write!(out, "caption={{{}}},", caption)?;
        }
        if let Some(language) = language {
            write!(out, "language={{{}}},", language)?;
        }
        writeln!(out, "]")?;

        Ok(CodeBlockGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{lstlisting}}")?;
        Ok(())
    }
}
