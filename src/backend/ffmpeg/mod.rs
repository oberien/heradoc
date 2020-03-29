use std::borrow::Cow;
use std::io::{self, Write};

use crate::backend::latex::{self, Beamer};
use crate::backend::{Backend, CodeGenUnit, StatefulCodeGenUnit};
use crate::config::Config;
use crate::diagnostics::Diagnostics;
use crate::error::{FatalResult, Result};
use crate::frontend::range::{WithRange, SourceRange};
use crate::generator::event::{Event, Header};
use crate::generator::Generator;

#[derive(Debug)]
pub struct SlidesFfmpegEspeak {
    slides: Beamer,
}

#[rustfmt::skip]
impl<'a> Backend<'a> for SlidesFfmpegEspeak {
    type Text = <Beamer as Backend<'a>>::Text;
    type Latex = <Beamer as Backend<'a>>::Latex;
    type FootnoteReference = <Beamer as Backend<'a>>::FootnoteReference;
    type BiberReferences = <Beamer as Backend<'a>>::BiberReferences;
    type Url = <Beamer as Backend<'a>>::Url;
    type InterLink = <Beamer as Backend<'a>>::InterLink;
    type Image = <Beamer as Backend<'a>>::Image;
    type Svg = <Beamer as Backend<'a>>::Svg;
    type Label = <Beamer as Backend<'a>>::Label;
    type Pdf = <Beamer as Backend<'a>>::Pdf;
    type SoftBreak = <Beamer as Backend<'a>>::SoftBreak;
    type HardBreak = <Beamer as Backend<'a>>::HardBreak;
    type TaskListMarker = <Beamer as Backend<'a>>::TaskListMarker;
    type TableOfContents = <Beamer as Backend<'a>>::TableOfContents;
    type Bibliography = <Beamer as Backend<'a>>::Bibliography;
    type ListOfTables = <Beamer as Backend<'a>>::ListOfTables;
    type ListOfFigures = <Beamer as Backend<'a>>::ListOfFigures;
    type ListOfListings = <Beamer as Backend<'a>>::ListOfListings;
    type Appendix = <Beamer as Backend<'a>>::Appendix;

    type Paragraph = <Beamer as Backend<'a>>::Paragraph;
    type Rule = PseudoBeamerRuleGen<'a>;
    type Header = PseudoBeamerHeaderGen<'a>;
    type BlockQuote = <Beamer as Backend<'a>>::BlockQuote;
    type CodeBlock = <Beamer as Backend<'a>>::CodeBlock;
    type List = <Beamer as Backend<'a>>::List;
    type Enumerate = <Beamer as Backend<'a>>::Enumerate;
    type Item = <Beamer as Backend<'a>>::Item;
    type FootnoteDefinition = <Beamer as Backend<'a>>::FootnoteDefinition;
    type UrlWithContent = <Beamer as Backend<'a>>::UrlWithContent;
    type InterLinkWithContent = <Beamer as Backend<'a>>::InterLinkWithContent;
    type HtmlBlock = <Beamer as Backend<'a>>::HtmlBlock;
    type Figure = <Beamer as Backend<'a>>::Figure;

    type TableFigure = <Beamer as Backend<'a>>::TableFigure;
    type Table = <Beamer as Backend<'a>>::Table;
    type TableHead = <Beamer as Backend<'a>>::TableHead;
    type TableRow = <Beamer as Backend<'a>>::TableRow;
    type TableCell = <Beamer as Backend<'a>>::TableCell;

    type InlineEmphasis = <Beamer as Backend<'a>>::InlineEmphasis;
    type InlineStrong = <Beamer as Backend<'a>>::InlineStrong;
    type InlineStrikethrough = <Beamer as Backend<'a>>::InlineStrikethrough;
    type InlineCode = <Beamer as Backend<'a>>::InlineCode;
    type InlineMath = <Beamer as Backend<'a>>::InlineMath;

    type Equation = <Beamer as Backend<'a>>::Equation;
    type NumberedEquation = <Beamer as Backend<'a>>::NumberedEquation;
    type Graphviz = <Beamer as Backend<'a>>::Graphviz;

    fn new() -> Self {
        SlidesFfmpegEspeak {
            slides: Beamer::new(),
        }
    }

    fn gen_preamble(&mut self, cfg: &Config, mut out: &mut impl Write, diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        self.slides.gen_preamble(cfg, out, diagnostics)
    }

    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &Diagnostics<'a>) -> FatalResult<()> {
        self.slides.gen_epilogue(cfg, out, diagnostics)
    }
}

#[derive(Debug)]
pub struct PseudoBeamerRuleGen<'a> {
    cfg: &'a Config,
    range: SourceRange,
}

impl<'a> StatefulCodeGenUnit<'a, SlidesFfmpegEspeak, ()> for PseudoBeamerRuleGen<'a> {
    fn new(
        cfg: &'a Config, WithRange(_, range): WithRange<()>,
        _gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
    ) -> Result<Self> {
        Ok(PseudoBeamerRuleGen { cfg, range })
    }

    fn finish(
        self, gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let PseudoBeamerRuleGen { cfg, range } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        backend.slides.close_until(2, &mut out, range, diagnostics)?;
        backend.slides.open_until(2, cfg, &mut out, range, diagnostics)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PseudoBeamerHeaderGen<'a> {
    cfg: &'a Config,
    level: i32,
    label: WithRange<Cow<'a, str>>,
    range: SourceRange,
}

impl<'a> StatefulCodeGenUnit<'a, SlidesFfmpegEspeak, Header<'a>> for PseudoBeamerHeaderGen<'a> {
    fn new(
        cfg: &'a Config, header: WithRange<Header<'a>>,
        gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
    ) -> Result<Self> {
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        let WithRange(Header { label, level }, range) = header;

        // close old slide / beamerboxesrounded
        backend.slides.close_until(level, &mut out, range, diagnostics)?;

        write!(out, "\\{}section{{", "sub".repeat(level as usize - 1))?;

        Ok(PseudoBeamerHeaderGen { cfg, level, label, range })
    }

    fn finish(
        self, gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        let PseudoBeamerHeaderGen { cfg, level, label, range } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        writeln!(out, "}}\\label{{{}}}\n", label.0)?;

        backend.slides.open_until(level, cfg, &mut out, range, diagnostics)?;
        Ok(())
    }
}
