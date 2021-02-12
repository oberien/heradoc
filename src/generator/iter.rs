use std::collections::VecDeque;
use std::borrow::Cow;
use std::fs;
use std::io::Write;
use std::iter::Fuse;

use crate::backend::Backend;
use crate::diagnostics::Input;
use crate::error::{Error, FatalResult, Result};
use crate::frontend::{Event as FeEvent, EventKind as FeEventKind, Frontend, Include as FeInclude, Graphviz};
use crate::frontend::range::WithRange;
use crate::frontend::rustdoc::{Crate, Rustdoc};
use crate::generator::event::{Event, Tag, Image, Pdf, Svg};
use crate::generator::Generator;
use crate::resolve::{Include, ContextType, ResolveSecurity};

type FeEvents<'a> = dyn Iterator<Item=WithRange<FeEvent<'a>>> + 'a;

pub struct MarkdownIter<'a> {
    frontend: Fuse<Box<FeEvents<'a>>>,
    peek: VecDeque<(WithRange<Event<'a>>, FeEventKind)>,
    /// Contains the kind of the last FeEvent returned from `Self::next()`.
    ///
    /// This is used to `skip` correctly over events when an event couldn't be handled correctly.
    /// For example if this is `Start`, we'll skip until the corresponding `End` event.
    last_kind: FeEventKind,
}

enum IncludeSource<'a> {
    Resolve,
    Image(FeInclude<'a>),
    Synthetic {
        parameter: FeInclude<'a>,
        source: Cow<'a, str>,
    },
}

impl<'a> MarkdownIter<'a> {
    pub fn new(frontend: Frontend<'a>) -> Self {
        let frontend: Box<FeEvents<'a>> = Box::new(frontend);
        MarkdownIter { frontend: frontend.fuse(), peek: VecDeque::new(), last_kind: FeEventKind::Start }
    }

    pub fn with_rustdoc(rustdoc: Rustdoc<'a>) -> Self {
        let frontend: Box<FeEvents<'a>> = Box::new(rustdoc.map(|event| {
            WithRange(event, (0..0).into())
        }));

        MarkdownIter { frontend: frontend.fuse(), peek: VecDeque::new(), last_kind: FeEventKind::Start }
    }

    /// Retrieves and converts the next event that needs to be handled.
    ///
    /// If it's an include which is handled, it'll be handled internally and the next event will
    /// be returned. If there is some diagnostic error, it'll skip over that event and return
    /// the next one which should be handled.
    pub fn next(
        &mut self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> FatalResult<Option<WithRange<Event<'a>>>> {
        if let Some((peek, kind)) = self.peek.pop_front() {
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
            | FeEventKind::InlineHtml
            | FeEventKind::Latex
            | FeEventKind::FootnoteReference
            | FeEventKind::BiberReferences
            | FeEventKind::Url
            | FeEventKind::InterLink
            | FeEventKind::Include
            | FeEventKind::ResolveInclude
            | FeEventKind::SyntheticInclude
            | FeEventKind::Label
            | FeEventKind::SoftBreak
            | FeEventKind::HardBreak
            | FeEventKind::PageBreak
            | FeEventKind::TaskListMarker
            | FeEventKind::Command => return Ok(()),
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
                let include = gen.resolve(image.resolve_security, &image.dst, range)?;
                let source = IncludeSource::Image(image);
                self.convert_include(WithRange(include, range), source, gen)
            },
            FeEvent::ResolveInclude(include) => {
                let include = gen.resolve(ResolveSecurity::Default, &include, range)?;
                self.convert_include(WithRange(include, range), IncludeSource::Resolve, gen)
            },
            FeEvent::SyntheticInclude(include, parameter, source) => {
                let source = IncludeSource::Synthetic { parameter, source };
                self.convert_include(WithRange(include, range), source, gen)
            },
            e => Ok(e.into()),
        }
    }

    fn convert_include(
        &mut self, WithRange(include, range): WithRange<Include>, source: IncludeSource<'a>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Event<'a>> {
        let mut synthetic_source = None;
        let (label, caption, title, alt_text, scale, width, height, adjust_headers) =
            match source {
                IncludeSource::Image(FeInclude {
                    resolve_security: _,
                    label,
                    caption,
                    title,
                    alt_text,
                    dst: _dst,
                    scale,
                    width,
                    height,
                    adjust_headers,
                }) => {
                    (label, caption, title, alt_text, scale, width, height, adjust_headers)
                },
                IncludeSource::Synthetic {
                    parameter: FeInclude {
                        resolve_security: _,
                        label,
                        caption,
                        title,
                        alt_text,
                        dst: _dst,
                        scale,
                        width,
                        height,
                        adjust_headers,
                    }, 
                    source,
                } => {
                    synthetic_source = Some(source);
                    (label, caption, title, alt_text, scale, width, height, adjust_headers)
                },
                IncludeSource::Resolve => {
                    Default::default()
                },
            };
        match include {
            Include::Command(command) => Ok(command.into()),
            Include::Markdown(path, context) => {
                let markdown = if let Some(markdown) = synthetic_source {
                    markdown
                } else {
                    let source = fs::read_to_string(&path).map_err(|err| {
                        gen.diagnostics()
                            .error("error reading markdown include file")
                            .with_error_section(range, "in this include")
                            .error(format!("cause: {}", err))
                            .note(format!("reading from path {}", path.display()))
                            .emit();
                        Error::Diagnostic
                    })?;
                    Cow::Owned(source)
                };
                let input = match context.typ() {
                    ContextType::Remote => Input::Url(context.url().clone()),
                    ContextType::LocalRelative | ContextType::LocalAbsolute => {
                        Input::File(path)
                    },
                };
                let mut events = gen.get_events(markdown, context, input);
                events.adjust_header_levels = adjust_headers;
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
            Include::Rustdoc(path) => {
                let events = gen.get_rustdoc(Crate::Local(path))?;
                Ok(Event::IncludeRustdoc(Box::new(events)))
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
                        .error("can't read graphviz file")
                        .with_error_section(range, "in this include")
                        .error(format!("cause: {}", err))
                        .note(format!("reading from path {}", path.display()))
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
                self.peek.push_back((WithRange(Event::Text(content.into()), range), self.last_kind));
                self.peek.push_back((WithRange(Event::End(tag.clone()), range), self.last_kind));
                Ok(Event::Start(tag))
            }
        }
    }
}
