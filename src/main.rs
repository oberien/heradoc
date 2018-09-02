#![feature(rust_2018_preview)]

extern crate pulldown_cmark;
extern crate str_concat;
extern crate structopt;
extern crate void;
extern crate boolinator;
extern crate tempdir;
extern crate typed_arena;

use std::fs::File;
use std::process::Command;
use std::io::{self, Write};

use pulldown_cmark::{Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use structopt::StructOpt;
use tempdir::TempDir;
use typed_arena::Arena;

mod gen;
mod config;

use crate::config::{Config, OutType};
use crate::gen::latex::Article;

fn main() {
    let mut cfg = Config::from_args();
    cfg.normalize();
    println!("{:?}", cfg);

    let mut markdown = String::new();
    cfg.input.to_read().read_to_string(&mut markdown).unwrap();
    match cfg.output_type.unwrap() {
        OutType::Latex => gen::generate(Article, &Arena::new(), markdown, cfg.output.to_write()).unwrap(),
        OutType::Pdf => {
            let tmpdir = TempDir::new("pundoc").expect("can't create tempdir");
            let tex_path = tmpdir.path().join("document.tex");
            let tex_file = File::create(&tex_path)
                .expect("can't create temporary tex file");
            gen::generate(Article, &Arena::new(), markdown, tex_file).unwrap();
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

