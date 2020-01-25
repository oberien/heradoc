use std::fs;
use std::io::Write;
use std::iter::Fuse;

use crate::backend::Backend;
use crate::diagnostics::Input;
use crate::error::{Error, FatalResult, Result};
use crate::frontend::{Event as FeEvent, EventKind as FeEventKind, Frontend, Include as FeInclude};
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, Image, Pdf, Svg};
use crate::generator::Generator;
use crate::resolve::{Include, ContextType};

pub struct Iter<'a> {
    frontend: Fuse<Frontend<'a>>,
    peek: Option<(WithRange<Event<'a>>, FeEventKind)>,
    /// Contains the kind of the last FeEvent returned from `Self::next()`.
    ///
    /// This is used to `skip` correctly over events when an event couldn't be handled correctly.
    /// For example if this is `Start`, we'll skip until the corresponding `End` event.
    last_kind: FeEventKind,
}

impl<'a> Iter<'a> {
    pub fn new(frontend: Frontend<'a>) -> Self {
        Iter { frontend: frontend.fuse(), peek: None, last_kind: FeEventKind::Start }
    }

    /// Retrieves and converts the next event that needs to be handled.
    ///
    /// If it's an include which is handled, it'll be handled internally and the next event will
    /// be returned. If there is some diagnostic error, it'll skip over that event and return
    /// the next one which should be handled.
    pub fn next(
        &mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> FatalResult<Option<WithRange<Event<'a>>>> {
        if let Some((peek, kind)) = self.peek.take() {
            self.last_kind = kind;
            return Ok(Some(peek));
        }
        loop {
            match self.frontend.next() {
                None => return Ok(None),
                Some(WithRange(event, range)) => {
                    self.last_kind = FeEventKind::from(&event);
                    match self.convert_event(WithRange(event, range), gen) {
                        Ok(event) => return Ok(Some(WithRange(event, range))),
                        Err(Error::Diagnostic) => self.skip(gen)?,
                        Err(Error::Fatal(fatal)) => return Err(fatal),
                    }
                },
            }
        }
    }

    pub fn peek(
        &mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> FatalResult<Option<WithRange<&Event<'a>>>> {
        if self.peek.is_none() {
            let old_kind = self.last_kind;
            let peek = match self.next(gen)? {
                Some(peek) => peek,
                None => return Ok(None),
            };
            self.peek = Some((peek, self.last_kind));
            self.last_kind = old_kind;
        }
        Ok(self.peek.as_ref().map(|(peek, _)| peek.as_ref()))
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
            let evt = self.next(gen)?.unwrap().0;
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
        &mut self, event: WithRange<FeEvent<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Event<'a>> {
        let WithRange(event, range) = event;
        match event {
            FeEvent::Include(image) => {
                let include = gen.resolve(&image.dst, range)?;
                self.convert_include(WithRange(include, range), Some(image), gen)
            },
            FeEvent::ResolveInclude(include) => {
                let include = gen.resolve(&include, range)?;
                self.convert_include(WithRange(include, range), None, gen)
            },
            e => Ok(e.into()),
        }
    }

    fn convert_include(
        &mut self, WithRange(include, range): WithRange<Include>, image: Option<FeInclude<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Event<'a>> {
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
        match include {
            Include::Command(command) => Ok(command.into()),
            Include::Markdown(path, context) => {
                let markdown = fs::read_to_string(&path).map_err(|err| {
                    gen.diagnostics()
                        .error("error reading markdown include file")
                        .with_error_section(range, "in this include")
                        .error(format!("cause: {}", err))
                        .note(format!("reading from path {}", path.display()))
                        .emit();
                    Error::Diagnostic
                })?;
                let input = match context.typ() {
                    ContextType::Remote => Input::Url(context.url().clone()),
                    ContextType::LocalRelative | ContextType::LocalAbsolute => {
                        Input::File(path)
                    },
                };
                let events = gen.get_events(markdown, context, input);
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
        }
    }
}
