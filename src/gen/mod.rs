use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

#[macro_use]
mod macros;
mod peek;
mod preamble;
mod primitives;

use self::peek::Peek;

pub trait Generator<'a> {
    fn gen(&mut self, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()>;
    fn visit_event(&mut self, event: Event<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()>;
}

pub fn generate<'a>(events: impl IntoIterator<Item = Event<'a>>, mut out: impl Write) -> Result<()> {
    Document::new().gen(&mut events.into_iter().peekable(), &mut out)
}

pub struct Document;

impl<'a> Generator<'a> for Document {
    fn gen(&mut self, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
        // TODO: parse preamble first if it exists
        self.gen_preamble(out)?;
        loop {
            match events.next() {
                Some(evt) => self.visit_event(evt, events, out)?,
                None => break,
            }
        }
        self.gen_epilogue(out)?;
        Ok(())
    }

    fn visit_event(&mut self, event: Event<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
        match event {
            // simple
            Event::Text(text) => primitives::gen_text(&text, out),
            Event::Html(html) => unimplemented!(),
            Event::InlineHtml(html) => unimplemented!(),
            Event::FootnoteReference(fnote) => primitives::gen_footnote_reference(&fnote, out),
            Event::SoftBreak => primitives::gen_soft_break(out),
            Event::HardBreak => primitives::gen_hard_break(out),
            // complex
            Event::Start(Tag::Paragraph) => primitives::gen_par(self, events, out),
            Event::Start(Tag::Rule) => primitives::gen_rule(events, out),
            Event::Start(Tag::Header(level)) => primitives::gen_header(self, level, events, out),
            Event::Start(Tag::BlockQuote) => primitives::gen_block_quote(self, events, out),
            Event::Start(Tag::CodeBlock(lang)) => primitives::gen_code_block(self, &lang, events, out),
            Event::Start(Tag::List(start)) => primitives::ListGenerator::new(self).gen_list(start, events, out),
            Event::Start(Tag::Item) => unreachable!("list should be handled by ListGenerator"),
            Event::Start(Tag::FootnoteDefinition(fnote)) => primitives::gen_footnote_definition(self, &fnote, events, out),
            Event::Start(Tag::Table(align)) => primitives::gen_table(self, align, events, out),
            Event::Start(Tag::TableHead) => primitives::gen_table_head(self, events, out),
            Event::Start(Tag::TableRow) => primitives::gen_table_row(self, events, out),
            Event::Start(Tag::TableCell) => primitives::gen_table_cell(self, events, out),
            Event::Start(Tag::Emphasis) => primitives::gen_emphasized(self, events, out),
            Event::Start(Tag::Strong) => primitives::gen_strong(self, events, out),
            Event::Start(Tag::Code) => primitives::gen_code(self, events, out),
            Event::Start(Tag::Link(dst, title)) => primitives::gen_link(self, &dst, &title, events, out),
            Event::Start(Tag::Image(dst, title)) => primitives::gen_image(self, &dst, &title, events, out),
            Event::End(_) => unreachable!("end should be handled by gen_* functions"),
        }
    }

}

impl Document {
    fn new() -> Self {
        Document
    }

    fn gen_preamble(&mut self, out: &mut impl Write) -> Result<()> {
        // TODO: papersize, documentclass, geometry
        // TODO: itemizespacing
        writeln!(out, "\\documentclass[a4paper]{{scrartcl}}")?;
        writeln!(out, "\\usepackage[utf8]{{inputenc}}")?;
        writeln!(out)?;
        // TODO: include rust highlighting
        // TODO: use minted instead of lstlistings?
        // TODO: lstset
        writeln!(out, "\\usepackage{{listings}}")?;
        writeln!(out, "\\usepackage[usenames, dvipsnames]{{color}}")?;
        writeln!(out, "\\usepackage{{xcolor}}")?;
        writeln!(out, "{}", preamble::lstset)?;
        writeln!(out, "{}", preamble::lstdefineasm)?;
        writeln!(out, "{}", preamble::lstdefinerust)?;
        // TODO: graphicspath
        writeln!(out, "\\usepackage{{graphicx}}")?;
        writeln!(out, "\\usepackage{{hyperref}}")?;
        // TODO: cleveref options
        writeln!(out, "\\usepackage{{cleveref}}")?;
        writeln!(out, "\\usepackage{{refcount}}")?;
        writeln!(out, "\\usepackage{{array}}")?;
        writeln!(out, "{}", preamble::thickhline)?;
        writeln!(out)?;
        writeln!(out, "{}", preamble::aquote)?;
        writeln!(out)?;
        writeln!(out, "\\begin{{document}}")?;
        writeln!(out)?;
        Ok(())
    }

    fn gen_epilogue(&mut self, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}

