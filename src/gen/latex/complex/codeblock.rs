use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct CodeBlock;

impl<'a> State<'a> for CodeBlock {
    fn new(tag: Tag<'a>, stack: &[States<'a, impl Document<'a>>], out: &mut impl Write) -> Result<Self> {
        let lang = match tag {
            Tag::CodeBlock(lang) => lang,
            _ => unreachable!("CodeBlock::new must be called with Tag::CodeBlock"),
        };
        write!(out, "\\begin{{lstlisting}}")?;
        if !lang.is_empty() {
            write!(out, "[")?;
            let parts = lang.split(",");
            for (i, part) in parts.enumerate() {
                if i == 0 {
                    if !part.contains("=") {
                        // TODO: language translation (use correct language, e.g. `Rust` instead of `rust` if that matters)
                        match &*lang {
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
        Ok(CodeBlock)
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        Ok(Some(e))
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\end{{lstlisting}}")?;
        Ok(())
    }
}
