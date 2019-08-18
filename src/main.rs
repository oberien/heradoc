#![forbid(unsafe_code)]
// groups
#![warn(nonstandard_style)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![warn(future_incompatible)]
// single lints not in groups from https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
#![allow(box_pointers)]
#![warn(elided_lifetimes_in_paths)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
// for now
#![allow(variant_size_differences)]
#![allow(missing_docs)]
// seems to have quite some unchangeable false positives
// might need further inspection
#![allow(single_use_lifetimes)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic/*, clippy::cargo*/)]
#![allow(clippy::match_bool)]
#![allow(clippy::range_plus_one)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::use_self)]
#![allow(clippy::result_map_unwrap_or_else)]
#![allow(clippy::if_not_else)]
#![allow(clippy::single_match_else)]

use std::env;
use std::fs::{self, File};
use std::io::{self, Result, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

use structopt::StructOpt;
use tempdir::TempDir;
use typed_arena::Arena;
use codespan_reporting::termcolor::{ColorChoice, StandardStream};

mod backend;
mod config;
mod cskvp;
mod diagnostics;
mod error;
mod ext;
mod frontend;
mod generator;
mod resolve;
mod util;

use crate::backend::{Backend, latex::{Article, Beamer, Report, Thesis}};
use crate::config::{CliArgs, Config, DocumentType, FileConfig, OutType};
use crate::error::Fatal;

fn main() {
    let args = CliArgs::from_args();

    let mut markdown = String::new();
    args.input.to_read().read_to_string(&mut markdown).unwrap();

    let infile = if markdown.starts_with("```heradoc") || markdown.starts_with("```config") {
        let start = markdown
            .find('\n')
            .expect("unclosed preamble (not even a newline in the whole document)");
        let end = markdown.find("\n```").expect("unclosed preamble");
        let content = &markdown[(start + 1)..(end + 1)];
        let res = toml::from_str(content).expect("invalid config");
        markdown.drain(..(end + 4));
        res
    } else {
        FileConfig::default()
    };

    let cfgfile =
        args.configfile.as_ref().map_or_else(|| Path::new("Config.toml"), |p| p.as_path());
    let file = if cfgfile.is_file() {
        let content = fs::read_to_string(cfgfile).expect("error reading existing config file");
        toml::from_str(&content).expect("invalid config")
    } else {
        FileConfig::default()
    };

    let tmpdir = TempDir::new("heradoc").expect("can't create tempdir");
    let cfg = Config::new(args, infile, file, &tmpdir);
    if cfg.out_dir != cfg.temp_dir {
        // While initializing the config, some files may already be downloaded.
        // Thus we must only clear the output directory if it's not a temporary directory.
        clear_dir(&cfg.out_dir).expect("can't clear output directory");
    }
    println!("{:#?}", cfg);

    match cfg.output_type {
        OutType::Latex => gen(&cfg, markdown, cfg.output.to_write()),
        OutType::Pdf => {
            let tex_path = tmpdir.path().join("document.tex");
            let tex_file = File::create(&tex_path).expect("can't create temporary tex file");
            gen(&cfg, markdown, tex_file);

            pdflatex(&tmpdir, &cfg);
            if cfg.bibliography.is_some() {
                biber(&tmpdir);
                pdflatex(&tmpdir, &cfg);
            }
            pdflatex(&tmpdir, &cfg);
            let mut pdf = File::open(tmpdir.path().join("document.pdf"))
                .expect("unable to open generated pdf");
            io::copy(&mut pdf, &mut cfg.output.to_write()).expect("can't write to output");
        },
    }
}

fn gen(cfg: &Config, markdown: String, out: impl Write) {
    // TODO: make this configurable
    let stderr = Arc::new(Mutex::new(StandardStream::stderr(ColorChoice::Auto)));
    let res = match cfg.document_type {
        DocumentType::Article => backend::generate(cfg, Article::new(), &Arena::new(), markdown, out, stderr),
        DocumentType::Beamer => backend::generate(cfg, Beamer::new(), &Arena::new(), markdown, out, stderr),
        DocumentType::Report => backend::generate(cfg, Report::new(), &Arena::new(), markdown, out, stderr),
        DocumentType::Thesis => backend::generate(cfg, Thesis::new(), &Arena::new(), markdown, out, stderr),
    };
    match res {
        Ok(()) => (),
        Err(Fatal::Output(io)) => eprintln!("\n\nerror writing to output: {}", io),
        Err(Fatal::InternalCompilerError) => eprintln!("\n\nCan not continue due to internal error"),
    }
}

fn pdflatex<P: AsRef<Path>>(tmpdir: P, cfg: &Config) {
    let tmpdir = tmpdir.as_ref();
    let mut pdflatex = Command::new("pdflatex");
    pdflatex
        .arg("-halt-on-error")
        .args(&["-interaction", "nonstopmode"])
        .arg("-output-directory")
        .arg(tmpdir)
        .arg(tmpdir.join("document.tex"));
    if let Some(template) = &cfg.template {
        if let Some(parent) = template.parent() {
            let mut texinputs = env::var_os("TEXINPUTS").unwrap_or_default();
            texinputs.push(":");
            texinputs.push(parent);
            pdflatex.env("TEXINPUTS", texinputs);
        }
    }
    let out = pdflatex.output().expect("can't execute pdflatex");
    if !out.status.success() {
        let _ = File::create("pdflatex_stdout.log").map(|mut f| f.write_all(&out.stdout));
        let _ = File::create("pdflatex_stderr.log").map(|mut f| f.write_all(&out.stderr));
        // TODO: provide better info about signals
        panic!(
            "Pdflatex returned error code {:?}. Logs written to pdflatex_stdout.log and \
             pdflatex_stderr.log",
            out.status.code()
        );
    }
}

fn biber<P: AsRef<Path>>(tmpdir: P) {
    let tmpdir = tmpdir.as_ref();
    let mut biber = Command::new("biber");
    biber.arg("--output-directory").arg(tmpdir).arg("document.bcf");
    let out = biber.output().expect("can't execute biber");
    if !out.status.success() {
        let _ = File::create("biber_stdout.log").map(|mut f| f.write_all(&out.stdout));
        let _ = File::create("biber_stderr.log").map(|mut f| f.write_all(&out.stderr));
        // TODO: provide better info about signals
        panic!(
            "Biber returned error code {:?}. Logs written to biber_stdout.log and biber_stderr.log",
            out.status.code()
        );
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
