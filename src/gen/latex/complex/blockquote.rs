use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct BlockQuote {
    quote: Vec<u8>
}

impl<'a> State<'a> for BlockQuote {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        Ok(BlockQuote {
            quote: Vec::new(),
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.quote)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        let out = stack.get_out();
        let quote = String::from_utf8(self.quote).expect("invalid UTF8");
        let mut quote = quote.as_str();

        // check if last line of quote is source of quote
        let mut source = None;
        if let Some(pos) = quote.trim_right().rfind("\n") {
            let src = &quote[pos+1..];
            if src.starts_with("--") {
                let src = src.trim_left_matches("-");
                source = Some(src.trim());
                quote = &quote[..pos+1];
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
