use std::io::Write;

use crate::backend::latex::{self, preamble};
use crate::backend::Backend;
use crate::config::Config;
use crate::error::FatalResult;
use crate::diagnostics::Diagnostics;
use crate::backend::latex::preamble::ShortAuthor;

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
    type Svg = latex::SvgGen;
    type Label = latex::LabelGen;
    type Pdf = latex::PdfGen;
    type SoftBreak = latex::SoftBreakGen;
    type HardBreak = latex::HardBreakGen;
    type PageBreak = latex::PageBreakGen;
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

        if cfg.title.is_some() {
            // TODO: Warn if title isn't set but something else is
            preamble::write_maketitle_info(cfg, ShortAuthor::No, out)?;
            preamble::write_manual_titlepage_commands(cfg, out)?;
            writeln!(out, "\\maketitle")?;
        }
        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write, _diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}
