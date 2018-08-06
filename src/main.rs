extern crate pulldown_cmark;
extern crate str_concat;
extern crate structopt;
extern crate void;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use pulldown_cmark::{Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use structopt::StructOpt;

mod concat;
mod gen;
mod config;

use concat::Concat;

#[derive(StructOpt)]
#[structopt(name = "pundoc", about = "Convert Markdown to LaTeX / PDF")]
struct Opts {
    /// Output file to write to
    ///
    /// If left blank or `-` is specified, output will be written to std.
    #[structopt(short = "o", parse(from_os_str))]
    output: Option<PathBuf>,
    /// Input markdown file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output format to overwrite the determined one based on the file ending
    #[structopt(short = "t", long = "type", long = "outtype")]
    output_format: Option<OutType>,
}

enum OutType {
    Latex,
    Pdf,
}

impl FromStr for OutType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mapping = &[(&["tex", "latex"], OutType::Latex),
            (&["pdf"], OutType::Pdf)];
        for (list, res) in mapping {
            for variant in list {
                if s.eq_ignore_ascii_case(variant) {
                    return Ok(res);
                }
            }
        }
        Err(())
    }
}

fn main() {
    let opts = Opts::from_args();
    let outtype =

    let s = fs::read_to_string(opts.input).unwrap();
    let parser = Parser::new_with_broken_link_callback(
        &s,
        OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
        Some(&refsolve)
    );
    let events = Concat(parser.peekable()).collect::<Vec<_>>();
    println!("{:#?}", events);
    gen::generate(events, stdout.lock()).unwrap();
}

fn refsolve(a: &str, b: &str) -> Option<(String, String)> {
    println!("Unk: {:?} {:?}", a, b);
    if a.starts_with('@') {
        Some(("biblatex-link-dst".to_string(), "title".to_string()))
    } else {
        Some((a.to_string(), b.to_string()))
    }
}

