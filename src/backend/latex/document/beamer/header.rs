use std::io::{Result, Write};

use crate::backend::{CodeGenUnit, Backend};
use crate::generator::{PrimitiveGenerator, Stack};
use crate::config::Config;
use crate::generator::StackElement;
use crate::generator::event::{Event, Header, Tag};

/// A special beamer header generator.
///
/// Since subsubsections are discouraged in presentations anyways, we use these to construct
/// frames. Each frame opens an environment that also needs to be closed. Luckily, we can intercept
/// the end of a header where we push a new stack element to mark the open frame. Next time we
/// encounter a header, we walk the stack to close this. Also, we need to close the last frame when
/// the document ends.
#[derive(Debug)]
pub struct BeamerHeaderGen {
    label: String,
    frame: FrameState,
}

#[derive(Debug, PartialEq, Eq)]
enum FrameState {
    None,
    Begin,
    Marker,
}

impl BeamerHeaderGen {
    const MAGIC_HEADER: Header = Header { level: i32::max_value() };
}

impl<'a> CodeGenUnit<'a, Header> for BeamerHeaderGen {
    fn new(_cfg: &'a Config, header: Header, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        // Is there a residue frame header on top? That should be the magic marker one.
        // `take(1).last()` avoids extending the borrow on gen, `last` takes by value.
        if let Some(StackElement::Header(_)) = gen.iter_stack().take(1).last() {
            // Create the fitting end event.
            let event = Event::End(Tag::Header(Self::MAGIC_HEADER));

            // This should pop the header.
            gen.visit_event(event, None)?;

            if let Some(StackElement::Header(_)) = gen.iter_stack().last() {
                panic!("Frame unexpectedly not closed");
            }
        }

        assert!(header.level > 0);
        let frame = if header.level < 3 {
            write!(gen.get_out(), "\\{}section{{", "sub".repeat(header.level as usize - 1))?;
            FrameState::None
        } else if header.level == Self::MAGIC_HEADER.level {
            // Oh look, the magic marker value.
            FrameState::Marker
        } else {
            // Treat this as a new slide.
            // Mark all slides as fragile, this is slower but we can use verbatim etc.
            write!(gen.get_out(), "\\begin{{frame}}[fragile]{{")?;
            FrameState::Begin
        };

        Ok(BeamerHeaderGen {
            label: String::with_capacity(100),
            frame,
        })
    }

    fn intercept_event<'b>(&mut self, _stack: &mut Stack<'a, 'b, impl Backend<'a>, impl Write>, e: Event<'a>) -> Result<Option<Event<'a>>> {
        match &e {
            Event::Text(text) => self.label.extend(text.chars().map(|c| match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
                _ => '-',
            })),
            _ => (),
        }
        Ok(Some(e))
    }

    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        // On marker, just write the end.
        if self.frame == FrameState::Marker {
            // This should be the finish event or document end?.
            return write!(gen.get_out(), "\\end{{frame}}\n")
        }


        writeln!(gen.get_out(), "}}\\label{{sec:{}}}\n", self.label)?;
        
        if self.frame == FrameState::Begin {
            // Create another header start, with the marker value as level.
            let event = Event::Start(Tag::Header(Self::MAGIC_HEADER));

            // We push ourselves again, this must succeed.
            gen.visit_event(event, None).unwrap();
        }

        Ok(())
    }
}

