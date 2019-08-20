use std::io::{Write, Result};
use std::fmt;

pub struct OutJoiner<'a, W: Write> {
    out: W,
    join_chars: &'a str,
    need_comma: bool,
}

impl<'a, W: Write> OutJoiner<'a, W> {
    pub fn new(out: W, join_chars: &'a str) -> OutJoiner<'a, W> {
        OutJoiner {
            out,
            join_chars,
            need_comma: false,
        }
    }

    pub fn join(&mut self, args: fmt::Arguments<'_>) -> Result<()> {
        if self.need_comma {
            self.out.write_all(self.join_chars.as_bytes())?;
        }
        self.need_comma = true;
        self.out.write_fmt(args)
    }
}