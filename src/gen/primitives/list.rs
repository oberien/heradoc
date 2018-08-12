use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::{Generator, State};
use crate::gen::peek::Peek;

pub struct ListGenerator<'b, G: for<'a> Generator<'a> + 'b> {
    gen: &'b mut G,
    enumerate_depth: usize,
}

impl<'b, G: for<'a> Generator<'a> + 'b> ListGenerator<'b, G> {
    pub fn new(gen: &'b mut G) -> Self {
        ListGenerator {
            gen,
            enumerate_depth: 0,
        }
    }

    pub fn gen_list(&mut self, start: Option<usize>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
        if let Some(start) = start {
            let start = start as i32 - 1;
            self.enumerate_depth += 1;
            writeln!(state.out, "\\begin{{enumerate}}")?;
            writeln!(state.out, "\\setcounter{{enum{}}}{{{}}}", "i".repeat(self.enumerate_depth), start)?;
        } else {
            writeln!(state.out, "\\begin{{itemize}}")?;
        }

        loop {
            let evt = state.events.next().unwrap();
            match evt {
                Event::Start(Tag::Item) => self.gen_item(state)?,
                Event::Start(Tag::List(start)) => self.gen_list(start, state)?,
                Event::End(Tag::List(_)) => break,
                evt => self.gen.visit_event(evt, state)?,
            }
        }

        if start.is_some() {
            writeln!(state.out, "\\end{{enumerate}}")?;
            self.enumerate_depth -= 1;
        } else {
            writeln!(state.out, "\\end{{itemize}}")?;
        }
        Ok(())
    }

    fn gen_item(&mut self, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
        write!(state.out, "\\item ")?;
        loop {
            let evt = state.events.next().unwrap();
            match evt {
                Event::Start(Tag::List(start)) => self.gen_list(start, state)?,
                Event::End(Tag::Item) => break,
                evt => self.gen.visit_event(evt, state)?,
            }
        }
        writeln!(state.out)
    }
}

