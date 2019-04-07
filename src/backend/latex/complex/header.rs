use std::borrow::Cow;
use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, Header};
use crate::generator::Generator;

#[derive(Debug)]
pub struct HeaderGen<'a> {
    label: WithRange<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for HeaderGen<'a> {
    fn new(
        _cfg: &'a Config, header: WithRange<Header<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(Header { label, level }, _range) = header;
        assert!(level >= 0, "Header level should be positive, but is {}", level);
        write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 1))?;
        Ok(HeaderGen { label })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BookHeaderGen<'a> {
    label: WithRange<Cow<'a, str>>,
}

impl<'a> CodeGenUnit<'a, Header<'a>> for BookHeaderGen<'a> {
    fn new(
        _cfg: &'a Config, header: WithRange<Header<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(Header { label, level }, _range) = header;
        assert!(level >= 0, "Header level should be positive, but is {}", level);
        if level == 1 {
            write!(gen.get_out(), "\\chapter{{")?;
        } else {
            write!(gen.get_out(), "\\{}section{{", "sub".repeat(level as usize - 2))?;
        }
        Ok(BookHeaderGen { label })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        writeln!(gen.get_out(), "}}\\label{{{}}}\n", self.label.0)?;
        Ok(())
    }
}
