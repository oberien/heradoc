use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;
use crate::generator::event::{CodeBlock, Event};

#[derive(Debug)]
pub struct CodeBlockGen;

impl<'a> CodeGenUnit<'a, CodeBlock<'a>> for CodeBlockGen {
    fn new(_cfg: &'a Config, code_block: CodeBlock<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let out = gen.get_out();
        write!(out, "\\begin{{lstlisting}}")?;
        if !code_block.language.is_empty() {
            write!(out, "[")?;
            let parts = code_block.language.split(",");
            for (i, part) in parts.enumerate() {
                if i == 0 {
                    if !part.contains("=") {
                        // TODO: language translation (use correct language, e.g. `Rust` instead of `rust` if that matters)
                        match &*code_block.language {
                            // TODO: sequence and stuff generation
                            "sequence" => (),
                            lang => write!(out, "language={}", lang)?,
                        }
                        continue;
                    }
                }

                if !part.contains("=") {
                    panic!("any code-block argument except the first one (language) must be of format `key=value`");
                }
                write!(out, "{}", part)?;
            }
            write!(out, "]")?;
        }
        writeln!(out)?;
        Ok(CodeBlockGen)
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "\\end{{lstlisting}}")?;
        Ok(())
    }
}
