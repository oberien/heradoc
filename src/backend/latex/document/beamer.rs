use std::io::Write;

use crate::backend::Backend;
use crate::backend::latex::{self, preamble};
use crate::config::Config;
use crate::error::FatalResult;
use crate::diagnostics::Diagnostics;

#[derive(Debug)]
pub struct Beamer;

#[rustfmt::skip]
impl<'a> Backend<'a> for Beamer {
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
        Beamer
    }

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, _diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        write!(out, "\\documentclass[")?;
        write!(out, "{},", cfg.fontsize)?;
        for other in &cfg.classoptions {
            write!(out, "{},", other)?;
        }

        // Beamer already loads internally color, hyperref, xcolor. Correct their options.
        writeln!(out, "color={{usenames,dvipsnames}},")?;
        writeln!(out, "xcolor={{usenames,dvipsnames}},")?;
        writeln!(out, "hyperref={{pdfusetitle}},")?;

        writeln!(out, "]{{beamer}}")?;
        writeln!(out)?;

        preamble::write_packages(cfg, out)?;
        preamble::write_fixes(cfg, out)?;

        writeln!(out)?;
        writeln!(out, "\\def \\ifempty#1{{\\def\\temp{{#1}} \\ifx\\temp\\empty}}")?;

        writeln!(out)?;
        writeln!(out, "\\begin{{document}}")?;
        writeln!(out)?;

        fn get(o: &Option<String>) -> &str { o.as_ref().map(|s| s.as_str()).unwrap_or("") }
        writeln!(out, "\\newcommand*{{\\getTitle}}{{{}}}", get(&cfg.title))?;
        writeln!(out, "\\newcommand*{{\\getSubtitle}}{{{}}}", get(&cfg.subtitle))?;
        writeln!(out, "\\newcommand*{{\\getAuthor}}{{{}}}", get(&cfg.author))?;
        writeln!(out, "\\newcommand*{{\\getDate}}{{{}}}", get(&cfg.date))?;
        writeln!(out, "\\newcommand*{{\\getSupervisor}}{{{}}}", get(&cfg.supervisor))?;
        writeln!(out, "\\newcommand*{{\\getAdvisor}}{{{}}}", get(&cfg.advisor))?;
        if let Some(logo_university) = cfg.logo_university.as_ref() {
            writeln!(out, "\\newcommand*{{\\getLogoUniversity}}{{{}}}", logo_university.display())?;
        } else {
            writeln!(out, "\\newcommand*{{\\getLogoUniversity}}{{}}")?;
        }
        if let Some(logo_faculty) = cfg.logo_faculty.as_ref() {
            writeln!(out, "\\newcommand*{{\\getLogoFaculty}}{{{}}}", logo_faculty.display())?;
        } else {
            writeln!(out, "\\newcommand*{{\\getLogoFaculty}}{{}}")?;
        }
        writeln!(out, "\\newcommand*{{\\getUniversity}}{{{}}}", get(&cfg.university))?;
        writeln!(out, "\\newcommand*{{\\getFaculty}}{{{}}}", get(&cfg.faculty))?;
        writeln!(out, "\\newcommand*{{\\getLocation}}{{{}}}", get(&cfg.location))?;

        writeln!(out, "\\pagenumbering{{alph}}")?;

        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write, _diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}
