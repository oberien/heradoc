use std::io::{Write, Result};

use crate::backend::Backend;
use crate::config::Config;

mod preamble;

#[derive(Debug)]
pub struct Article;

impl<'a> Backend<'a> for Article {
    type Text = super::TextGen;
    type FootnoteReference = super::FootnoteReferenceGen;
    type Link = super::LinkGen;
    type Image = super::ImageGen;
    type Pdf = super::PdfGen;
    type SoftBreak = super::SoftBreakGen;
    type HardBreak = super::HardBreakGen;
    type TableOfContents = super::TableOfContentsGen;
    type Bibliography = super::BibliographyGen;
    type ListOfTables = super::ListOfTablesGen;
    type ListOfFigures = super::ListOfFiguresGen;
    type ListOfListings = super::ListOfListingsGen;
    type Appendix = super::AppendixGen;

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
    type InlineMath = super::InlineMathGen;
    type Equation = super::EquationGen;
    type NumberedEquation = super::NumberedEquationGen;
    type Graphviz = super::GraphvizGen<'a>;

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

        writeln!(out, "\\usepackage[{}]{{babel}}", cfg.lang.to_name().to_ascii_lowercase())?;
        writeln!(out, "\\usepackage{{csquotes}}")?;

        // geometry
        write!(out, "\\usepackage[")?;
        cfg.geometry.write_latex_options(&mut *out)?;
        writeln!(out, "]{{geometry}}")?;

        writeln!(out, "\\usepackage[utf8]{{inputenc}}")?;
        writeln!(out)?;

        // TODO: biblatex options (natbib?)
        if let Some(bibliography) = &cfg.bibliography {
            writeln!(out, "\\usepackage[backend=biber,citestyle={},bibstyle={}]{{biblatex}}", cfg.citestyle, cfg.bibstyle)?;
            writeln!(out, "\\addbibresource{{{}}}", bibliography.display())?;
        }

        // TODO: use minted instead of lstlistings?
        writeln!(out, "\\usepackage{{listings}}")?;
        writeln!(out, "\\usepackage[usenames, dvipsnames]{{color}}")?;
        writeln!(out, "\\usepackage{{xcolor}}")?;
        writeln!(out, "\\usepackage{{pdfpages}}")?;
        writeln!(out, "\\usepackage{{amssymb}}")?;
        writeln!(out, "\\usepackage{{amsmath}}")?;
        writeln!(out, "{}", preamble::lstset)?;
        writeln!(out, "{}", preamble::lstdefineasm)?;
        writeln!(out, "{}", preamble::lstdefinerust)?;
        // TODO: graphicspath
        writeln!(out, "\\usepackage{{graphicx}}")?;
        writeln!(out, "\\usepackage[pdfusetitle]{{hyperref}}")?;
        writeln!(out, "\\usepackage{{caption}}")?;
        // TODO: cleveref options
        writeln!(out, "\\usepackage{{cleveref}}")?;
        writeln!(out, "\\usepackage{{refcount}}")?;
        writeln!(out, "\\usepackage[titletoc,toc,title]{{appendix}}")?;
        writeln!(out, "\\usepackage{{array}}")?;
        writeln!(out, "{}", preamble::thickhline)?;
        writeln!(out)?;
        writeln!(out, "{}", preamble::aquote)?;
        writeln!(out)?;

        for include in &cfg.header_includes {
            writeln!(out, "{}", include)?;
        }

        writeln!(out)?;
        writeln!(out, "\\begin{{document}}")?;
        writeln!(out)?;

        if let Some(title) = &cfg.title {
            writeln!(out, "\\title{{{}}}", title)?;
        }
        if let Some(subtitle) = &cfg.subtitle {
            writeln!(out, "\\subtitle{{{}}}", subtitle)?;
        }
        if let Some(author) = &cfg.author {
            writeln!(out, "\\author{{{}}}", author)?;
        }
        if let Some(date) = &cfg.date {
            writeln!(out, "\\date{{{}}}", date)?;
        }
        let publisher = match (&cfg.publisher, &cfg.supervisor, &cfg.advisor) {
            (None, None, None) => None,
            (a, b, c) => {
                let mut buffer = String::new();
                a.as_ref().map(|s| { buffer.push_str(s); buffer.push_str("\\\\"); });
                // TODO: i18n
                // TODO: better use table here
                b.as_ref().map(|s| { buffer.push_str("Supervisor: "); buffer.push_str(s); buffer.push_str("\\\\"); });
                c.as_ref().map(|s| { buffer.push_str("Advisor: "); buffer.push_str(s); buffer.push_str("\\\\"); });
                // strip possibly leading linebreak
                buffer.pop(); buffer.pop();
                Some(buffer)
            }
        };
        if let Some(publisher) = publisher {
            writeln!(out, "\\publishers{{{}}}", publisher)?;
        }

        if cfg.title.is_some() {
            // TODO: Warn if title isn't set but something else is
            writeln!(out, "\\maketitle")?;
        }
        writeln!(out)?;

        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}

