use std::io::Write;
use diagnostic::{FileId, Span};

use crate::backend::Backend;
use crate::backend::latex::{self, preamble};
use crate::config::Config;
use crate::error::{FatalResult, Result, Error, DiagnosticCode};
use crate::backend::latex::preamble::ShortAuthor;
use crate::Diagnostics;

#[derive(Debug)]
pub struct Beamer {
    /// Stack of used headings, used to close frames.
    ///
    /// 1: section
    /// 2: subsection / frame
    /// 3: beamerboxesrounded
    headings: Vec<i32>,
}

#[derive(Clone, Copy, Debug)]
pub enum FrameEvent {
    BeginFrame,
    BeginBox,
    EndFrame,
    EndBox,
}

impl Beamer {
    /// Closes the beamerboxesrounded / slides until including the given level.
    /// Also performs the according checks and updates the heading stack.
    pub fn close_until(
        &mut self, level: i32, out: &mut impl Write, span: Span,
        diagnostics: &Diagnostics,
    ) -> Result<Vec<FrameEvent>> {
        check_level(level, span, diagnostics)?;

        let mut levels = Vec::new();
        while let Some(&stack_level) = self.headings.last() {
            if stack_level < level {
                break;
            }
            self.headings.pop().unwrap();
            // TODO: make heading-level configurable
            match stack_level {
                1 => {},
                2 => {
                    writeln!(out, "\\end{{frame}}\n")?;
                    levels.push(FrameEvent::EndFrame);
                },
                3 => {
                    writeln!(out, "\\end{{beamerboxesrounded}}")?;
                    levels.push(FrameEvent::EndBox);
                    if level == 3 {
                        // space between two beamerboxesrounded
                        writeln!(out, "\\vspace{{1em}}")?;
                    }
                },
                _ => unreachable!(),
            }
        }
        Ok(levels)
    }

    /// Opens the beamerboxesrounded / slides until including the given level, updating the heading
    /// stack.
    pub fn open_until(
        &mut self, level: i32, cfg: &Config, out: &mut impl Write, span: Span,
        diagnostics: &Diagnostics,
    ) -> Result<Vec<FrameEvent>> {
        check_level(level, span, diagnostics)?;
        let last = self.headings.last().cloned().unwrap_or(0);
        let mut levels = Vec::new();
        for level in (last+1)..=level {
            self.headings.push(level);
            match level {
                1 => if cfg.sectionframes {
                    writeln!(out, "\\begin{{frame}}")?;
                    levels.push(FrameEvent::BeginFrame);
                    writeln!(out, "\\Huge\\centering \\insertsection")?;
                    writeln!(out, "\\end{{frame}}\n")?;
                    levels.push(FrameEvent::EndFrame);
                },
                2 => {
                    // Mark all slides as fragile, this is slower but we can use verbatim etc.
                    writeln!(out, "\\begin{{frame}}[fragile]")?;
                    if cfg.frameheadings {
                        writeln!(out, "\\frametitle{{\\insertsection}}")?;
                        writeln!(out, "\\framesubtitle{{\\insertsubsection}}")?;
                    }
                    levels.push(FrameEvent::BeginFrame);
                },
                3 => {
                    writeln!(out, "\\begin{{beamerboxesrounded}}{{\\insertsubsubsection}}")?;
                    levels.push(FrameEvent::BeginBox);
                },
                _ => unreachable!(),
            }
        }
        Ok(levels)
    }
}

fn check_level(level: i32, span: Span, diagnostics: &Diagnostics) -> Result<()> {
    assert!(level > 0, "Header level should be positive, but is {}", level);
    if level > 3 {
        diagnostics
            .error(DiagnosticCode::InvalidHeaderLevel)
            .with_error_label(span, "heading level in beamer greater than 3")
            .with_note("beamer only supports levels <= 3")
            .with_note("skipping over it")
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
    type Rule = latex::RuleGen;
    type PageBreak = latex::BeamerPageBreakGen<'a>;
    type TaskListMarker = latex::TaskListMarkerGen;
    type TableOfContents = latex::TableOfContentsGen;
    type Bibliography = latex::BibliographyGen;
    type ListOfTables = latex::ListOfTablesGen;
    type ListOfFigures = latex::ListOfFiguresGen;
    type ListOfListings = latex::ListOfListingsGen;
    type Appendix = latex::AppendixGen;

    type Paragraph = latex::ParagraphGen;
    type Header = latex::BeamerHeaderGen<'a>;
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
        Beamer {
            headings: Vec::new(),
        }
    }

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &'a Diagnostics) -> FatalResult<()> {
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
            preamble::write_maketitle_info(cfg, ShortAuthor::Yes, out, diagnostics)?;
            preamble::write_manual_titlepage_commands(cfg, out)?;
            writeln!(out, "\\frame{{\\titlepage}}")?;
        }


        Ok(())
    }

    fn gen_epilogue(&mut self, _cfg: &Config, out: &mut impl Write, diagnostics: &'a Diagnostics) -> FatalResult<()> {
        match self.close_until(1, out, Span { file: FileId::synthetic("test"), start: 0, end: 0 }, diagnostics) {
            Ok(_events) => (),
            Err(Error::Diagnostic) => unreachable!(),
            Err(Error::Fatal(fatal)) => return Err(fatal),
        }
        writeln!(out, "\\end{{document}}")?;
        Ok(())
    }
}
