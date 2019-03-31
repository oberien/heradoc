use std::ops::Range;
use std::io::Write;
use std::fs;
use std::iter::Fuse;

use crate::error::{FatalResult, Error, Result};
use crate::frontend::{Frontend, Event as FeEvent, EventKind as FeEventKind, Include as FeInclude};
use crate::generator::Generator;
use crate::generator::event::{Event, Image, Pdf};
use crate::backend::Backend;
use crate::resolve::{Include, Context};
use crate::diagnostics::Input;

pub struct Iter<'a> {
    frontend: Fuse<Frontend<'a>>,
    peek: Option<(Event<'a>, Range<usize>)>,
    last_kind: FeEventKind,
}

impl<'a> Iter<'a> {
    pub fn new(frontend: Frontend<'a>) -> Self {
        Iter {
            frontend: frontend.fuse(),
            peek: None,
            last_kind: FeEventKind::Start,
        }
    }

    /// Retrieves and converts the next event that needs to be handled.
    ///
    /// If it's an include which is handled, it'll be handled internally and the next event will
    /// be returned. If there is some diagnostic error, it'll skip over that event and return
    /// the next one which should be handled.
    pub fn next(&mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> FatalResult<Option<(Event<'a>, Range<usize>)>> {
        if let Some(peek) = self.peek.take() {
            self.last_kind = FeEventKind::from(&peek.0);
            return Ok(Some(peek));
        }
        loop {
            match self.frontend.next() {
                None => return Ok(None),
                Some((event, range)) => {
                    self.last_kind = FeEventKind::from(&event);
                    match self.convert_event(event, range.clone(), gen) {
                        Ok(None) => continue,
                        Ok(Some(event)) => return Ok(Some((event, range))),
                        Err(Error::Diagnostic) => self.skip(gen)?,
                        Err(Error::Fatal(fatal)) => return Err(fatal),
                    }
                },
            }
        }
    }

    pub fn peek(&mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> FatalResult<Option<(&Event<'a>, Range<usize>)>> {
        if let Some((peek, peek_range)) = &self.peek {
            return Ok(Some((peek, peek_range.clone())));
        }
        self.peek = self.next(gen)?;
        Ok(self.peek.as_ref().map(|(peek, range)| (peek, range.clone())))
    }

    /// Skips events until the next one that can be handled again.
    pub fn skip(&mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> FatalResult<()> {
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
            let evt = if let Some((peek, _)) = self.peek.take() {
                peek
            } else {
                self.next(gen)?.0
            };
            match evt {
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
        &mut self, event: FeEvent<'a>, range: Range<usize>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Event<'a>> {
        match event {
            FeEvent::Include(image) => {
                let include = gen.resolve(&image.dst, range.clone())?;
                self.convert_include(include, Some(image), range)
            },
            FeEvent::ResolveInclude(include) => {
                let include = gen.resolve(&include, range.clone())?;
                self.convert_include(include, None, range)
            },
            e => Ok(Some(e.into())),
        }
    }

    fn convert_include(
        &mut self, include: Include, image: Option<FeInclude<'a>>, range: Range<usize>,
    ) -> Result<Event<'a>> {
        match include {
            Include::Command(command) => Ok(command.into()),
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
                Ok(Event::IncludeMarkdown(events))
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
            Include::Pdf(path) => Ok(Event::Pdf(Pdf { path })),
        }
    }
}
