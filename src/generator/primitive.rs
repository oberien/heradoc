use std::io::{Write, Result};

use crate::backend::{Backend, MediumCodeGenUnit};
use crate::config::Config;
use crate::generator::event::Event;
use super::{Stack, StackElement};
use crate::resolve::Context;

pub struct PrimitiveGenerator<'a, B: Backend<'a>, W: Write> {
    pub(super) cfg: &'a Config,
    pub(super) default_out: W,
    stack: Vec<StackElement<'a, B>>,
}

impl<'a, B: Backend<'a>, W: Write> PrimitiveGenerator<'a, B, W> {
    pub fn new(cfg: &'a Config, default_out: W, context: Context) -> Self {
        let mut gen = PrimitiveGenerator::without_context(cfg, default_out);
        gen.push(StackElement::Context(context));
        gen
    }

    pub fn without_context(cfg: &'a Config, default_out: W) -> Self {
        PrimitiveGenerator {
            cfg,
            default_out,
            stack: Vec::new(),
        }
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

        let mut stack = Stack::new(&mut self.default_out, &mut self.stack);
        match event {
            None => (),
            Some(Event::End(_)) => unreachable!(),
            Some(Event::Start(tag)) => {
                let state = StackElement::new(self.cfg, tag, self)?;
                self.stack.push(state);
            },
            Some(Event::Text(text)) => B::Text::gen(text, &mut stack)?,
            Some(Event::Html(html)) => B::Text::gen(html, &mut stack)?,
            Some(Event::InlineHtml(html)) => B::Text::gen(html, &mut stack)?,
            Some(Event::FootnoteReference(fnote)) => B::FootnoteReference::gen(fnote, &mut stack)?,
            Some(Event::Link(link)) => B::Link::gen(link, &mut stack)?,
            Some(Event::Image(img)) => B::Image::gen(img, &mut stack)?,
            Some(Event::Pdf(pdf)) => B::Pdf::gen(pdf, &mut stack)?,
            Some(Event::SoftBreak) => B::SoftBreak::gen((), &mut stack)?,
            Some(Event::HardBreak) => B::HardBreak::gen((), &mut stack)?,
            Some(Event::TableOfContents) => B::TableOfContents::gen((), &mut stack)?,
            Some(Event::Bibliography) => B::Bibliography::gen((), &mut stack)?,
            Some(Event::ListOfTables) => B::ListOfTables::gen((), &mut stack)?,
            Some(Event::ListOfFigures) => B::ListOfFigures::gen((), &mut stack)?,
            Some(Event::ListOfListings) => B::ListOfListings::gen((), &mut stack)?,
            Some(Event::Appendix) => B::Appendix::gen((), &mut stack)?,
        }

        Ok(())
    }

    pub fn iter_stack(&self) -> impl Iterator<Item=&StackElement<'a, B>> {
        self.stack.iter().rev()
    }

    pub(super) fn push(&mut self, unit: StackElement<'a, B>) {
        self.stack.push(unit);
    }

    pub(super) fn pop(&mut self) -> Option<StackElement<'a, B>> {
        self.stack.pop()
    }

    pub fn get_out<'s: 'b, 'b>(&'s mut self) -> &'b mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(&mut self.default_out)
    }
}
