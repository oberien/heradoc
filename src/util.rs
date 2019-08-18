use std::io::{Write, Result};
use std::fmt;

pub struct OutJoiner<W: Write> {
    out: W,
    need_comma: bool,
}

impl<W: Write> OutJoiner<W> {
    pub fn new(out: W) -> OutJoiner<W> {
        OutJoiner {
            out,
            need_comma: false,
        }
    }

    pub fn join(&mut self, args: fmt::Arguments<'_>) -> Result<()> {
        if self.need_comma {
            self.out.write_all(b", ")?;
        }
        self.need_comma = true;
        self.out.write_fmt(args)
    }

    pub fn into_inner(self) -> W {
        self.out
    }
}