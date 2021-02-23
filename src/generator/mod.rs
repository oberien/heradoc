use std::borrow::Cow;
use std::fmt;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

use typed_arena::Arena;
use codespan_reporting::termcolor::StandardStream;

use crate::backend::{Backend, StatefulCodeGenUnit};
use crate::config::{Config, FileOrStdio};
use crate::diagnostics::{Diagnostics, Input};
use crate::frontend::{Frontend, Tag};
use crate::frontend::range::{SourceRange, WithRange};
use crate::frontend::rustdoc::{Crate, Rustdoc};
use crate::resolve::{Context, Include, Resolver, ResolveSecurity};

mod code_gen_units;
pub mod event;
mod iter;
mod stack;

pub use self::stack::Stack;

use self::code_gen_units::StackElement;
use self::event::Event;
use crate::error::{Error, FatalResult, Result};
use crate::generator::iter::MarkdownIter;

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    arena: &'a Arena<String>,
    backend: B,
    cfg: &'a Config,
    default_out: W,
    stack: Vec<StackElement<'a, B>>,
    resolver: Resolver,
    template: Option<String>,
    stderr: Arc<Mutex<StandardStream>>,
    header_level_adjustment: i32,
    latest_header_level: i32,
}

pub struct Events<'a> {
    events: MarkdownIter<'a>,
    diagnostics: Arc<Diagnostics<'a>>,
    context: Context,
    /// Whether to generate all events with an adjusted header level.
    adjust_header_levels: bool,
}

impl<'a> fmt::Debug for Events<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Events")
            .field("events", &"MarkdownIter")
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
            resolver: Resolver::new(cfg.project_root.clone(), cfg.document_folder.clone(), cfg.cache_dir.clone()),
            template,
            stderr,
            header_level_adjustment: 0,
            latest_header_level: 0 
        }
    }

    pub fn get_events(&mut self, markdown: Cow<'a, str>, context: Context, input: Input) -> Events<'a> {
        let markdown = match markdown {
            Cow::Owned(markdown) => self.arena.alloc(markdown),
            Cow::Borrowed(borrowed) => borrowed,
        };
        let diagnostics = Arc::new(Diagnostics::new(markdown, input, Arc::clone(&self.stderr)));
        let frontend = Frontend::new(self.cfg, markdown, Arc::clone(&diagnostics));
        let events = MarkdownIter::new(frontend);
        Events { events, diagnostics, context, adjust_header_levels: false }
    }

    pub fn get_rustdoc(&mut self, target: Crate, context: Context) -> FatalResult<Rustdoc<'a>> {
        let target = target.generate(self.diagnostics())?;
        let diagnostics = Arc::new(Diagnostics::new("", Input::Stdin, Arc::clone(&self.stderr)));
        Ok(Rustdoc::new(self.cfg, target, diagnostics, context))
    }

    pub fn generate(&mut self, markdown: String) -> FatalResult<()> {
        let context = Context::from_project_root();

        let input = match &self.cfg.input {
            FileOrStdio::File(path) => Input::File(path.clone()),
            FileOrStdio::StdIo => Input::Stdin,
        };
        let events = self.get_events(markdown.into(), context, input);
        if let Some(template) = self.template.take() {
            let body_index =
                template.find("\nHERADOCBODY\n").expect("HERADOCBODY not found in template");
            self.default_out.write_all(&template.as_bytes()[..body_index])?;
            self.generate_body(events)?;
            self.default_out.write_all(&template.as_bytes()[body_index + "\nHERADOCBODY\n".len()..])?;
        } else {
            let diagnostics = Arc::clone(&events.diagnostics);
            self.backend.gen_preamble(self.cfg, &mut self.default_out, &diagnostics)?;
            self.generate_body(events)?;
            assert!(self.stack.pop().is_none());
            self.backend.gen_epilogue(self.cfg, &mut self.default_out, &diagnostics)?;
        }
        Ok(())
    }

    pub fn generate_body(&mut self, events: Events<'a>) -> FatalResult<()> {
        // Store the level adjustment.
        let context = StackElement::Context(events.context, events.diagnostics, self.header_level_adjustment);

        self.header_level_adjustment = self.latest_header_level;
        self.generate_markdown_in(events.events, context)?;

        Ok(())
    }

    pub fn generate_rustdoc(&mut self, events: Rustdoc<'a>) -> FatalResult<()> {
        // TODO: we should push some context..
        let context = StackElement::Context(
            events.context().clone(),
            events.diagnostics(),
            self.header_level_adjustment);
        self.generate_markdown_in(MarkdownIter::with_rustdoc(events), context)?;

        Ok(())
    }

    fn generate_markdown_in(&mut self, events: MarkdownIter<'a>, context: StackElement<'a, B>) -> FatalResult<()> {
        self.stack.push(context);

        self.generate_markdown(events)?;

        match self.stack.pop() {
            Some(StackElement::Context(_, _, header_level_adjustment)) => {
                self.header_level_adjustment = header_level_adjustment;
            },
            element => panic!(
                "Expected context as stack element after body generation is finished, got {:?}",
                element
            ),
        }

        Ok(())
    }

    fn generate_markdown(&mut self, mut events: MarkdownIter<'a>) -> FatalResult<()> {
        while let Some(WithRange(event, range)) = events.next(self)? {
            let peek = events.peek(self)?;
            match self.visit_event(WithRange(event, range), self.cfg, peek) {
                Ok(()) => {},
                Err(Error::Diagnostic) => events.skip(self)?,
                Err(Error::Fatal(fatal)) => return Err(fatal),
            }
        }
        Ok(())
    }

    pub fn visit_event(
        &mut self, event: WithRange<Event<'a>>, config: &'a Config, peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let WithRange(event, range) = event;
        if let Event::End(tag) = event {
            let state = self.stack.pop().unwrap();
            state.finish(tag, self, peek)?;
            return Ok(());
        }

        match event {
            Event::End(_) => unreachable!(),
            Event::Start(tag) => {
                let tag = self.adjust_tag_from_context(tag);
                if let Tag::Header(header) = &tag {
                    self.latest_header_level = header.level;
                }
                let state = StackElement::new(self.cfg, WithRange(tag, range), self)?;
                self.stack.push(state);
            },
            Event::Text(text) => B::Text::new(config, WithRange(text, range), self)?.finish(self, peek)?,
            Event::Html(html) => B::Text::new(config, WithRange(html, range), self)?.finish(self, peek)?,
            Event::InlineHtml(html) => B::Text::new(config, WithRange(html, range), self)?.finish(self, peek)?,
            Event::Latex(latex) => B::Latex::new(config, WithRange(latex, range), self)?.finish(self, peek)?,
            Event::IncludeMarkdown(events) => self.generate_body(*events)?,
            Event::IncludeRustdoc(events) => self.generate_rustdoc(*events)?,
            Event::FootnoteReference(fnote) => {
                B::FootnoteReference::new(config, WithRange(fnote, range), self)?.finish(self, peek)?
            },
            Event::BiberReferences(biber) => {
                B::BiberReferences::new(config, WithRange(biber, range), self)?.finish(self, peek)?
            },
            Event::Url(url) => B::Url::new(config, WithRange(url, range), self)?.finish(self, peek)?,
            Event::InterLink(interlink) => B::InterLink::new(config, WithRange(interlink, range), self)?.finish(self, peek)?,
            Event::Image(img) => B::Image::new(config, WithRange(img, range), self)?.finish(self, peek)?,
            Event::Svg(svg) => B::Svg::new(config, WithRange(svg, range), self)?.finish(self, peek)?,
            Event::Label(label) => B::Label::new(config, WithRange(label, range), self)?.finish(self, peek)?,
            Event::Pdf(pdf) => B::Pdf::new(config, WithRange(pdf, range), self)?.finish(self, peek)?,
            Event::SoftBreak => B::SoftBreak::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::HardBreak => B::HardBreak::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::PageBreak => B::PageBreak::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::TaskListMarker(marker) => {
                B::TaskListMarker::new(config, WithRange(marker, range), self)?.finish(self, peek)?
            },
            Event::TableOfContents => B::TableOfContents::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::Bibliography => B::Bibliography::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::ListOfTables => B::ListOfTables::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::ListOfFigures => B::ListOfFigures::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::ListOfListings => B::ListOfListings::new(config, WithRange((), range), self)?.finish(self, peek)?,
            Event::Appendix => B::Appendix::new(config, WithRange((), range), self)?.finish(self, peek)?,
        }

        Ok(())
    }

    /// Adjust the tag based on the context of this potentially included file within the full
    /// document.
    fn adjust_tag_from_context(&mut self, mut tag: Tag<'a>) -> Tag<'a> {
        if let Tag::Header(header) = &mut tag {
            header.level += self.header_level_adjustment;
        }

        tag
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
    ) -> (&mut Context, &Diagnostics<'a>, &mut Resolver, &mut B, &mut dyn Write) {
        let mut context = None;
        let mut out = None;
        for state in self.stack.iter_mut().rev() {
            if context.is_none() {
                match state {
                    StackElement::Context(ctx, diagnostics, _) => context = Some((ctx, diagnostics)),
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

    pub fn diagnostics(&mut self) -> &Diagnostics<'a> {
        self.top_context().1
    }

    pub fn backend_and_out(&mut self) -> (&Diagnostics<'a>, &mut B, &mut dyn Write) {
        let (_, diagnostics, _, backend, out) = self.top_context();
        (diagnostics, backend, out)
    }

    fn resolve(&mut self, resolve_security: ResolveSecurity, url: &str, range: SourceRange) -> Result<Include> {
        let (context, diagnostics, resolver, _, _) = self.top_context();
        resolver.resolve(resolve_security, context, url, range, diagnostics)
    }
}
