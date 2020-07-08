use std::io::{Write, Result};
use std::path::{Path, Component, Prefix};
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

pub trait ToUnix {
    fn to_unix(&self) -> Option<String>;
}
impl ToUnix for Path {
    #[cfg(not(windows))]
    fn to_unix(&self) -> Option<String> {
        self.to_str().to_owned()
    }

    #[cfg(windows)]
    fn to_unix(&self) -> Option<String> {
        let mut buf = String::new();
        for c in self.components() {
            match c {
                Component::Prefix(prefix) => {
                    match prefix.kind() {
                        Prefix::Verbatim(s) => {
                            buf.push('/');
                            buf.push_str(s.to_str()?);
                        }
                        Prefix::VerbatimUNC(_, _) | Prefix::UNC(_, _) => todo!(),
                        Prefix::DeviceNS(_) => todo!(),
                        Prefix::VerbatimDisk(disk) | Prefix::Disk(disk) => {
                            buf.push(disk as char);
                            buf.push(':');
                        }
                    }
                }
                Component::RootDir => (),
                Component::CurDir => buf.push('.'),
                Component::ParentDir => buf.push_str(".."),
                Component::Normal(s) => buf.push_str(s.to_str()?),
            }
            buf.push('/');
        }

        if buf != "/" {
            // Pop last `/`
            buf.pop();
        }
        Some(buf)
    }
}