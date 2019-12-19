use std::io::Write;

use crate::backend::Backend;
use crate::backend::latex::{self, preamble};
use crate::config::Config;
use crate::error::{FatalResult, Result, Error};
use crate::diagnostics::Diagnostics;
use crate::frontend::range::SourceRange;

#[derive(Debug)]
pub struct Beamer {
    /// Stack of used headings, used to close frames.
    ///
    /// 1: section
    /// 2: subsection / frame
    /// 3: beamerboxesrounded
    headings: Vec<i32>,
}

impl Beamer {
    /// Closes the beamerboxesrounded / slides until including the given level.
    /// Also performs the according checks and updates the heading stack.
    pub fn close_until(
        &mut self, level: i32, out: &mut impl Write, range: SourceRange,
        diagnostics: &Diagnostics<'_>,
    ) -> Result<()> {
        check_level(level, range, diagnostics)?;

        while let Some(&stack_level) = self.headings.last() {
            if stack_level < level {
                break;
            }
            self.headings.pop().unwrap();
            // TODO: make heading-level configurable
            match stack_level {
                1 => (),
                2 => writeln!(out, "\\end{{frame}}\n")?,
                3 => writeln!(out, "\\end{{beamerboxesrounded}}")?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    /// Opens the beamerboxesrounded / slides until including the given level, updating the heading
    /// stack.
    pub fn open_until(
        &mut self, level: i32, cfg: &Config, out: &mut impl Write, range: SourceRange,
        diagnostics: &Diagnostics<'_>,
    ) -> Result<()> {
        check_level(level, range, diagnostics)?;
        let last = self.headings.last().cloned().unwrap_or(0);
        for level in (last+1)..=level {
            self.headings.push(level);
            match level {
                1 => if cfg.sectionframes {
                    writeln!(out, "\\begin{{frame}}")?;
                    writeln!(out, "\\Huge\\centering \\insertsection")?;
                    writeln!(out, "\\end{{frame}}\n")?;
                },
                2 => {
                    // Mark all slides as fragile, this is slower but we can use verbatim etc.
                    writeln!(out, "\\begin{{frame}}[fragile]")?;
                    writeln!(out, "\\frametitle{{\\insertsection}}")?;
                    writeln!(out, "\\framesubtitle{{\\insertsubsection}}")?;
                },
                3 => writeln!(out, "\\begin{{beamerboxesrounded}}{{\\insertsubsubsection}}")?,
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

fn check_level(level: i32, range: SourceRange, diagnostics: &Diagnostics<'_>) -> Result<()> {
    assert!(level > 0, "Header level should be positive, but is {}", level);
    if level > 3 {
        diagnostics
            .error("heading level in beamer greater than 3")
            .with_error_section(range, "for this heading")
            .note("beamer only supports levels >= 3")
            .note("skipping over it")
            .emit();
        return Err(Error::Diagnostic);
    }
    Ok(())
}

#[rustfmt::skip]
impl<'a> Backend<'a> for Beamer {
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
    type TaskListMarker = latex::TaskListMarkerGen;
    type TableOfContents = latex::TableOfContentsGen;
    type Bibliography = latex::BibliographyGen;
    type ListOfTables = latex::ListOfTablesGen;
    type ListOfFigures = latex::ListOfFiguresGen;
    type ListOfListings = latex::ListOfListingsGen;
    type Appendix = latex::AppendixGen;

    type Paragraph = latex::ParagraphGen;
    type Rule = latex::BeamerRuleGen<'a>;
    type Header = latex::BeamerHeaderGen<'a>;
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

    type MathBlock = latex::MathBlockGen<'a>;
    type Graphviz = latex::GraphvizGen<'a>;

    fn new() -> Self {
        Beamer {
            headings: Vec::new(),
        }
    }

    fn gen_preamble(&mut self, cfg: &Config, mut out: &mut impl Write, _diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        // Beamer already loads internally color, hyperref, xcolor. Correct their options.
        preamble::write_documentclass(cfg, out, "beamer", "color={usenames,dvipsnames},xcolor={usenames,dvipsnames},hyperref={pdfusetitle},")?;
        writeln!(out, "\\usetheme{{{}}}", cfg.beamertheme)?;
        writeln!(out)?;

        preamble::write_packages(cfg, out)?;
        preamble::write_fixes(cfg, out)?;

        writeln!(out)?;
        writeln!(out, "\\begin{{document}}")?;
        writeln!(out)?;
        writeln!(out, "\\pagenumbering{{alph}}")?;
        writeln!(out)?;

        if cfg.titlepage {
            // TODO: warn if any info is set but titlepage false
            preamble::write_maketitle_info(cfg, out)?;
            writeln!(out, "\\frame{{\\titlepage}}")?;
        }


        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write, diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        match self.close_until(1, out, SourceRange { start: 0, end: 0 }, diagnostics) {
            Ok(()) => (),
            Err(Error::Diagnostic) => unreachable!(),
            Err(Error::Fatal(fatal)) => return Err(fatal),
        }
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}
