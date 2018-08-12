use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::{Generator, State, primitives};
use crate::gen::peek::Peek;

mod preamble;

pub struct Article;

impl<'a> Generator<'a> for Article {
    fn gen(&mut self, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
        // TODO: parse preamble first if it exists
        self.gen_preamble(&mut state.out)?;
        loop {
            match state.events.next() {
                Some(evt) => self.visit_event(evt, state)?,
                None => break,
            }
        }
        self.gen_epilogue(&mut state.out)?;
        Ok(())
    }

    fn visit_event(&mut self, event: Event<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
        match event {
            // simple
            Event::Text(text) => primitives::gen_text(&text, state),
            Event::Html(html) => unimplemented!(),
            Event::InlineHtml(html) => unimplemented!(),
            Event::FootnoteReference(fnote) => primitives::gen_footnote_reference(&fnote, state),
            Event::SoftBreak => primitives::gen_soft_break(state),
            Event::HardBreak => primitives::gen_hard_break(state),
            // complex
            Event::Start(Tag::Paragraph) => primitives::gen_par(self, state),
            Event::Start(Tag::Rule) => primitives::gen_rule(state),
            Event::Start(Tag::Header(level)) => primitives::gen_header(self, level, state),
            Event::Start(Tag::BlockQuote) => primitives::gen_block_quote(self, state),
            Event::Start(Tag::CodeBlock(lang)) => primitives::gen_code_block(self, &lang, state),
            Event::Start(Tag::List(start)) => primitives::ListGenerator::new(self).gen_list(start, state),
            Event::Start(Tag::Item) => unreachable!("list should be handled by ListGenerator"),
            Event::Start(Tag::FootnoteDefinition(fnote)) => primitives::gen_footnote_definition(self, &fnote, state),
            Event::Start(Tag::Table(align)) => primitives::gen_table(self, align, state),
            Event::Start(Tag::TableHead) => primitives::gen_table_head(self, state),
            Event::Start(Tag::TableRow) => primitives::gen_table_row(self, state),
            Event::Start(Tag::TableCell) => primitives::gen_table_cell(self, state),
            Event::Start(Tag::Emphasis) => primitives::gen_emphasized(self, state),
            Event::Start(Tag::Strong) => primitives::gen_strong(self, state),
            Event::Start(Tag::Code) => primitives::gen_code(self, state),
            Event::Start(Tag::Link(dst, title)) => primitives::gen_link(self, &dst, &title, state),
            Event::Start(Tag::Image(dst, title)) => primitives::gen_image(self, &dst, &title, state),
            Event::End(_) => unreachable!("end should be handled by gen_* functions"),
        }
    }

}

impl Article {
    pub fn new() -> Self {
        Article
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

