extern crate pulldown_cmark;
extern crate str_concat;
#[macro_use]
extern crate structopt;
extern crate void;
extern crate boolinator;

use pulldown_cmark::{Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use structopt::StructOpt;

mod concat;
mod gen;
mod config;

use concat::Concat;

fn main() {
    let mut cfg = config::Config::from_args();
    cfg.normalize();

    let mut markdown = String::new();
    cfg.input.to_read().read_to_string(&mut markdown).expect("Can't read input");

    let parser = Parser::new_with_broken_link_callback(
        &markdown,
        OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
        Some(&refsolve)
    );
    let events = Concat(parser.peekable()).collect::<Vec<_>>();
    println!("{:#?}", events);
    gen::generate(events, cfg.output.to_write()).unwrap();
}

fn refsolve(a: &str, b: &str) -> Option<(String, String)> {
    println!("Unk: {:?} {:?}", a, b);
    if a.starts_with('@') {
        Some(("biblatex-link-dst".to_string(), "title".to_string()))
    } else {
        Some((a.to_string(), b.to_string()))
    }
}

