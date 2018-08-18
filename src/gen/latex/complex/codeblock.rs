use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct CodeBlock;

impl<'a> State<'a> for CodeBlock {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        let out = stack.get_out();
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

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        writeln!(stack.get_out(), "\\end{{lstlisting}}")?;
        Ok(())
    }
}
