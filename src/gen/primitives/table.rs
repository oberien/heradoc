use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag, Alignment};

use crate::gen::Generator;
use crate::gen::peek::Peek;

pub fn gen_table(gen: &mut impl Generator<'a>, align: Vec<Alignment>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    // TODO: in-cell linebreaks
    // TODO: merging columns
    // TODO: merging rows
    // TODO: easier custom formatting
    write!(out, "\\begin{{tabular}}{{|")?;
    for align in align {
        match align {
            Alignment::None | Alignment::Left => write!(out, " l |")?,
            Alignment::Center => write!(out, " c |")?,
            Alignment::Right => write!(out, " r |")?,
        }
    }
    writeln!(out, "}}")?;
    writeln!(out, "\\hline")?;
    handle_until!(gen, events, out, Tag::Table(_));
    writeln!(out, "\\end{{tabular}}")?;
    Ok(())
}

pub fn gen_table_head(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    handle_until!(gen, events, out, Tag::TableHead);
    writeln!(out, "\\\\ \\thickhline")
}

pub fn gen_table_row(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    handle_until!(gen, events, out, Tag::TableRow);
    writeln!(out, "\\\\ \\hline")
}

pub fn gen_table_cell(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    handle_until!(gen, events, out, Tag::TableCell);
    if let Event::Start(Tag::TableCell) = events.peek().unwrap() {
        write!(out, "&")?;
    }
    Ok(())
}

