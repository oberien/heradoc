use std::io::{Write, Result};
use std::path::{Path, PathBuf, Component};
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

pub fn strip_root<P: AsRef<Path>>(p: P) -> PathBuf {
    p.as_ref().components().filter(|p| match p {
        Component::Prefix(_) => false,
        Component::RootDir => false,
        _ => true
    }).collect()
}

pub trait ToUnix {
    fn to_unix(&self) -> Option<String>;
}
impl ToUnix for Path {
    #[cfg(not(windows))]
    fn to_unix(&self) -> Option<String> {
        self.to_str().map(str::to_owned)
    }

    #[cfg(windows)]
    fn to_unix(&self) -> Option<String> {
        use std::path::Prefix;

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