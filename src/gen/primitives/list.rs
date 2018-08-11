use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::Generator;
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

    pub fn gen_list(&mut self, start: Option<usize>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
        println!("start: {:?}", start);
        if let Some(start) = start {
            let start = start as i32 - 1;
            self.enumerate_depth += 1;
            writeln!(out, "\\begin{{enumerate}}")?;
            writeln!(out, "\\setcounter{{enum{}}}{{{}}}", "i".repeat(self.enumerate_depth), start)?;
        } else {
            writeln!(out, "\\begin{{itemize}}")?;
        }

        loop {
            let evt = events.next().unwrap();
            println!("{:?}", evt);
            match evt {
                Event::Start(Tag::Item) => self.gen_item(events, out)?,
                Event::Start(Tag::List(start)) => self.gen_list(start, events, out)?,
                Event::End(Tag::List(_)) => break,
                evt => self.gen.visit_event(evt, events, out)?,
            }
        }

        if start.is_some() {
            writeln!(out, "\\end{{enumerate}}")?;
            self.enumerate_depth -= 1;
        } else {
            writeln!(out, "\\end{{itemize}}")?;
        }
        Ok(())
    }

    fn gen_item(&mut self, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
        write!(out, "\\item ")?;
        loop {
            let evt = events.next().unwrap();
            println!("{:?}", evt);
            match evt {
                Event::Start(Tag::List(start)) => self.gen_list(start, events, out)?,
                Event::End(Tag::Item) => break,
                evt => self.gen.visit_event(evt, events, out)?,
            }
        }
        writeln!(out)
    }
}

