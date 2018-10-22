use std::io::Write;
use std::fs;

use pulldown_cmark::{Parser as CmarkParser, OPTION_ENABLE_FOOTNOTES, OPTION_ENABLE_TABLES};
use typed_arena::Arena;

use crate::backend::{Backend};
use crate::frontend::{Frontend, Event as FeEvent, Link as FeLink, Image as FeImage};
use crate::config::Config;
use crate::resolve::{Resolver, Context, Include, Command};
use crate::ext::StrExt;

mod concat;
mod stack;
mod primitive;
mod code_gen_units;
pub mod event;
mod error;

pub use self::stack::Stack;
pub use self::primitive::PrimitiveGenerator;
pub use self::error::{Error, Result};

use self::concat::Concat;
use self::code_gen_units::StackElement;
use self::event::{Event, Image, Pdf};

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    arena: &'a Arena<String>,
    doc: B,
    prim: PrimitiveGenerator<'a, B, W>,
    resolver: Resolver,
}

pub trait Positioned {
    /// Get a relative position indication, like a simple offset.
    ///
    /// The position information is used to enrich any occurring error.
    fn current_position(&self) -> usize;
}

impl<'a, B: Backend<'a>> Positioned for Frontend<'a, B> {
    fn current_position(&self) -> usize {
        self.inner().get_offset()
    }
}

impl<'a, B: Backend<'a>, W: Write> Generator<'a, B, W> {
    pub fn new(cfg: &'a Config, doc: B, default_out: W, arena: &'a Arena<String>) -> Self {
        let prim = PrimitiveGenerator::new(cfg, default_out, Context::LocalRelative(cfg.input_dir.clone()));
        Generator {
            arena,
            doc,
            prim,
            resolver: Resolver::new(cfg.input_dir.clone()),
        }
    }

    fn get_events(&mut self, markdown: String) -> (impl Iterator<Item = FeEvent<'a>> + Positioned, &'a str) {
        let markdown = self.arena.alloc(markdown);
        let parser: Frontend<'_, B> = Frontend::new(self.prim.cfg, CmarkParser::new_with_broken_link_callback(
            markdown,
            OPTION_ENABLE_FOOTNOTES | OPTION_ENABLE_TABLES,
            Some(&refsolve)
        ));
        (Concat::new(parser), markdown)
    }

    pub fn generate(&mut self, markdown: String) -> Result<()> {
        let (events, markdown) = self.get_events(markdown);
        self.generate_with_events(events).map_err(|err| err.with_source_span(markdown))
    }

    pub fn generate_with_events(&mut self, events: impl Iterator<Item = FeEvent<'a>> + Positioned) -> Result<()> {
        self.doc.gen_preamble(self.prim.cfg, &mut self.prim.default_out)?;
        self.generate_body(events)?;
        assert!(self.prim.pop().is_none());
        self.doc.gen_epilogue(self.prim.cfg, &mut self.prim.default_out)?;
        Ok(())
    }

    fn convert_event(&mut self, event: FeEvent<'a>) -> Result<Option<Event<'a>>> {
        match event {
            FeEvent::Link(FeLink::Url(link)) => {
                if link.starts_with_ignore_ascii_case("include ") {
                    let include = self.resolve(&link[8..])?;
                    assert!(self.handle_include(include, None)?.is_none());
                    Ok(None)
                } else if link.eq_ignore_ascii_case("toc")
                    || link.eq_ignore_ascii_case("tableofcontents")
                    || link.eq_ignore_ascii_case("bibliography")
                    || link.eq_ignore_ascii_case("references")
                    || link.eq_ignore_ascii_case("listoftables")
                    || link.eq_ignore_ascii_case("listoffigures")
                    || link.eq_ignore_ascii_case("listoflistings")
                {
                    let include = self.resolve(&format!("//{}", link))?;
                    Ok(self.handle_include(include, None).unwrap())
                } else {
                    Ok(Some(FeEvent::Link(FeLink::Url(link)).into()))
                }
            }
            FeEvent::Image(image) => {
                let include = self.resolve(&image.dst)?;
                Ok(self.handle_include(include, Some(image))?)
            }
            e => Ok(Some(e.into())),
        }
    }

    pub fn generate_body(&mut self, mut events: impl Iterator<Item = FeEvent<'a>> + Positioned) -> Result<()> {
        let mut begin;
        let mut peek = loop {
            begin = events.current_position();
            match events.next() {
                None => break None,
                Some(e) => match self.convert_event(e).map_err(|err| err.with_span(begin..events.current_position()))? {
                    None => continue,
                    Some(e) => break Some(e),
                }
            }
        };
        while let Some(event) = peek {
            peek = loop {
                begin = events.current_position();
                match events.next() {
                    None => break None,
                    Some(e) => match self.convert_event(e).map_err(|err| err.with_span(begin..events.current_position()))? {
                        None => continue,
                        Some(e) => break Some(e),
                    }
                }
            };
            self.prim.visit_event(event, peek.as_ref())
                .map_err(|err| Error::from(err).with_span(begin..events.current_position()))?;
        }
        match self.prim.pop() {
            Some(StackElement::Context(_)) => (),
            element => panic!("Expected context as stack element after body generation is finished, got {:?}", element),
        }
        Ok(())
    }

    fn resolve(&mut self, url: &str) -> Result<Include> {
        let context = self.prim.iter_stack().find_map(|se| match se {
            StackElement::Context(context) => Some(context),
            _ => None,
        }).expect("no Context???");
        self.resolver.resolve(context, url)
            .map_err(Error::from)
    }

    fn handle_include(&mut self, include: Include, image: Option<FeImage<'a>>) -> Result<Option<Event<'a>>> {
        match include {
            Include::Command(Command::Toc) => Ok(Some(Event::TableOfContents)),
            Include::Command(Command::Bibliography) => Ok(Some(Event::Bibliography)),
            Include::Command(Command::ListOfTables) => Ok(Some(Event::ListOfTables)),
            Include::Command(Command::ListOfFigures) => Ok(Some(Event::ListOfFigures)),
            Include::Command(Command::ListOfListings) => Ok(Some(Event::ListOfListings)),
            Include::Markdown(path, context) => {
                let markdown = fs::read_to_string(path)?;
                let (events, _markdown) = self.get_events(markdown);
                self.prim.push(StackElement::Context(context));
                // TODO: change source, we must supply the error information from the other now.
                self.generate_body(events)?;
                Ok(None)
            }
            Include::Image(path) => {
                let image = image.unwrap();
                Ok(Some(Event::Image(Image {
                    path,
                    caption: image.caption,
                    width: image.width,
                    height: image.height,
                })))
            }
            Include::Pdf(path) => Ok(Some(Event::Pdf(Pdf { path }))),
        }

    }
}

fn refsolve(a: &str, b: &str) -> Option<(String, String)> {
    // pass everything, it's handled in the frontend::refs implementation
    Some((a.to_string(), b.to_string()))
}

