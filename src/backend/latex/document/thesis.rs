use std::fs;
use std::io::Write;
use std::path::PathBuf;
use diagnostic::{Span, Spanned};

use crate::backend::latex::{self, preamble};
use crate::backend::Backend;
use crate::config::Config;
use crate::{CONFIG_SPAN, Diagnostics};
use crate::error::{DiagnosticCode, FatalResult};
use crate::generator::Generator;
use crate::resolve::Context;

#[derive(Debug)]
pub struct Thesis;

#[rustfmt::skip]
impl<'a> Backend<'a> for Thesis {
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
    type Rule = latex::RuleGen;
    type PageBreak = latex::PageBreakGen;
    type TaskListMarker = latex::TaskListMarkerGen;
    type TableOfContents = latex::TableOfContentsGen;
    type Bibliography = latex::BibliographyGen;
    type ListOfTables = latex::ListOfTablesGen;
    type ListOfFigures = latex::ListOfFiguresGen;
    type ListOfListings = latex::ListOfListingsGen;
    type Appendix = latex::AppendixGen;

    type Paragraph = latex::ParagraphGen;
    type Header = latex::BookHeaderGen<'a>;
    type BlockQuote = latex::BlockQuoteGen;
    type CodeBlock = latex::CodeBlockGen;
    type List = latex::ListGen;
    type Enumerate = latex::EnumerateGen;
    type Item = latex::ItemGen;
    type FootnoteDefinition = latex::FootnoteDefinitionGen;
    type UrlWithContent = latex::UrlWithContentGen<'a>;
    type InterLinkWithContent = latex::InterLinkWithContentGen;
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
        Thesis
    }

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &'a Diagnostics) -> FatalResult<()> {
        // TODO: itemizespacing
        preamble::write_documentclass(cfg, out, "scrbook", "headsepline,footsepline,BCOR=12mm,DIV=12,")?;
        preamble::write_packages(cfg, out)?;
        preamble::write_fixes(cfg, out)?;

        writeln!(out)?;
        writeln!(out, "\\begin{{document}}")?;
        writeln!(out)?;

        preamble::write_manual_titlepage_commands(cfg, out)?;

        writeln!(out, "\\pagenumbering{{alph}}")?;
        writeln!(out, "{}", preamble::THESIS_COVER)?;

        writeln!(out, "\\frontmatter{{}}")?;

        writeln!(out, "{}", preamble::THESIS_TITLE)?;

        if let Some(disclaimer) = &cfg.disclaimer {
            writeln!(out, "\\newcommand*{{\\getDisclaimer}}{{{}}}", disclaimer)?;
            writeln!(out, "{}", preamble::THESIS_DISCLAIMER)?;
        }

        writeln!(out, "\\cleardoublepage{{}}")?;

        if let Some(abstract1) = &cfg.abstract1 {
            gen_abstract(abstract1.clone(), "abstract", cfg, out, diagnostics)?;
        }
        if let Some(abstract2) = &cfg.abstract2 {
            gen_abstract(abstract2.clone(), "abstract2", cfg, out, diagnostics)?;
        }

        writeln!(out)?;
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\tableofcontents{{}}")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        writeln!(out)?;
        writeln!(out, "\\mainmatter{{}}")?;

        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write, _diagnostics: &'a Diagnostics) -> FatalResult<()> {
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}

fn gen_abstract(path: PathBuf, abstract_name: &str, cfg: &Config, out: &mut impl Write, diagnostics: &Diagnostics) -> FatalResult<()> {
    let mut gen = Generator::new(cfg, Thesis, out, diagnostics);
    let markdown = fs::read_to_string(&path)?;
    let context = match Context::from_path(path.clone()) {
        Ok(context) => context,
        Err(e) => {
            diagnostics
                .error(DiagnosticCode::InvalidConfigPath(abstract_name.to_string()))
                .with_error_label(CONFIG_SPAN, format!("cause: {:?}", e))
                .with_note("can't create a URL from the path")
                .with_note("skipping over it")
                .emit();
            return Ok(());
        }
    };
    let (fileid, markdown) = diagnostics.add_file(path.display().to_string(), markdown);
    let markdown = Spanned::new(markdown, Span::new(fileid, 0, markdown.len()));
    let events = gen.get_events(markdown, context);
    gen.generate_body(events)?;
    Ok(())
}
