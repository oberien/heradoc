use std::fmt;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

use typed_arena::Arena;
use codespan_reporting::termcolor::StandardStream;

use crate::backend::{Backend, MediumCodeGenUnit};
use crate::config::{Config, FileOrStdio};
use crate::diagnostics::{Diagnostics, Input};
use crate::frontend::Frontend;
use crate::frontend::range::{SourceRange, WithRange};
use crate::resolve::{Context, Include, Resolver};

mod code_gen_units;
pub mod event;
mod iter;
mod stack;

pub use self::stack::Stack;

use self::code_gen_units::StackElement;
use self::event::Event;
use crate::error::{Error, FatalResult, Fatal, Result};
use crate::generator::iter::Iter;

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    arena: &'a Arena<String>,
    backend: B,
    cfg: &'a Config,
    default_out: W,
    stack: Vec<StackElement<'a, B>>,
    resolver: Resolver,
    template: Option<String>,
    stderr: Arc<Mutex<StandardStream>>,
}

pub struct Events<'a> {
    events: Iter<'a>,
    diagnostics: Arc<Diagnostics<'a>>,
    context: Context,
}

impl<'a> fmt::Debug for Events<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Events")
            .field("events", &"Iter")
            .field("diagnostics", &self.diagnostics)
            .field("context", &self.context)
            .finish()
    }
}

impl<'a, B: Backend<'a>, W: Write> Generator<'a, B, W> {
    pub fn new(
        cfg: &'a Config, backend: B, default_out: W, arena: &'a Arena<String>,
        stderr: Arc<Mutex<StandardStream>>,
    ) -> Self {
        let template = cfg
            .template
            .as_ref()
            .map(|path| fs::read_to_string(path).expect("can't read template"));
        Generator {
            arena,
            backend,
            cfg,
            default_out,
            stack: Vec::new(),
            resolver: Resolver::new(cfg.project_root.clone(), cfg.temp_dir.clone()),
            template,
            stderr,
        }
    }

    pub fn get_events(&mut self, markdown: String, context: Context, input: Input) -> Events<'a> {
        let markdown = self.arena.alloc(markdown);
        let diagnostics = Arc::new(Diagnostics::new(markdown, input, Arc::clone(&self.stderr)));
        let frontend = Frontend::new(self.cfg, markdown, Arc::clone(&diagnostics));
        let events = Iter::new(frontend);
        Events { events, diagnostics, context }
    }

    pub fn generate(&mut self, markdown: String) -> FatalResult<()> {
        let context = match Context::from_path(self.cfg.project_root.clone()) {
            Ok(context) => context,
            Err(e) => {
                self.diagnostics()
                    .bug("Context can't be created from project_root")
                    .note(format!("cause: {:?}", e))
                    .emit();
                return Err(Fatal::InteralCompilerError);
            }
        };

        let input = match &self.cfg.input {
            FileOrStdio::File(path) => Input::File(path.clone()),
            FileOrStdio::StdIo => Input::Stdin,
        };
        let events = self.get_events(markdown, context, input);
        if let Some(template) = self.template.take() {
            let body_index =
                template.find("\nHERADOCBODY\n").expect("HERADOCBODY not found in template on");
            self.get_out().write_all(&template.as_bytes()[..body_index])?;
            self.generate_body(events)?;
            self.get_out()
                .write_all(&template.as_bytes()[body_index + "\nHERADOCBODY\n".len()..])?;
        } else {
            self.backend.gen_preamble(self.cfg, &mut self.default_out, &events.diagnostics)?;
            self.generate_body(events)?;
            assert!(self.stack.pop().is_none());
            self.backend.gen_epilogue(self.cfg, &mut self.default_out, &events.diagnostics)?;
        }
        Ok(())
    }

    pub fn generate_body(&mut self, events: Events<'a>) -> FatalResult<()> {
        self.stack.push(StackElement::Context(events.context, events.diagnostics));
        let mut events = events.events;

        while let Some(WithRange(event, range)) = events.next(self)? {
            let peek = events.peek(self)?;
            match self.visit_event(WithRange(event, range), peek) {
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
        &mut self, event: WithRange<Event<'a>>, peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let WithRange(event, range) = event;
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, self, peek)?;
            return Ok(());
        }

        let mut stack = Stack::new(&mut self.default_out, &mut self.stack);
        match event {
            Event::End(_) => unreachable!(),
            Event::Start(tag) => {
                let state = StackElement::new(self.cfg, WithRange(tag, range), self)?;
                self.stack.push(state);
            },
            Event::Text(text) => B::Text::gen(WithRange(text, range), &mut stack)?,
            Event::Html(html) => B::Text::gen(WithRange(html, range), &mut stack)?,
            Event::InlineHtml(html) => B::Text::gen(WithRange(html, range), &mut stack)?,
            Event::Latex(latex) => B::Latex::gen(WithRange(latex, range), &mut stack)?,
            Event::IncludeMarkdown(events) => self.generate_body(*events)?,
            Event::FootnoteReference(fnote) => {
                B::FootnoteReference::gen(WithRange(fnote, range), &mut stack)?
            },
            Event::BiberReferences(biber) => {
                B::BiberReferences::gen(WithRange(biber, range), &mut stack)?
            },
            Event::Url(url) => B::Url::gen(WithRange(url, range), &mut stack)?,
            Event::InterLink(interlink) => B::InterLink::gen(WithRange(interlink, range), &mut stack)?,
            Event::Image(img) => B::Image::gen(WithRange(img, range), &mut stack)?,
            Event::Label(label) => B::Label::gen(WithRange(label, range), &mut stack)?,
            Event::Pdf(pdf) => B::Pdf::gen(WithRange(pdf, range), &mut stack)?,
            Event::SoftBreak => B::SoftBreak::gen(WithRange((), range), &mut stack)?,
            Event::HardBreak => B::HardBreak::gen(WithRange((), range), &mut stack)?,
            Event::TaskListMarker(marker) => {
                B::TaskListMarker::gen(WithRange(marker, range), &mut stack)?
            },
            Event::TableOfContents => B::TableOfContents::gen(WithRange((), range), &mut stack)?,
            Event::Bibliography => B::Bibliography::gen(WithRange((), range), &mut stack)?,
            Event::ListOfTables => B::ListOfTables::gen(WithRange((), range), &mut stack)?,
            Event::ListOfFigures => B::ListOfFigures::gen(WithRange((), range), &mut stack)?,
            Event::ListOfListings => B::ListOfListings::gen(WithRange((), range), &mut stack)?,
            Event::Appendix => B::Appendix::gen(WithRange((), range), &mut stack)?,
        }

        Ok(())
    }

    pub fn iter_stack(&self) -> impl Iterator<Item = &StackElement<'a, B>> {
        self.stack.iter().rev()
    }

    pub fn get_out<'s: 'b, 'b>(&'s mut self) -> &'b mut dyn Write {
        self.stack
            .iter_mut()
            .rev()
            .filter_map(|state| state.output_redirect())
            .next()
            .unwrap_or(&mut self.default_out)
    }

    fn top_context(&mut self) -> (&mut Context, &Diagnostics<'a>, &mut Resolver) {
        let (context, diagnostics) = self.stack
            .iter_mut()
            .rev()
            .find_map(|se| match se {
                StackElement::Context(context, diagnostics) => Some((context, diagnostics)),
                _ => None,
            })
            .expect("no Context???");
        // partial self borrows aren't a thing, so we need to return the resolver as well as it's
        // needed in Self::resolve
        (context, diagnostics, &mut self.resolver)
    }

    pub fn diagnostics(&mut self) -> &Diagnostics<'a> {
        self.top_context().1
    }

    pub fn backend(&mut self) -> &mut B {
        &mut self.backend
    }

    fn resolve(&mut self, url: &str, range: SourceRange) -> Result<Include> {
        let (context, diagnostics, resolver) = self.top_context();
        resolver.resolve(context, url, range, diagnostics)
    }
}
