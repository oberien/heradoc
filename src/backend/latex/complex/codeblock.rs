use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{CodeBlock, Event};
use crate::generator::Generator;
use crate::util::OutJoiner;

#[derive(Debug)]
pub struct CodeBlockGen;

impl<'a> CodeGenUnit<'a, CodeBlock<'a>> for CodeBlockGen {
    fn new(
        _cfg: &'a Config, code_block: WithRange<CodeBlock<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(CodeBlock { label, caption, language }, _range) = code_block;

        let mut out = gen.get_out();
        write!(out, "\\begin{{lstlisting}}[")?;
        let mut joiner = OutJoiner::new(&mut out, ", ");
        if let Some(WithRange(label, _)) = label {
            joiner.join(format_args!("label={{{}}}", label))?;
        }
        if let Some(WithRange(caption, _)) = caption {
            joiner.join(format_args!("caption={{{}}}", caption))?;
        }
        if let Some(WithRange(language, _)) = language {
            // some fixes for weird latex-specific language naming
            let language = match language.as_ref() {
                "asm-x86" | "x86" | "x86-asm" => "[x86_64]{Assembler}",
                lang => lang
            };
            joiner.join(format_args!("language={{{}}}", language))?;
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
