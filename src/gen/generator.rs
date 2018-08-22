use std::io::{Write, Result};
use std::path::Path;
use std::fs::File;

use pulldown_cmark::{Tag, Event};

use gen::{Document, Stack, State, States, Simple};

pub struct Generator<'a, D: Document<'a>, W: Write> {
    doc: D,
    default_out: W,
    stack: Vec<States<'a, D>>,
}

impl<'a, D: Document<'a>, W: Write> Generator<'a, D, W> {
    pub fn new(doc: D, default_out: W) -> Self {
        Generator {
            doc,
            default_out,
            stack: Vec::new(),
        }
    }

    pub fn generate(mut self, events: impl IntoIterator<Item = Event<'a>>) -> Result<()> {
        self.doc.gen_preamble(&mut self.default_out)?;
        let mut events = events.into_iter().peekable();

        while let Some(event) = events.next() {
            self.visit_event(event, events.peek())?;
        }
        self.doc.gen_epilogue(&mut self.default_out)?;
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

    fn handle_include(&mut self, text: &str) -> Result<bool> {
        let text = text.trim();
        let should_include = text.starts_with("!!include{") && text.ends_with("}")
            && !self.iter_stack().any(|state| state.is_inline() || state.is_code_block());

        if !should_include {
            return Ok(false);
        }
        let mut file = File::open(&text[10..text.len() - 1])?;
        let mut buf = String::new();
        let mut events = super::get_parser(&mut buf, file).peekable();
        while let Some(event) = events.next() {
            self.visit_event(event, events.peek())?;
        }
        Ok(false)
    }
}
