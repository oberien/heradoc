extern crate pulldown_cmark;
extern crate str_concat;

use std::fs;
use std::io;

use pulldown_cmark::{Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};

mod concat;
mod gen;

use concat::Concat;

fn main() {
    let s = fs::read_to_string("test.md").unwrap();
    let parser = Parser::new_with_broken_link_callback(
        &s,
        OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
        Some(&refsolve)
//        None
    );
    let events = Concat(parser.peekable()).collect::<Vec<_>>();
    println!("{:#?}", events);
    let stdout = io::stdout();
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

