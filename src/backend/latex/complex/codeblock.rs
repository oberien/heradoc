use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{CodeBlock, Event};
use crate::generator::Generator;

#[derive(Debug)]
pub struct CodeBlockGen;

impl<'a> CodeGenUnit<'a, CodeBlock<'a>> for CodeBlockGen {
    fn new(
        _cfg: &'a Config, code_block: WithRange<CodeBlock<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(CodeBlock { label, caption, language }, _range) = code_block;

        let out = gen.get_out();
        write!(out, "\\begin{{lstlisting}}[")?;
        if let Some(WithRange(label, _)) = label {
            write!(out, "label={{{}}},", label)?;
        }
        if let Some(WithRange(caption, _)) = caption {
            write!(out, "caption={{{}}},", caption)?;
        }
        if let Some(WithRange(language, _)) = language {
            write!(out, "language={{{}}},", language)?;
        }
        writeln!(out, "]")?;

        Ok(CodeBlockGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{lstlisting}}")?;
        Ok(())
    }
}
