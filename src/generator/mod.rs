use std::fmt;
use std::fs;
use std::io::Write;
use diagnostic::{Span, Spanned};

use crate::backend::{Backend, StatefulCodeGenUnit};
use crate::config::Config;
use crate::frontend::Frontend;
use crate::resolve::{Context, Include, Resolver, ResolveSecurity};
use crate::error::Diagnostics;

mod code_gen_units;
pub mod event;
mod iter;
mod stack;

pub use self::stack::Stack;

use self::code_gen_units::StackElement;
use self::event::Event;
use crate::error::{Error, FatalResult, Result};
use crate::generator::iter::Iter;

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    backend: B,
    cfg: &'a Config,
    default_out: W,
    stack: Vec<StackElement<'a, B>>,
    resolver: Resolver,
    template: Option<String>,
    diagnostics: &'a Diagnostics,
}

pub struct Events<'a> {
    events: Iter<'a>,
    context: Context,
}

impl<'a> fmt::Debug for Events<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Events")
            .field("events", &"Iter")
            .field("context", &self.context)
            .finish()
    }
}

impl<'a, B: Backend<'a>, W: Write> Generator<'a, B, W> {
    pub fn new(cfg: &'a Config, backend: B, default_out: W, diagnostics: &'a Diagnostics) -> Self {
        let template = cfg
            .template
            .as_ref()
            .map(|path| fs::read_to_string(path).expect("can't read template"));
        Generator {
            backend,
            cfg,
            default_out,
            stack: Vec::new(),
            resolver: Resolver::new(cfg.project_root.clone(), cfg.document_folder.clone(), cfg.temp_dir.clone()),
            template,
            diagnostics,
        }
    }

    pub fn get_events(&mut self, markdown: Spanned<&'a str>, context: Context) -> Events<'a> {
        let frontend = Frontend::new(self.cfg, markdown, self.diagnostics);
        let events = Iter::new(frontend);
        Events { events, context }
    }

    pub fn generate(&mut self, markdown: Spanned<&'a str>) -> FatalResult<()> {
        let context = Context::from_project_root();

        let events = self.get_events(markdown, context);
        if let Some(template) = self.template.take() {
            let body_index =
                template.find("\nHERADOCBODY\n").expect("HERADOCBODY not found in template");
            self.default_out.write_all(&template.as_bytes()[..body_index])?;
            self.generate_body(events)?;
            self.default_out.write_all(&template.as_bytes()[body_index + "\nHERADOCBODY\n".len()..])?;
        } else {
            self.backend.gen_preamble(self.cfg, &mut self.default_out, &*self.diagnostics)?;
            self.generate_body(events)?;
            assert!(self.stack.pop().is_none());
            self.backend.gen_epilogue(self.cfg, &mut self.default_out, &*self.diagnostics)?;
        }
        Ok(())
    }

    pub fn generate_body(&mut self, events: Events<'a>) -> FatalResult<()> {
        self.stack.push(StackElement::Context(events.context, self.diagnostics));
        let mut events = events.events;

        while let Some(Spanned { value: event, span }) = events.next(self)? {
            let peek = events.peek(self)?;
            match self.visit_event(Spanned::new(event, span), self.cfg, peek) {
                Ok(()) => {},
                Err(Error::Diagnostic) => events.skip(self)?,
                Err(Error::Fatal(fatal)) => return Err(fatal),
            }
        }
        match self.stack.pop() {
            Some(StackElement::Context(..)) => (),
            element => panic!(
                "Expected context as stack element after body generation is finished, got {:?}",
                element
            ),
        }
        Ok(())
    }

    pub fn visit_event(
        &mut self, event: Spanned<Event<'a>>, config: &'a Config, peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        let Spanned { value: event, span } = event;
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, self, peek)?;
            return Ok(());
        }

        match event {
            Event::End(_) => unreachable!(),
            Event::Start(tag) => {
                let state = StackElement::new(self.cfg, Spanned::new(tag, span), self)?;
                self.stack.push(state);
            },
            Event::Text(text) => B::Text::new(config, Spanned::new(text, span), self)?.finish(self, peek)?,
            Event::Html(html) => B::Text::new(config, Spanned::new(html, span), self)?.finish(self, peek)?,
            Event::Latex(latex) => B::Latex::new(config, Spanned::new(latex, span), self)?.finish(self, peek)?,
            Event::IncludeMarkdown(events) => self.generate_body(*events)?,
            Event::FootnoteReference(fnote) => {
                B::FootnoteReference::new(config, Spanned::new(fnote, span), self)?.finish(self, peek)?
            },
            Event::BiberReferences(biber) => {
                B::BiberReferences::new(config, Spanned::new(biber, span), self)?.finish(self, peek)?
            },
            Event::Url(url) => B::Url::new(config, Spanned::new(url, span), self)?.finish(self, peek)?,
            Event::InterLink(interlink) => B::InterLink::new(config, Spanned::new(interlink, span), self)?.finish(self, peek)?,
            Event::Image(img) => B::Image::new(config, Spanned::new(img, span), self)?.finish(self, peek)?,
            Event::Svg(svg) => B::Svg::new(config, Spanned::new(svg, span), self)?.finish(self, peek)?,
            Event::Label(label) => B::Label::new(config, Spanned::new(label, span), self)?.finish(self, peek)?,
            Event::Pdf(pdf) => B::Pdf::new(config, Spanned::new(pdf, span), self)?.finish(self, peek)?,
            Event::SoftBreak => B::SoftBreak::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::HardBreak => B::HardBreak::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::Rule => B::Rule::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::PageBreak => B::PageBreak::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::TaskListMarker(marker) => {
                B::TaskListMarker::new(config, Spanned::new(marker, span), self)?.finish(self, peek)?
            },
            Event::TableOfContents => B::TableOfContents::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::Bibliography => B::Bibliography::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::ListOfTables => B::ListOfTables::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::ListOfFigures => B::ListOfFigures::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::ListOfListings => B::ListOfListings::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
            Event::Appendix => B::Appendix::new(config, Spanned::new((), span), self)?.finish(self, peek)?,
        }

        Ok(())
    }

    pub fn stack(&mut self) -> Stack<'a, '_, B, W> {
        Stack::new(&mut self.default_out, &mut self.stack)
    }

    pub fn iter_stack(&self) -> impl Iterator<Item = &StackElement<'a, B>> {
        self.stack.iter().rev()
    }

    pub fn get_out<'s: 'b, 'b>(&'s mut self) -> &'b mut dyn Write {
        self.top_context().4
    }

    /// Because we don't have partial borrows, returns all information required somewhere in some
    /// combination, for example for resolving.
    fn top_context(
        &mut self
    ) -> (&mut Context, &'a Diagnostics, &mut Resolver, &mut B, &mut dyn Write) {
        let mut context = None;
        let mut out = None;
        for state in self.stack.iter_mut().rev() {
            if context.is_none() {
                match state {
                    StackElement::Context(ctx, diagnostics) => context = Some((ctx, diagnostics)),
                    _ => (),
                }
            } else if out.is_none() {
                out = state.output_redirect();
            }
        }

        let (context, diagnostics) = context.expect("no Context???");
        let out = out.unwrap_or(&mut self.default_out);
        (context, diagnostics, &mut self.resolver, &mut self.backend, out)
    }

    pub fn diagnostics(&mut self) -> &'a Diagnostics {
        self.top_context().1
    }

    pub fn backend_and_out(&mut self) -> (&'a Diagnostics, &mut B, &mut dyn Write) {
        let (_, diagnostics, _, backend, out) = self.top_context();
        (diagnostics, backend, out)
    }

    fn resolve(&mut self, resolve_security: ResolveSecurity, url: &str, span: Span) -> Result<Include> {
        let (context, diagnostics, resolver, _, _) = self.top_context();
        resolver.resolve(resolve_security, context, url, span, diagnostics)
    }
}
