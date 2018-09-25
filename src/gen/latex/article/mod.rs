use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::Backend;
use crate::config::Config;

mod preamble;

#[derive(Debug)]
pub struct Article;

impl<'a> Backend<'a> for Article {
    type Text = super::TextGen;
    type FootnoteReference = super::FootnoteReferenceGen;
    type SoftBreak = super::SoftBreakGen;
    type HardBreak = super::HardBreakGen;

    type Paragraph = super::ParagraphGen;
    type Rule = super::RuleGen;
    type Header = super::HeaderGen;
    type BlockQuote = super::BlockQuoteGen;
    type CodeBlock = super::CodeBlockGen;
    type List = super::ListGen;
    type Enumerate = super::EnumerateGen;
    type Item = super::ItemGen;
    type FootnoteDefinition = super::FootnoteDefinitionGen;
    type Table = super::TableGen;
    type TableHead = super::TableHeadGen;
    type TableRow = super::TableRowGen;
    type TableCell = super::TableCellGen;
    type InlineEmphasis = super::InlineEmphasisGen;
    type InlineStrong = super::InlineStrongGen;
    type InlineCode = super::InlineCodeGen;
    type Link = super::LinkGen<'a>;
    type Image = super::ImageGen<'a>;

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

