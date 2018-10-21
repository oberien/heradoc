extern crate pulldown_cmark;
extern crate str_concat;
extern crate structopt;
extern crate void;
extern crate boolinator;
extern crate tempdir;
extern crate typed_arena;
extern crate url;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use std::io::{self, Write, Result};

use structopt::StructOpt;
use tempdir::TempDir;
use typed_arena::Arena;

mod ext;
mod config;
mod resolve;
mod frontend;
mod generator;
mod backend;

use crate::config::{Config, CliArgs, FileConfig, OutType};
use crate::backend::latex::Article;

fn main() {
    let args = CliArgs::from_args();

    let mut markdown = String::new();
    args.input.to_read().read_to_string(&mut markdown).unwrap();

    let infile = if markdown.starts_with("```pundoc") || markdown.starts_with("```config") {
        let start = markdown.find('\n')
            .expect("unclosed preamble (not even a newline in the whole document)");
        let end = markdown.find("\n```").expect("unclosed preamble");
        let content = &markdown[(start + 1)..(end + 1)];
        let res = toml::from_str(content).expect("invalid config");
        markdown.drain(..(end + 4));
        res
    } else {
        FileConfig::default()
    };

    let cfgfile = args.configfile.as_ref().map(|p| p.as_path()).unwrap_or_else(|| Path::new("Config.toml"));
    let file = if cfgfile.is_file() {
        let content = fs::read_to_string(cfgfile)
            .expect("error reading existing config file");
        toml::from_str(&content).expect("invalid config")
    } else {
        FileConfig::default()
    };

    let tmpdir = TempDir::new("pundoc").expect("can't create tempdir");
    let cfg = Config::new(args, infile, file, &tmpdir);
    clear_dir(&cfg.out_dir).expect("can't clear directory");
    println!("{:#?}", cfg);

    // TODO bibliography
    match cfg.output_type {
        OutType::Latex => backend::generate(&cfg, Article, &Arena::new(), markdown, cfg.output.to_write()).unwrap(),
        OutType::Pdf => {
            let tex_path = tmpdir.path().join("document.tex");
            let tex_file = File::create(&tex_path)
                .expect("can't create temporary tex file");
            backend::generate(&cfg, Article, &Arena::new(), markdown, tex_file).unwrap();

            pdflatex(tmpdir.path());
            if cfg.bibliography.is_some() {
                biber(tmpdir.path());
                pdflatex(tmpdir.path());
            }
            pdflatex(tmpdir.path());
            let mut pdf = File::open(tmpdir.path().join("document.pdf"))
                .expect("unable to open generated pdf");
            io::copy(&mut pdf, &mut cfg.output.to_write()).expect("can't write to output");
        }
    }
}

fn pdflatex<P: AsRef<Path>>(tmpdir: P) {
    let tmpdir = tmpdir.as_ref();
    let mut pdflatex = Command::new("pdflatex");
    pdflatex.arg("-halt-on-error")
        .args(&["-interaction", "nonstopmode"])
        .arg("-output-directory").arg(tmpdir)
        .arg(tmpdir.join("document.tex"));
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

fn biber<P: AsRef<Path>>(tmpdir: P) {
    let tmpdir = tmpdir.as_ref();
    let mut biber = Command::new("biber");
    biber.arg("--output-directory").arg(tmpdir)
        .arg("document.bcf");
    let out = biber.output().expect("can't execute biber");
    if !out.status.success() {
        let _ = File::create("biber_stdout.log")
            .map(|mut f| f.write_all(&out.stdout));
        let _ = File::create("biber_stderr.log")
            .map(|mut f| f.write_all(&out.stderr));
        // TODO: provide better info about signals
        panic!("Biber returned error code {:?}. Logs written to biber_stdout.log and biber_stderr.log", out.status.code());
    }
}

fn clear_dir<P: AsRef<Path>>(dir: P) -> Result<()> {
    for e in fs::read_dir(dir)? {
        let e = e?;
        if e.file_type()?.is_dir() {
            fs::remove_dir_all(e.path())?;
        } else {
            fs::remove_file(e.path())?;
        }
    }
    Ok(())
}
