use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::iter::Fuse;
use diagnostic::{Span, Spanned};

use crate::backend::Backend;
use crate::error::{DiagnosticCode, Error, FatalResult, Result};
use crate::frontend::{Event as FeEvent, EventKind as FeEventKind, Frontend, Include as FeInclude, Graphviz};
use crate::generator::event::{Event, Tag, Image, Pdf, Svg};
use crate::generator::Generator;
use crate::resolve::{Include, ResolveSecurity};

pub struct Iter<'a> {
    frontend: Fuse<Frontend<'a>>,
    peek: VecDeque<(Spanned<Event<'a>>, FeEventKind)>,
    /// Contains the kind of the last FeEvent returned from `Self::next()`.
    ///
    /// This is used to `skip` correctly over events when an event couldn't be handled correctly.
    /// For example if this is `Start`, we'll skip until the corresponding `End` event.
    last_kind: FeEventKind,
}

impl<'a> Iter<'a> {
    pub fn new(frontend: Frontend<'a>) -> Self {
        Iter { frontend: frontend.fuse(), peek: VecDeque::new(), last_kind: FeEventKind::Start }
    }

    /// Retrieves and converts the next event that needs to be handled.
    ///
    /// If it's an include which is handled, it'll be handled internally and the next event will
    /// be returned. If there is some diagnostic error, it'll skip over that event and return
    /// the next one which should be handled.
    pub fn next(
        &mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> FatalResult<Option<Spanned<Event<'a>>>> {
        if let Some((peek, kind)) = self.peek.pop_front() {
            self.last_kind = kind;
            return Ok(Some(peek));
        }
        loop {
            match self.frontend.next() {
                None => return Ok(None),
                Some(Spanned { value: event, span }) => {
                    self.last_kind = FeEventKind::from(&event);
                    match self.convert_event(Spanned::new(event, span), gen) {
                        Ok(event) => return Ok(Some(Spanned::new(event, span))),
                        Err(Error::Diagnostic) => self.skip(gen)?,
                        Err(Error::Fatal(fatal)) => return Err(fatal),
                    }
                },
            }
        }
    }

    pub fn peek(
        &mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> FatalResult<Option<Spanned<&Event<'a>>>> {
        if self.peek.is_empty() {
            let old_kind = self.last_kind;
            let peek = match self.next(gen)? {
                Some(peek) => peek,
                None => return Ok(None),
            };
            self.peek.push_front((peek, self.last_kind));
            self.last_kind = old_kind;
        }
        Ok(self.peek.front().map(|(peek, _)| peek.as_ref()))
    }

    /// Skips events until the next one that can be handled again.
    pub fn skip(
        &mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> FatalResult<()> {
        match self.last_kind {
            // continue consuming until end
            FeEventKind::Start => (),
            // TODO: unsure if `End` might just be unrecoverable
            FeEventKind::End
            | FeEventKind::Text
            | FeEventKind::Html
            | FeEventKind::Latex
            | FeEventKind::FootnoteReference
            | FeEventKind::BiberReferences
            | FeEventKind::Url
            | FeEventKind::InterLink
            | FeEventKind::Include
            | FeEventKind::ResolveInclude
            | FeEventKind::Label
            | FeEventKind::SoftBreak
            | FeEventKind::HardBreak
            | FeEventKind::Rule
            | FeEventKind::PageBreak
            | FeEventKind::TaskListMarker
            | FeEventKind::Command => return Ok(()),
        }
        let mut depth = 0;
        loop {
            let evt = self.next(gen)?.unwrap().value;
            match evt {
                Event::Start(_) => depth += 1,
                Event::End(_) if depth > 0 => depth -= 1,
                Event::End(_) => return Ok(()),
                _ => {},
            }
        }
    }

    /// Converts an event, resolving any includes. If the include is handled, returns Ok(None).
    /// If it fails, returns the original event.
    fn convert_event(
        &mut self, event: Spanned<FeEvent<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Event<'a>> {
        let Spanned { value: event, span } = event;
        match event {
            FeEvent::Include(image) => {
                let include = gen.resolve(image.resolve_security, &image.dst, span)?;
                self.convert_include(Spanned::new(include, span), Some(image), gen)
            },
            FeEvent::ResolveInclude(include) => {
                let include = gen.resolve(ResolveSecurity::Default, &include, span)?;
                self.convert_include(Spanned::new(include, span), None, gen)
            },
            e => Ok(e.into()),
        }
    }

    fn convert_include(
        &mut self, Spanned { value: include, span }: Spanned<Include>, image: Option<FeInclude<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Event<'a>> {
        let (label, caption, title, alt_text, scale, width, height) =
            if let Some(FeInclude {
                resolve_security: _,
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
        match include {
            Include::Command(command) => Ok(command.into()),
            Include::Markdown(path, context) => {
                let markdown = fs::read_to_string(&path).map_err(|err| {
                    gen.diagnostics()
                        .error(DiagnosticCode::ErrorReadingIncludedMarkdownFile)
                        .with_error_label(span, "error reading markdown include file")
                        .with_error_label(span, format!("cause: {}", err))
                        .with_note(format!("reading from path {}", path.display()))
                        .emit();
                    Error::Diagnostic
                })?;
                let (fileid, markdown) = gen.diagnostics.add_file(path.display().to_string(), markdown);
                let markdown_span = Span::new(fileid, 0, markdown.len());
                let events = gen.get_events(Spanned::new(markdown, markdown_span), context);
                Ok(Event::IncludeMarkdown(Box::new(events)))
            },
            Include::Image(path) => {
                Ok(Event::Image(Image {
                    label,
                    caption,
                    title,
                    alt_text,
                    path,
                    scale,
                    width,
                    height,
                }))
            },
            Include::Svg(path) => {
                Ok(Event::Svg(Svg {
                    label,
                    caption,
                    title,
                    alt_text,
                    path,
                    scale,
                    width,
                    height,
                }))
            },
            Include::Pdf(path) => Ok(Event::Pdf(Pdf { path })),
            Include::Graphviz(path) => {
                let content = fs::read_to_string(&path).map_err(|err| {
                    gen.diagnostics()
                        .error(DiagnosticCode::ErrorReadingGraphvizFile)
                        .with_error_label(span, "can't read this graphviz file")
                        .with_error_label(span, format!("cause: {}", err))
                        .with_note(format!("reading from path {}", path.display()))
                        .emit();
                    Error::Diagnostic
                })?;
                let tag = Tag::Graphviz(Graphviz {
                    label,
                    caption,
                    scale,
                    width,
                    height,
                });
                self.peek.push_back((Spanned::new(Event::Text(content.into()), span), self.last_kind));
                self.peek.push_back((Spanned::new(Event::End(tag.clone()), span), self.last_kind));
                Ok(Event::Start(tag))
            }
        }
    }
}
