use std::fs;
use std::io::Write;
use std::ops::Range;

use typed_arena::Arena;

use crate::backend::{Backend, MediumCodeGenUnit};
use crate::config::{Config, FileOrStdio};
use crate::frontend::{Event as FeEvent, EventKind as FeEventKind, Frontend, Include as FeInclude};
use crate::resolve::{Context, Include, Resolver};
use crate::diagnostics::{Diagnostics, Input};

mod code_gen_units;
pub mod event;
mod stack;

pub use self::stack::Stack;

use self::code_gen_units::StackElement;
use self::event::{Event, EventKind, Image, Pdf};
use crate::error::{FatalResult, Result, Error};

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    arena: &'a Arena<String>,
    doc: B,
    cfg: &'a Config,
    default_out: W,
    stack: Vec<StackElement<'a, B>>,
    resolver: Resolver,
    template: Option<String>,
}

pub struct Events<'a, I: Iterator<Item = (FeEvent<'a>, Range<usize>)>> {
    events: I,
    diagnostics: Diagnostics<'a>,
    context: Context,
}

impl<'a, B: Backend<'a>, W: Write> Generator<'a, B, W> {
    pub fn new(cfg: &'a Config, doc: B, default_out: W, arena: &'a Arena<String>) -> Self {
        let template = cfg
            .template
            .as_ref()
            .map(|path| fs::read_to_string(path).expect("can't read template"));
        Generator {
            arena,
            doc,
            cfg,
            default_out,
            stack: Vec::new(),
            resolver: Resolver::new(cfg.input_dir.clone(), cfg.temp_dir.clone()),
            template,
        }
    }

    pub fn get_events(&mut self, markdown: String, context: Context, input: Input) -> Events<'a, impl Iterator<Item = (FeEvent<'a>, Range<usize>)>> {
        let markdown = self.arena.alloc(markdown);
        let diagnostics = Diagnostics::new(markdown, input);
        let parser: Frontend<'_, B> = Frontend::new(self.cfg, markdown, diagnostics.clone());
        Events {
            events: parser,
            diagnostics,
            context,
        }
    }

    pub fn generate(&mut self, markdown: String) -> FatalResult<()> {
        let context = Context::LocalRelative(self.cfg.input_dir.clone());
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
            self.doc.gen_preamble(self.cfg, &mut self.default_out)?;
            self.generate_body(events)?;
            assert!(self.stack.pop().is_none());
            self.doc.gen_epilogue(self.cfg, &mut self.default_out)?;
        }
        Ok(())
    }

    pub fn generate_body(&mut self, events: Events<'a, impl Iterator<Item = (FeEvent<'a>, Range<usize>)>>) -> FatalResult<()> {
        self.stack.push(StackElement::Context(events.context, events.diagnostics));
        let mut events = events.events.fuse();

        let (mut peek, mut peek_range) = self.next();
        while let Some(event) = peek {
            let (p, pr) = self.next();
            let range = peek_range;
            peek = p;
            peek_range = pr;
            let kind = FeEventKind::from(&event);
            match self.visit_event(event, range, peek.as_ref(), peek_range) {
                Ok(()) => {},
                Err((Error::Diagnostic, event)) => self.skip_gen(event),
                Err((Error::Fatal(fatal), _)) => return Err(fatal),
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

    /// Retrieves and converts the next event that needs to be handled.
    ///
    /// If it's an include which is handled, it'll be handled internally and the next event will
    /// be returned. If there is some diagnostic error, it'll skip over that event and return
    /// the next one which should be handled.
    fn next(&mut self, events: &mut impl Iterator<Item = (FeEvent<'a>, Range<usize>)>) -> FatalResult<(Event<'a>, Range<usize>)> {
        loop {
            match events.next() {
                None => return (None, Range { start: 0, end: 0 }),
                Some((event, range)) => match self.convert_event(event, range.clone()) {
                    Ok(None) => continue,
                    Ok(Some(event)) => return (Some(event), range),
                    Err((Error::Diagnostic, event)) => self.skip_fe(event)?,
                    Err((Error::Fatal(fatal), _)) => return Err(fatal),
                },
            }
        }
    }

    /// Skips events until the next one that can be handled again. Argument is the event that
    /// produced a diagnostic during handling.
    fn skip_fe(&mut self, kind: FeEventKind) -> FatalResult<()> {
        match event {
            // continue consuming until end
            FeEventKind::Start => (),
            // TODO: unsure if `End` might just be unrecoverable
            FeEventKind::End
            | FeEventKind::Text
            | FeEventKind::Html
            | FeEventKind::InlineHtml
            | FeEventKind::Latex
            | FeEventKind::FootnoteReference
            | FeEventKind::BiberReferences
            | FeEventKind::Url
            | FeEventKind::InterLink
            | FeEventKind::Include
            | FeEventKind::Label
            | FeEventKind::SoftBreak
            | FeEventKind::HardBreak
            | FeEventKind::TaskListMarker
            | FeEventKind::Command
            | FeEventKind::ResolveInclude => return Ok(()),
        }
        let mut depth = 0;
        loop {
            match self.next()?.0 {
                FeEvent::Start(_) => depth += 1,
                FeEvent::End(_) if depth > 0 => depth -= 1,
                FeEvent::End(_) => return Ok(()),
                _ => {},
            }
        }
    }

    /// Same as [`skip_fe`], but with generator::Event instead of frontend::Event.
    fn skip_gen(&mut self, kind: EventKind) {
        match kind {
            EventKind::Start => (),
            // TODO: unsure if `End` might just be unrecoverable
            EventKind::End
            | EventKind::Text
            | EventKind::Html
            | EventKind::InlineHtml
            | EventKind::Latex
            | EventKind::FootnoteReference
            | EventKind::BiberReferences
            | EventKind::Url
            | EventKind::InterLink
            | EventKind::Image
            | EventKind::Label
            | EventKind::Pdf
            | EventKind::SoftBreak
            | EventKind::HardBreak
            | EventKind::TaskListMarker
            | EventKind::TableOfContents
            | EventKind::Bibliography
            | EventKind::ListOfTables
            | EventKind::ListOfFigures
            | EventKind::ListOfListings
            | EventKind::Appendix => return Ok(()),
        }
        let mut depth = 0;
        loop {
            match self.next()?.0 {
                FeEvent::Start(_) => depth += 1,
                FeEvent::End(_) if depth > 0 => depth -= 1,
                FeEvent::End(_) => return Ok(()),
                _ => {},
            }
        }
    }

    /// Converts an event, resolving any includes. If the include is handled, returns Ok(None).
    /// If it fails, returns the original event.
    fn convert_event(
        &mut self, event: FeEvent<'a>, range: Range<usize>
    ) -> std::result::Result<Option<Event<'a>>, (Error, FeEvent<'a>)> {
        match event {
            FeEvent::Include(image) => {
                let include = self.resolve(&image.dst, range.clone())?;
                Ok(self.handle_include(include, Some(image), range)?)
            },
            FeEvent::ResolveInclude(include) => {
                let include = self.resolve(&include, range.clone())?;
                Ok(self.handle_include(include, None, range)?)
            },
            e => Ok(Some(e.into())),
        }
    }

    pub fn visit_event(
        &mut self, event: Event<'a>, range: Range<usize>, peek: Option<&Event<'a>>,
        peek_range: Range<usize>,
    ) -> Result<()> {
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
            Some(Event::Latex(latex)) => B::Latex::gen(latex, &mut stack)?,
            Some(Event::FootnoteReference(fnote)) => B::FootnoteReference::gen(fnote, &mut stack)?,
            Some(Event::BiberReferences(biber)) => B::BiberReferences::gen(biber, &mut stack)?,
            Some(Event::Url(url)) => B::Url::gen(url, &mut stack)?,
            Some(Event::InterLink(interlink)) => B::InterLink::gen(interlink, &mut stack)?,
            Some(Event::Image(img)) => B::Image::gen(img, &mut stack)?,
            Some(Event::Label(label)) => B::Label::gen(label, &mut stack)?,
            Some(Event::Pdf(pdf)) => B::Pdf::gen(pdf, &mut stack)?,
            Some(Event::SoftBreak) => B::SoftBreak::gen((), &mut stack)?,
            Some(Event::HardBreak) => B::HardBreak::gen((), &mut stack)?,
            Some(Event::TaskListMarker(marker)) => B::TaskListMarker::gen(marker, &mut stack)?,
            Some(Event::TableOfContents) => B::TableOfContents::gen((), &mut stack)?,
            Some(Event::Bibliography) => B::Bibliography::gen((), &mut stack)?,
            Some(Event::ListOfTables) => B::ListOfTables::gen((), &mut stack)?,
            Some(Event::ListOfFigures) => B::ListOfFigures::gen((), &mut stack)?,
            Some(Event::ListOfListings) => B::ListOfListings::gen((), &mut stack)?,
            Some(Event::Appendix) => B::Appendix::gen((), &mut stack)?,
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

    pub fn diagnostics(&mut self) -> &mut Diagnostics<'a> {
        self.stack.iter_mut().rev()
            .filter_map(|state| match state {
                StackElement::Context(_, diagnostics) => Some(diagnostics),
                _ => None,
            }).next().unwrap()
    }

    fn resolve(&mut self, url: &str, range: Range<usize>) -> Result<Include> {
        let context = self
            .iter_stack()
            .find_map(|se| match se {
                StackElement::Context(context, _) => Some(context),
                _ => None,
            })
            .expect("no Context???");
        self.resolver.resolve(context, url, range, self.diagnostics())
    }

    /// Possibly returns an event. If it doesn't return an event, the include was already handled
    /// internally by this function.
    fn handle_include(
        &mut self, include: Include, image: Option<FeInclude<'a>>, range: Range<usize>,
    ) -> Result<Option<Event<'a>>> {
        match include {
            Include::Command(command) => Ok(Some(command.into())),
            Include::Markdown(path, context) => {
                let markdown = fs::read_to_string(&path)
                    .map_err(|err| {
                        self.diagnostics()
                            .error("error reading markdown include file")
                            .with_section(&range, "in this include")
                            .error(format!("cause: {}", err))
                            .emit();
                        Error::Diagnostic
                    })?;
                let input = match &context {
                    Context::Remote(url) => Input::Url(url.clone()),
                    Context::LocalRelative(_)
                    | Context::LocalAbsolute(_) => Input::File(path.clone()),
                };
                let events = self.get_events(markdown, context, input);
                self.generate_body(events);
                Ok(None)
            },
            Include::Image(path) => {
                let (label, caption, title, alt_text, scale, width, height) =
                    if let Some(FeInclude {
                        label,
                        caption,
                        title,
                        alt_text,
                        dst: _dst,
                        scale,
                        width,
                        height,
                    }) = image
                    {
                        (label, caption, title, alt_text, scale, width, height)
                    } else {
                        Default::default()
                    };
                Ok(Some(Event::Image(Image {
                    label,
                    caption,
                    title,
                    alt_text,
                    path,
                    scale,
                    width,
                    height,
                })))
            },
            Include::Pdf(path) => Ok(Some(Event::Pdf(Pdf { path }))),
        }
    }
}
