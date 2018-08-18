use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::Document;

mod preamble;

#[derive(Debug)]
pub struct Article;

impl<'a> Document<'a> for Article {
    type Simple = super::SimpleGen;
    type Paragraph = super::Paragraph;
    type Rule = super::Rule;
    type Header = super::Header<'a>;
    type BlockQuote = super::BlockQuote<'a>;
    type CodeBlock = super::CodeBlock;
    type List = super::List;
    type Item = super::Item;
    type FootnoteDefinition = super::FootnoteDefinition;
    type Table = super::Table;
    type TableHead = super::TableHead;
    type TableRow = super::TableRow;
    type TableCell = super::TableCell;
    type InlineEmphasis = super::InlineEmphasis;
    type InlineStrong = super::InlineStrong;
    type InlineCode = super::InlineCode;
    type Link = super::Link<'a>;
    type Image = super::Image<'a>;

    fn new() -> Self {
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

