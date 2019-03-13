use std::io::{Write, Result};
use std::fs;

use typed_arena::Arena;

use crate::backend::{Backend};
use crate::frontend::{Frontend, Event as FeEvent, Include as FeInclude};
use crate::config::Config;
use crate::resolve::{Resolver, Context, Include};

mod stack;
mod primitive;
mod code_gen_units;
pub mod event;

pub use self::stack::Stack;
pub use self::primitive::PrimitiveGenerator;

use self::code_gen_units::StackElement;
use self::event::{Event, Image, Pdf};

pub struct Generator<'a, B: Backend<'a>, W: Write> {
    arena: &'a Arena<String>,
    doc: B,
    prim: PrimitiveGenerator<'a, B, W>,
    resolver: Resolver,
    template: Option<String>,
}

impl<'a, B: Backend<'a>, W: Write> Generator<'a, B, W> {
    pub fn new(cfg: &'a Config, doc: B, default_out: W, arena: &'a Arena<String>) -> Self {
        let prim = PrimitiveGenerator::new(cfg, default_out, Context::LocalRelative(cfg.input_dir.clone()));
        let template = cfg.template.as_ref().map(|path| {
            fs::read_to_string(path)
                .expect("can't read template")
        });
        Generator {
            arena,
            doc,
            prim,
            resolver: Resolver::new(cfg.input_dir.clone(), cfg.temp_dir.clone()),
            template,
        }
    }

    pub fn get_events(&mut self, markdown: String) -> impl Iterator<Item = FeEvent<'a>> {
        let markdown = self.arena.alloc(markdown);
        let parser: Frontend<'_, B> = Frontend::new(self.prim.cfg, markdown);
        parser.peekable()
    }

    pub fn generate(&mut self, markdown: String) -> Result<()> {
        let events = self.get_events(markdown);
        if let Some(template) = self.template.take() {
            let body_index = template.find("\nHERADOCBODY\n")
                .expect("HERADOCBODY not found in template on");
            self.prim.get_out().write_all(&template.as_bytes()[..body_index])?;
            self.generate_body(events)?;
            self.prim.get_out().write_all(&template.as_bytes()[body_index + "\nHERADOCBODY\n".len()..])?;
        } else {
            self.generate_with_events(events)?;
        }
        Ok(())
    }

    pub fn generate_with_events(&mut self, events: impl Iterator<Item = FeEvent<'a>>) -> Result<()> {
        self.doc.gen_preamble(self.prim.cfg, &mut self.prim.default_out)?;
        self.generate_body(events)?;
        assert!(self.prim.pop().is_none());
        self.doc.gen_epilogue(self.prim.cfg, &mut self.prim.default_out)?;
        Ok(())
    }

    fn convert_event(&mut self, event: FeEvent<'a>) -> Result<Option<Event<'a>>> {
        match event {
            FeEvent::Include(image) => {
                let include = self.resolve(&image.dst)?;
                Ok(self.handle_include(include, Some(image))?)
            }
            FeEvent::ResolveInclude(include) => {
                let include = self.resolve(&include)?;
                Ok(self.handle_include(include, None)?)
            }
            e => Ok(Some(e.into())),
        }
    }

    pub fn generate_body(&mut self, events: impl Iterator<Item = FeEvent<'a>>) -> Result<()> {
        let mut events = events.fuse();
        let mut peek = loop {
            match events.next() {
                None => break None,
                Some(e) => match self.convert_event(e)? {
                    None => continue,
                    Some(e) => break Some(e),
                }
            }
        };
        while let Some(event) = peek {
            peek = loop {
                match events.next() {
                    None => break None,
                    Some(e) => match self.convert_event(e)? {
                        None => continue,
                        Some(e) => break Some(e),
                    }
                }
            };
            self.prim.visit_event(event, peek.as_ref())?;
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
    }

    fn handle_include(&mut self, include: Include, image: Option<FeInclude<'a>>) -> Result<Option<Event<'a>>> {
        match include {
            Include::Command(command) => Ok(Some(command.into())),
            Include::Markdown(path, context) => {
                let markdown = fs::read_to_string(path)?;
                let events = self.get_events(markdown);
                self.prim.push(StackElement::Context(context));
                self.generate_body(events)?;
                Ok(None)
            }
            Include::Image(path) => {
                let (label, caption, title, alt_text, scale, width, height) =
                    if let Some(FeInclude { label, caption, title, alt_text, dst: _dst, scale, width, height }) = image {
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
            }
            Include::Pdf(path) => Ok(Some(Event::Pdf(Pdf { path }))),
        }

    }
}

