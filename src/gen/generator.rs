use std::io::{Read, Write, Result};
use std::path::Path;
use std::fs::File;
use std::iter::Peekable;

use pulldown_cmark::{Tag, Event, Parser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use typed_arena::Arena;

use crate::gen::{Document, Stack, State, States, Simple};
use crate::gen::concat::Concat;
use crate::config::Config;

pub struct Generator<'a, D: Document<'a>, W: Write> {
    cfg: &'a Config,
    arena: &'a Arena<String>,
    doc: D,
    default_out: W,
    stack: Vec<States<'a, D>>,
}

impl<'a, D: Document<'a>, W: Write> Generator<'a, D, W> {
    pub fn new(cfg: &'a Config, doc: D, default_out: W, arena: &'a Arena<String>) -> Self {
        Generator {
            cfg,
            arena,
            doc,
            default_out,
            stack: Vec::new(),
        }
    }

    pub fn get_events(&mut self, markdown: String) -> Peekable<impl Iterator<Item = Event<'a>>> {
        let markdown = self.arena.alloc(markdown);
        let parser = Parser::new_with_broken_link_callback(
            markdown,
            OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
            Some(&refsolve)
        );
        // TODO: don't print events
        let events: Vec<_> = Concat(parser.peekable()).collect();
//        println!("{:#?}", events);
        events.into_iter().peekable()
    }

    pub fn generate(&mut self, events: Peekable<impl Iterator<Item = Event<'a>>>) -> Result<()> {
        self.doc.gen_preamble(self.cfg, &mut self.default_out)?;
        self.generate_body(events);
        self.doc.gen_epilogue(self.cfg, &mut self.default_out)?;
        Ok(())
    }

    pub fn generate_body(&mut self, mut events: Peekable<impl Iterator<Item = Event<'a>>>) -> Result<()> {
        while let Some(event) = events.next() {
            self.visit_event(event, events.peek())?;
        }
        Ok(())
    }

    pub fn visit_event(&mut self, event: Event<'a>, peek: Option<&Event<'a>>) -> Result<()> {
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, self, peek)?;
            return Ok(());
        }

        let event = if !self.stack.is_empty() {
            let index = self.stack.len() - 1;
            let (stack, last) = self.stack.split_at_mut(index);
            last[0].intercept_event(&mut Stack::new(&mut self.default_out, stack), event)?
        } else {
            Some(event)
        };

        match event {
            None => (),
            Some(Event::End(_)) => unreachable!(),
            Some(Event::Start(tag)) => {
                let state = States::new(tag, self)?;
                self.stack.push(state);
            },
            Some(Event::Text(text)) => if !self.handle_include(&text)? {
                D::Simple::gen_text(&text, &mut self.get_out())?
            },
            Some(Event::Html(html)) => unimplemented!(),
            Some(Event::InlineHtml(html)) => unimplemented!(),
            Some(Event::FootnoteReference(fnote)) => D::Simple::gen_footnote_reference(&fnote, &mut self.get_out())?,
            Some(Event::SoftBreak) => D::Simple::gen_soft_break(&mut self.get_out())?,
            Some(Event::HardBreak) => D::Simple::gen_hard_break(&mut self.get_out())?,
        }

        Ok(())
    }

    pub fn iter_stack(&self) -> impl Iterator<Item = &States<'a, D>> {
        self.stack.iter()
    }

    pub fn get_out<'s: 'b, 'b>(&'s mut self) -> &'b mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(&mut self.default_out)
    }

    /// Checks if the passed text is an include and handles it if it is.
    ///
    /// Returns true if the text was consumed and false if the caller should handle it further.
    fn handle_include(&mut self, text: &str) -> Result<bool> {
        let text = text.trim();
        let should_include = text.starts_with("!!include{") && text.ends_with("}")
            && !self.iter_stack().any(|state| state.is_inline() || state.is_code_block());

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
    // pass everything, it's handled in the respective link implementation
    Some((a.to_string(), b.to_string()))
}
