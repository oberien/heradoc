use std::io::Write;
use diagnostic::Spanned;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::generator::event::{CodeBlock, Event};
use crate::generator::Generator;
use crate::util::OutJoiner;

#[derive(Debug)]
pub struct CodeBlockGen;

impl<'a> CodeGenUnit<'a, CodeBlock<'a>> for CodeBlockGen {
    fn new(
        _cfg: &'a Config, code_block: Spanned<CodeBlock<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: CodeBlock { label, caption, language, basicstyle }, .. } = code_block;

        let mut out = gen.get_out();
        write!(out, "\\begin{{lstlisting}}[")?;
        let mut joiner = OutJoiner::new(&mut out, ", ");
        if let Some(Spanned { value: label, .. }) = label {
            joiner.join(format_args!("label={{{}}}", label))?;
        }
        if let Some(Spanned { value: caption, .. }) = caption {
            joiner.join(format_args!("caption={{{}}}", caption))?;
        }
        if let Some(Spanned { value: language, .. }) = language {
            // some fixes for weird latex-specific language naming
            let language = match language.as_ref() {
                "asm-x86" | "x86" | "x86-asm" => "[x86_64]{Assembler}",
                lang => lang
            };
            joiner.join(format_args!("language={{{}}}", language))?;
        }
        if let Some(Spanned { value: basicstyle, .. }) = basicstyle {
            joiner.join(format_args!("basicstyle={{{basicstyle}}}"))?;
        }

        writeln!(out, "]")?;

        Ok(CodeBlockGen)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{lstlisting}}")?;
        Ok(())
    }
}
