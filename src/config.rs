use std::path::PathBuf;
use std::fs::File;
use std::str::FromStr;
use std::io::{self, Read, Write};
use std::fmt;

use void::Void;
use boolinator::Boolinator;

#[derive(StructOpt)]
#[structopt(name = "pundoc", about = "Convert Markdown to LaTeX / PDF")]
pub struct Config {
    #[structopt(short = "o", long = "out", long = "output", default_value = "FileOrStream::StdIo")]
    pub output: FileOrStdio,
    #[structopt(short = "i", long = "in", long = "input", default_value = "FileOrStream::StdIo")]
    pub input: FileOrStdio,
    #[structopt(short = "t", long = "to", long = "type")]
    pub output_type: Option<OutType>,
}

impl Config {
    pub fn normalize(&mut self) {
        self.output_type.get_or_insert_with(|| match self.input {
            FileOrStdio::StdIo => OutType::Pdf,
            FileOrStdio::File(path) => {
                path.extension()
                    .and_then(|s| s.to_str())
                    .and_then(|s| {
                        (s.eq_ignore_ascii_case("tex") || s.eq_ignore_ascii_case("latex"))
                            .as_some(OutType::Latex)
                    })
                    .unwrap_or(OutType::Pdf)
            }
        });
    }
}

pub enum FileOrStdio {
    StdIo,
    File(PathBuf),
}

impl FileOrStdio {
    pub fn to_read(&self) -> Box<Read> {
        match self {
            FileOrStdio::StdIo => Box::new(Box::leak(Box::new(io::stdin())).lock()),
            FileOrStdio::File(path) => Box::new(File::open(path).expect("can't open input source")),
        }
    }

    pub fn to_write(&self) -> Box<Write> {
        match self {
            FileOrStdio::StdIo => Box::new(Box::leak(Box::new(io::stdout())).lock()),
            FileOrStdio::File(path) => Box::new(File::create(path).expect("can't open input source")),
        }
    }
}

impl FromStr for FileOrStdio {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" | "-" => Ok(FileOrStdio::StdIo),
            s => Ok(FileOrStdio::File(PathBuf::from(s))),
        }
    }
}

#[derive(Clone, Copy)]
pub enum OutType {
    Latex,
    Pdf,
}

struct OutTypeParseError<'a>(&'a str);

impl fmt::Display for OutTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown type `{}`", self.0)
    }
}

impl FromStr for OutType {
    type Err = OutTypeParseError<'a>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mapping = &[
            (&["tex", "latex"][..], OutType::Latex),
            (&["pdf"][..], OutType::Pdf)
        ];
        for &(list, res) in mapping {
            for variant in list {
                if s.eq_ignore_ascii_case(variant) {
                    return Ok(res);
                }
            }
        }
        Err(OutTypeParseError(s))
    }
}

