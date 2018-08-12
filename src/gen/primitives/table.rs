use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag, Alignment};

use crate::gen::{Generator, State};
use crate::gen::peek::Peek;

pub fn gen_table(gen: &mut impl Generator<'a>, align: Vec<Alignment>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    // TODO: in-cell linebreaks
    // TODO: merging columns
    // TODO: merging rows
    // TODO: easier custom formatting
    write!(state.out, "\\begin{{tabular}}{{|")?;
    for align in align {
        match align {
            Alignment::None | Alignment::Left => write!(state.out, " l |")?,
            Alignment::Center => write!(state.out, " c |")?,
            Alignment::Right => write!(state.out, " r |")?,
        }
    }
    writeln!(state.out, "}}")?;
    writeln!(state.out, "\\hline")?;
    handle_until!(gen, state, Tag::Table(_));
    writeln!(state.out, "\\end{{tabular}}")?;
    Ok(())
}

pub fn gen_table_head(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    handle_until!(gen, state, Tag::TableHead);
    writeln!(state.out, "\\\\ \\thickhline")
}

pub fn gen_table_row(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    handle_until!(gen, state, Tag::TableRow);
    writeln!(state.out, "\\\\ \\hline")
}

pub fn gen_table_cell(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    handle_until!(gen, state, Tag::TableCell);
    if let Event::Start(Tag::TableCell) = state.events.peek().unwrap() {
        write!(state.out, "&")?;
    }
    Ok(())
}

