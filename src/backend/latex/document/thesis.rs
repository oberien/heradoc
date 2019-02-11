use std::io::{Write, Result};
use std::fs;
use std::path::Path;

use typed_arena::Arena;

use crate::backend::Backend;
use crate::backend::latex::{self, preamble};
use crate::config::Config;
use crate::generator::Generator;

#[derive(Debug)]
pub struct Thesis;

impl<'a> Backend<'a> for Thesis {
    type Text = latex::TextGen;
    type FootnoteReference = latex::FootnoteReferenceGen;
    type Link = latex::LinkGen;
    type Image = latex::ImageGen;
    type Label = latex::LabelGen;
    type Pdf = latex::PdfGen;
    type SoftBreak = latex::SoftBreakGen;
    type HardBreak = latex::HardBreakGen;
    type TableOfContents = latex::TableOfContentsGen;
    type Bibliography = latex::BibliographyGen;
    type ListOfTables = latex::ListOfTablesGen;
    type ListOfFigures = latex::ListOfFiguresGen;
    type ListOfListings = latex::ListOfListingsGen;
    type Appendix = latex::AppendixGen;

    type Paragraph = latex::ParagraphGen;
    type Rule = latex::RuleGen;
    type Header = latex::BookHeaderGen<'a>;
    type BlockQuote = latex::BlockQuoteGen;
    type CodeBlock = latex::CodeBlockGen;
    type List = latex::ListGen;
    type Enumerate = latex::EnumerateGen;
    type Item = latex::ItemGen;
    type FootnoteDefinition = latex::FootnoteDefinitionGen;
    type Figure = latex::FigureGen<'a>;
    type Table = latex::TableGen;
    type TableHead = latex::TableHeadGen;
    type TableRow = latex::TableRowGen;
    type TableCell = latex::TableCellGen;
    type InlineEmphasis = latex::InlineEmphasisGen;
    type InlineStrong = latex::InlineStrongGen;
    type InlineCode = latex::InlineCodeGen;
    type InlineMath = latex::InlineMathGen;
    type Equation = latex::EquationGen;
    type NumberedEquation = latex::NumberedEquationGen;
    type Graphviz = latex::GraphvizGen<'a>;

    fn new() -> Self {
        Thesis
    }

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write) -> Result<()> {
        // TODO: itemizespacing
        // documentclass
        write!(out, "\\documentclass[")?;
        write!(out, "{},", cfg.fontsize)?;
        write!(out, "headsepline,footsepline,BCOR=12mm,DIV=12,")?;
        for other in &cfg.classoptions {
            write!(out, "{},", other)?;
        }
        writeln!(out, "]{{scrbook}}")?;
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
        writeln!(out, "\\newcommand*{{\\getThesisType}}{{{}}}", get(&cfg.thesis_type))?;
        writeln!(out, "\\newcommand*{{\\getLocation}}{{{}}}", get(&cfg.location))?;

        writeln!(out, "\\pagenumbering{{alph}}")?;
        writeln!(out, "{}", preamble::THESIS_COVER)?;

        writeln!(out, "\\frontmatter{{}}")?;

        writeln!(out, "{}", preamble::THESIS_TITLE)?;

        if let Some(disclaimer) = &cfg.disclaimer {
            writeln!(out, "\\newcommand*{{\\getDisclaimer}}{{{}}}", disclaimer)?;
            writeln!(out, "{}", preamble::THESIS_DISCLAIMER)?;
        }

        writeln!(out, "\\cleardoublepage{{}}")?;

        if let Some(_abstract) = &cfg._abstract {
            gen(_abstract, cfg, out)?;
        }
        if let Some(abstract2) = &cfg.abstract2 {
            gen(abstract2, cfg, out)?;
        }

        writeln!(out)?;
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\tableofcontents{{}}")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        writeln!(out)?;
        writeln!(out, "\\mainmatter{{}}")?;

        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}

fn gen<P: AsRef<Path>>(path: P, cfg: &Config, out: &mut impl Write) -> Result<()> {
    let arena = Arena::new();
    let mut gen = Generator::new(cfg, Thesis, out, &arena);
    let markdown = fs::read_to_string(path)?;
    let events = gen.get_events(markdown);
    gen.generate_body(events)?;
    Ok(())

}
