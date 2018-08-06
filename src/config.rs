use std::path::PathBuf;
use std::fs::File;
use std::str::FromStr;
use std::io::{self, Read, Write};

use void::Void;

#[derive(StructOpt)]
#[structopt(name = "pundoc", about = "Convert Markdown to LaTeX / PDF")]
pub struct Config {
    #[structopt(short = "o", long = "out", long = "output", default_value = FileOrStream::StdIo)]
    pub output: FileOrStream,
    #[structopt(short = "i", long = "in", long = "input", default_value = FileOrStream::StdIo)]
    pub input: FileOrStream,

}

enum OutType {
    Latex,
    Pdf,
}

pub enum FileOrStream {
    StdIo,
    File(PathBuf),
}

impl FileOrStream {
    pub fn into_read(self) -> Box<Write> {
        Box::new(match self {
            FileOrStream::StdIo => Box::leak(Box::new(io::stdin())).lock(),
            FileOrStream::File(path) => File::open(path),
        })
    }

    pub fn into_write(self) -> Box<Write> {
        Box::new(match self {
            FileOrStream::StdIo => Box::leak(Box::new(io::stdout())).lock(),
            FileOrStream::File(path) => File::create(path),
        })
    }
}

impl FromStr for FileOrStream {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" | "-" => Ok(FileOrStream::StdIo),
            s => Ok(FileOrStream::File(PathBuf::from(s))),
        }
    }
}
