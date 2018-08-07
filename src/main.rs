extern crate pulldown_cmark;
extern crate str_concat;
#[macro_use]
extern crate structopt;
extern crate void;
extern crate boolinator;
extern crate tempdir;

use std::fs::File;
use std::process::Command;
use std::io::{self, Write};

use pulldown_cmark::{Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use structopt::StructOpt;
use tempdir::TempDir;

mod concat;
mod gen;
mod config;

use concat::Concat;
use config::{Config, OutType};

fn main() {
    let mut cfg = Config::from_args();
    cfg.normalize();
    println!("{:?}", cfg);

    let mut markdown = String::new();
    cfg.input.to_read().read_to_string(&mut markdown).expect("can't read input");

    let parser = Parser::new_with_broken_link_callback(
        &markdown,
        OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
        Some(&refsolve)
    );
    let events = Concat(parser.peekable()).collect::<Vec<_>>();
    println!("{:#?}", events);
    match cfg.output_type.unwrap() {
        OutType::Latex => gen::generate(events, cfg.output.to_write()).unwrap(),
        OutType::Pdf => {
            let tmpdir = TempDir::new("pundoc").expect("can't create tempdir");
            let tex_path = tmpdir.path().join("document.tex");
            let tex_file = File::create(&tex_path)
                .expect("can't create temporary tex file");
            gen::generate(events, tex_file).unwrap();
            let mut pdflatex = Command::new("pdflatex");
            pdflatex.arg("-halt-on-error")
                .args(&["-interaction", "nonstopmode"])
                .arg("-output-directory").arg(tmpdir.path())
                .arg(&tex_path);
            for _ in 0..3 {
                let out = pdflatex.output().expect("can't execute pdflatex");
                if !out.status.success() {
                    let _ = File::create("pdflatex_stdout.log")
                        .map(|mut f| f.write_all(&out.stdout));
                    let _ = File::create("pdflatex_stderr.log")
                        .map(|mut f| f.write_all(&out.stderr));
                    // TODO: provide better info about signals
                    panic!("Pdflatex returned error code {:?}. Logs written to pdflatex_stdout.log and pdflatex_stderr.log", out.status.code());
                }
            }
            let mut pdf = File::open(tmpdir.path().join("document.pdf"))
                .expect("unable to open generated pdf");
            io::copy(&mut pdf, &mut cfg.output.to_write()).expect("can't write to output");
        }
    }
}

fn refsolve(a: &str, b: &str) -> Option<(String, String)> {
    println!("Unk: {:?} {:?}", a, b);
    if a.starts_with('@') {
        Some(("biblatex-link-dst".to_string(), "title".to_string()))
    } else {
        Some((a.to_string(), b.to_string()))
    }
}

