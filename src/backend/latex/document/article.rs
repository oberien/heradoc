use std::io::Write;

use crate::backend::latex::{self, preamble};
use crate::backend::Backend;
use crate::config::Config;
use crate::error::FatalResult;
use crate::diagnostics::Diagnostics;

#[derive(Debug)]
pub struct Article;

#[rustfmt::skip]
impl<'a> Backend<'a> for Article {
    type Text = latex::TextGen;
    type Latex = latex::LatexGen;
    type FootnoteReference = latex::FootnoteReferenceGen;
    type BiberReferences = latex::BiberReferencesGen;
    type Url = latex::UrlGen;
    type InterLink = latex::InterLinkGen;
    type Image = latex::ImageGen;
    type Label = latex::LabelGen;
    type Pdf = latex::PdfGen;
    type SoftBreak = latex::SoftBreakGen;
    type HardBreak = latex::HardBreakGen;
    type TaskListMarker = latex::TaskListMarkerGen;
    type TableOfContents = latex::TableOfContentsGen;
    type Bibliography = latex::BibliographyGen;
    type ListOfTables = latex::ListOfTablesGen;
    type ListOfFigures = latex::ListOfFiguresGen;
    type ListOfListings = latex::ListOfListingsGen;
    type Appendix = latex::AppendixGen;

    type Paragraph = latex::ParagraphGen;
    type Rule = latex::RuleGen;
    type Header = latex::HeaderGen<'a>;
    type BlockQuote = latex::BlockQuoteGen;
    type CodeBlock = latex::CodeBlockGen;
    type List = latex::ListGen;
    type Enumerate = latex::EnumerateGen;
    type Item = latex::ItemGen;
    type FootnoteDefinition = latex::FootnoteDefinitionGen;
    type UrlWithContent = latex::UrlWithContentGen<'a>;
    type InterLinkWithContent = latex::InterLinkWithContentGen;
    type HtmlBlock = latex::HtmlBlockGen;
    type Figure = latex::FigureGen<'a>;

    type TableFigure = latex::TableFigureGen<'a>;
    type Table = latex::TableGen<'a>;
    type TableHead = latex::TableHeadGen;
    type TableRow = latex::TableRowGen;
    type TableCell = latex::TableCellGen;

    type InlineEmphasis = latex::InlineEmphasisGen;
    type InlineStrong = latex::InlineStrongGen;
    type InlineStrikethrough = latex::InlineStrikethroughGen;
    type InlineCode = latex::InlineCodeGen;
    type InlineMath = latex::InlineMathGen;

    type Equation = latex::EquationGen<'a>;
    type NumberedEquation = latex::NumberedEquationGen<'a>;
    type Graphviz = latex::GraphvizGen<'a>;

    fn new() -> Self {
        Article
    }

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, _diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        // TODO: itemizespacing
        preamble::write_documentclass(cfg, out, "scrartcl", "")?;

        preamble::write_packages(cfg, out)?;
        preamble::write_fixes(cfg, out)?;

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

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write, _diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}
