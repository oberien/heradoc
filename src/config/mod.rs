use std::path::{PathBuf};
use std::fs::File;
use std::str::FromStr;
use std::io::{self, Read, Write};
use std::fmt;
use std::collections::{HashMap, HashSet};

use void::Void;
use boolinator::Boolinator;
use structopt::StructOpt;
use serde::{Deserialize, Deserializer, de};

mod geometry;

use self::geometry::{Geometry, Papersize, Orientation};

#[derive(StructOpt, Debug)]
#[structopt(name = "pundoc", about = "Convert Markdown to LaTeX / PDF")]
pub struct CliArgs {
    #[structopt(short = "o", long = "out", long = "output")]
    pub output: Option<FileOrStdio>,
    #[structopt()]
    pub input: FileOrStdio,
    #[structopt(flatten)]
    pub fileconfig: FileConfig,
}

#[derive(Debug, Default, Deserialize, StructOpt)]
pub struct FileConfig {
    #[structopt(short = "t", long = "to", long = "type", parse(try_from_str = "OutType::from_str"))]
    pub output_type: Option<OutType>,

    #[structopt(long = "bibliography")]
    pub bibliography: Option<String>,

    #[structopt(long = "documentclass")]
    pub documentclass: Option<String>,
    #[structopt(long = "fontsize")]
    pub fontsize: Option<String>,
    #[structopt(long = "titlepage")]
    pub titlepage: Option<bool>,
    #[structopt(long = "classoptions")]
    pub classoptions: Vec<String>,

    // geometry
    #[structopt(flatten)]
    pub geometry: Geometry,
}

#[derive(Debug)]
// TODO: make strongly typed
pub struct Config {
    // IO
    pub output: FileOrStdio,
    pub input: FileOrStdio,
    pub output_type: OutType,

    pub bibliography: Option<PathBuf>,

    // document
    pub documentclass: String,
    pub fontsize: String,
    pub titlepage: bool,
    pub classoptions: HashSet<String>,

    // geometry
    pub geometry: Geometry,
}

impl Config {
    pub fn new(args: CliArgs, infile: FileConfig, file: FileConfig) -> Config {
        // verify input file
        match &args.input {
            FileOrStdio::StdIo => (),
            FileOrStdio::File(path) if path.is_file() => (),
            FileOrStdio::File(path) => panic!("Invalid File {:?}", path),
        }
        // cli > infile > configfile
        let output_type = match args.fileconfig.output_type.or(infile.output_type).or(file.output_type) {
            Some(typ) => typ,
            None => match &args.output {
                Some(FileOrStdio::StdIo) => OutType::Latex,
                Some(FileOrStdio::File(path)) => path.extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| {
                        (ext.eq_ignore_ascii_case("tex") || ext.eq_ignore_ascii_case("latex"))
                            .as_some(OutType::Latex)
                    })
                    .unwrap_or(OutType::Pdf),
                None => OutType::Pdf,
            }
        };
        let output = match args.output {
            Some(fos) => fos,
            None => {
                let mut filename = match &args.input {
                    FileOrStdio::File(path) => PathBuf::from(path.file_stem().unwrap()),
                    FileOrStdio::StdIo => PathBuf::from("out"),
                };
                match output_type {
                    OutType::Latex => assert!(filename.set_extension("tex")),
                    OutType::Pdf => assert!(filename.set_extension("pdf")),
                }
                FileOrStdio::File(filename)
            }
        };

        let mut classoptions = HashSet::new();
        classoptions.extend(args.fileconfig.classoptions);
        classoptions.extend(infile.classoptions);
        classoptions.extend(file.classoptions);

        Config {
            output,
            input: args.input,
            output_type,
            bibliography: args.fileconfig.bibliography
                .or(infile.bibliography)
                .or(file.bibliography)
                .map(|bib| PathBuf::from(bib)),
            documentclass: args.fileconfig.documentclass
                .or(infile.documentclass)
                .or(file.documentclass)
                .unwrap_or_else(|| "scrartcl".to_string()),
            fontsize: args.fileconfig.fontsize
                .or(infile.fontsize)
                .or(file.fontsize)
                .unwrap_or_else(|| "10pt".to_string()),
            titlepage: args.fileconfig.titlepage
                .or(infile.titlepage)
                .or(file.titlepage)
                .unwrap_or(true),
            classoptions,
            geometry: args.fileconfig.geometry
                .merge(infile.geometry)
                .merge(file.geometry),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MaybeUnknown<T> {
    Known(T),
    Unknown(String),
}

impl<T: Default> Default for MaybeUnknown<T> {
    fn default() -> Self {
        MaybeUnknown::Known(T::default())
    }
}

impl<T: FromStr> FromStr for MaybeUnknown<T> {
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Void> {
        Ok(T::from_str(s)
            .map(MaybeUnknown::Known)
            .unwrap_or_else(|_| MaybeUnknown::Unknown(s.to_string())))
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

impl<'de> Deserialize<'de> for OutType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl FromStr for OutType {
    type Err = String;

    fn from_str(s: &str) -> Result<OutType, Self::Err> {
        if s.eq_ignore_ascii_case("tex") || s.eq_ignore_ascii_case("latex") {
            Ok(OutType::Latex)
        } else if s.eq_ignore_ascii_case("pdf") {
            Ok(OutType::Pdf)
        } else {
            Err(format!("unknown output type {:?}", s))
        }
    }
}
