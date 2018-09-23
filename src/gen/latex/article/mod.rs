use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::Document;
use crate::config::Config;

mod preamble;

#[derive(Debug)]
pub struct Article;

impl<'a> Document<'a> for Article {
    type Simple = super::SimpleGen;
    type Paragraph = super::Paragraph;
    type Rule = super::Rule;
    type Header = super::Header;
    type BlockQuote = super::BlockQuote;
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

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()> {
        // TODO: language / locale
        // TODO: itemizespacing
        // documentclass
        write!(out, "\\documentclass[")?;
        write!(out, "{},", cfg.fontsize)?;
        match cfg.titlepage {
            true => write!(out, "titlepage,")?,
            false => write!(out, "notitlepage,")?,
        }
        for other in &cfg.classoptions {
            write!(out, "{},", other)?;
        }
        writeln!(out, "]{{{}}}", cfg.documentclass)?;
        // geometry
        write!(out, "\\usepackage[")?;
        cfg.geometry.write_latex_options(&mut *out)?;
        writeln!(out, "]{{geometry}}")?;

        writeln!(out, "\\usepackage[utf8]{{inputenc}}")?;
        writeln!(out)?;

        // TODO: biblatex options (natbib?)
        if let Some(bibliography) = &cfg.bibliography {
            write!(out, "\\usepackage[backend=biber,citestyle={},bibstyle={}]{{biblatex}}", cfg.citestyle, cfg.bibstyle)?;
            writeln!(out, "\\addbibresource{{{}}}", bibliography.display())?;
        }

        // TODO: use minted instead of lstlistings?
        writeln!(out, "\\usepackage{{listings}}")?;
        writeln!(out, "\\usepackage[usenames, dvipsnames]{{color}}")?;
        writeln!(out, "\\usepackage{{xcolor}}")?;
        writeln!(out, "{}", preamble::lstset)?;
        writeln!(out, "{}", preamble::lstdefineasm)?;
        writeln!(out, "{}", preamble::lstdefinerust)?;
        // TODO: graphicspath
        writeln!(out, "\\usepackage{{graphicx}}")?;
        writeln!(out, "\\usepackage{{hyperref}}")?;
        // TODO: fix this?!
        writeln!(out, "\\usepackage[all]{{hypcap}}")?;
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

    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()> {
        // TODO: [bibliography]
        if cfg.bibliography.is_some() {
            writeln!(out, "\\printbibliography")?;
        }
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}

