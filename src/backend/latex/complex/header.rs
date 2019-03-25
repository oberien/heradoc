use std::io::{Result, Write};
use std::borrow::Cow;

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::PrimitiveGenerator;
use crate::config::Config;
use crate::generator::event::{Event, Header};

#[derive(Debug)]
pub struct HeaderGen<'a> {
    label: Cow<'a, str>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for HeaderGen<'a> {
    fn new(_cfg: &'a Config, header: Header<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Header { label, level } = header;
        assert!(level >= 0, "Header level should be positive, but is {}", level);
        write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(HeaderGen {
            label,
        })
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BookHeaderGen<'a> {
    label: Cow<'a, str>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for BookHeaderGen<'a> {
    fn new(_cfg: &'a Config, header: Header<'a>, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Header { label, level } = header;
        assert!(level >= 0, "Header level should be positive, but is {}", level);
        if level == 1 {
            write!(gen.get_out(), "\\chapter{{")?;
        } else {
            write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 2))?;
        }
        Ok(BookHeaderGen {
            label,
        })
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label)?;
        Ok(())
    }
}
