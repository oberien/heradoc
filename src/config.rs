use std::path::{PathBuf};
use std::fs::File;
use std::str::FromStr;
use std::io::{self, Read, Write};
use std::fmt;

use void::Void;
use boolinator::Boolinator;

#[derive(StructOpt, Debug)]
#[structopt(name = "pundoc", about = "Convert Markdown to LaTeX / PDF")]
pub struct Config {
    #[structopt(short = "o", long = "out", long = "output", default_value = "-")]
    pub output: FileOrStdio,
    #[structopt()]
    pub input: FileOrStdio,
    #[structopt(short = "t", long = "to", long = "type", parse(try_from_str = "OutType::from_str"))]
    pub output_type: Option<OutType>,
}

impl Config {
    pub fn normalize(&mut self) {
        let output = &self.output;
        self.output_type.get_or_insert_with(|| match output {
            FileOrStdio::StdIo => OutType::Latex,
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

#[derive(Debug)]
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
            FileOrStdio::File(path) => Box::new(File::create(path).expect("can't open output source")),
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

#[derive(Debug, Clone, Copy)]
pub enum OutType {
    Latex,
    Pdf,
}

#[derive(Debug)]
struct OutTypeParseError<'a>(&'a str);

impl<'a> fmt::Display for OutTypeParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown type `{}`", self.0)
    }
}

impl OutType {
    fn from_str<'a>(s: &'a str) -> Result<OutType, OutTypeParseError<'a>> {
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
