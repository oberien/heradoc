use std::io::{Read, Write, Result};
use std::fs::File;
use std::iter::Peekable;

use pulldown_cmark::{Parser as CmarkParser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use typed_arena::Arena;

use crate::backend::{Backend, CodeGenUnits, SimpleCodeGenUnit};
use crate::frontend::{Frontend, Event};
use crate::config::Config;

mod concat;
mod stack;
mod primitive;

pub use self::stack::Stack;
pub use self::primitive::PrimitiveGenerator;

use self::concat::Concat;

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    arena: &'a Arena<String>,
    doc: B,
    prim: PrimitiveGenerator<'a, B, W>,
}

impl<'a, B: Backend<'a>, W: Write> Generator<'a, B, W> {
    pub fn new(cfg: &'a Config, doc: B, default_out: W, arena: &'a Arena<String>) -> Self {
        Generator {
            arena,
            doc,
            prim: PrimitiveGenerator::new(cfg, default_out),
        }
    }

    pub fn get_events(&mut self, markdown: String) -> Peekable<impl Iterator<Item = Event<'a>>> {
        let markdown = self.arena.alloc(markdown);
        let parser: Frontend<'_, B> = Frontend::new(self.prim.cfg, CmarkParser::new_with_broken_link_callback(
            markdown,
            OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
            Some(&refsolve)
        ));
        // TODO: don't print events
        let events: Vec<_> = Concat(parser.peekable()).collect();
//        println!("{:#?}", events);
        events.into_iter().peekable()
    }

    pub fn generate_body(&mut self, mut events: Peekable<impl Iterator<Item=Event<'a>>>) -> Result<()> {
        while let Some(event) = events.next() {
            self.visit_event(event, events.peek())?;
        }
        Ok(())
    }

    pub fn generate(&mut self, events: Peekable<impl Iterator<Item = Event<'a>>>) -> Result<()> {
        self.doc.gen_preamble(self.prim.cfg, &mut self.prim.default_out)?;
        self.generate_body(events)?;
        self.doc.gen_epilogue(self.prim.cfg, &mut self.prim.default_out)?;
        Ok(())
    }

    pub fn visit_event(&mut self, event: Event<'a>, peek: Option<&Event<'a>>) -> Result<()> {
        // intercept event and test for commands etc before the PrimitiveGenerator gets it and
        // forwards it to the backend

        match event {
            Event::Text(text) => if !self.handle_include(&text)? {
                self.prim.visit_event(Event::Text(text), peek)?;
            },
            evt => self.prim.visit_event(evt, peek)?,
        }

        Ok(())
    }

    /// Checks if the passed text is an include and handles it if it is.
    ///
    /// Returns true if the text was consumed and false if the caller should handle it further.
    fn handle_include(&mut self, text: &str) -> Result<bool> {
        let text = text.trim();
        let should_include = text.starts_with("!!include{") && text.ends_with("}")
            && !self.prim.iter_stack().any(|state| state.is_inline() || state.is_code_block());

        if !should_include {
            return Ok(false);
        }

        let mut file = File::open(&text[10..text.len() - 1])?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let events = self.get_events(buf);
        self.generate_body(events).map(|()| true)
    }
}

fn refsolve(a: &str, b: &str) -> Option<(String, String)> {
    // pass everything, it's handled in the frontend::refs implementation
    Some((a.to_string(), b.to_string()))
}

