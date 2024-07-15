use std::borrow::Cow;
use std::io::Write;
use std::fs::{File, OpenOptions};
use diagnostic::{Span, Spanned};

use crate::backend::latex::{Beamer, BeamerFrameEvent};
use crate::backend::{Backend, CodeGenUnit, StatefulCodeGenUnit};
use crate::config::Config;
use crate::Diagnostics;
use crate::error::{DiagnosticCode, Error, Fatal, FatalResult, Result};
use crate::generator::event::{CodeBlock, Event, Header};
use crate::generator::Generator;

/// Combines beamer structure but forks off some of the comments into separate files where we will
/// later read it and generate commentary with `espeak`/`espeak-ng`. This is a generator as slides
/// and audio will need to be synchronized.
#[derive(Debug)]
pub struct SlidesFfmpegEspeak {
    current_frame: CurrentFrame,
    slides: Beamer,
}

#[derive(Debug)]
struct CurrentFrame(u32);

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
    type PageBreak = PseudoBeamerPageBreakGen<'a>;
    type Rule = <Beamer as Backend<'a>>::Rule;
    type TaskListMarker = <Beamer as Backend<'a>>::TaskListMarker;
    type TableOfContents = <Beamer as Backend<'a>>::TableOfContents;
    type Bibliography = <Beamer as Backend<'a>>::Bibliography;
    type ListOfTables = <Beamer as Backend<'a>>::ListOfTables;
    type ListOfFigures = <Beamer as Backend<'a>>::ListOfFigures;
    type ListOfListings = <Beamer as Backend<'a>>::ListOfListings;
    type Appendix = <Beamer as Backend<'a>>::Appendix;

    type Paragraph = <Beamer as Backend<'a>>::Paragraph;
    type Header = PseudoBeamerHeaderGen<'a>;
    type BlockQuote = <Beamer as Backend<'a>>::BlockQuote;
    type CodeBlock = CodeBlockGen;
    type List = <Beamer as Backend<'a>>::List;
    type Enumerate = <Beamer as Backend<'a>>::Enumerate;
    type Item = <Beamer as Backend<'a>>::Item;
    type FootnoteDefinition = <Beamer as Backend<'a>>::FootnoteDefinition;
    type UrlWithContent = <Beamer as Backend<'a>>::UrlWithContent;
    type InterLinkWithContent = <Beamer as Backend<'a>>::InterLinkWithContent;
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
            current_frame: CurrentFrame(0),
            slides: Beamer::new(),
        }
    }

    fn gen_preamble(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &'a Diagnostics) -> FatalResult<()> {
        self.slides.gen_preamble(cfg, out, diagnostics)
    }

    fn gen_epilogue(&mut self, cfg: &Config, out: &mut impl Write, diagnostics: &'a Diagnostics) -> FatalResult<()> {
        self.slides.gen_epilogue(cfg, out, diagnostics)
    }
}

impl SlidesFfmpegEspeak {
    fn create_speech_file(&self, cfg: &Config, diagnostics: &Diagnostics) -> Result<File> {
        let i = self.current_frame.get();
        let p = cfg.out_dir.join(format!("espeak_{}.txt", i));
        let res = OpenOptions::new().create(true).write(true).open(&p);
        match res {
            Ok(file) => Ok(file),
            Err(e) => {
                diagnostics
                    .error(DiagnosticCode::EspeakCreationError)
                    .with_note(format!("error creating espeak file `{}` for frame {}", p.display(), i))
                    .with_note(format!("cause: {}", e))
                    .with_note("this is fatal")
                    .emit();
                Err(Error::Fatal(Fatal::Output(e)))
            },
        }
    }
}

impl CurrentFrame {
    const fn get(&self) -> u32 {
        self.0
    }
}

impl CurrentFrame {
    fn advance_with<Iter: IntoIterator<Item=BeamerFrameEvent>>(&mut self, iter: Iter) {
        for item in iter {
            if let BeamerFrameEvent::EndFrame = item {
                self.0 += 1;
            }
        }
    }
}

#[derive(Debug)]
pub struct PseudoBeamerPageBreakGen<'a> {
    cfg: &'a Config,
    span: Span,
}

impl<'a> StatefulCodeGenUnit<'a, SlidesFfmpegEspeak, ()> for PseudoBeamerPageBreakGen<'a> {
    fn new(
        cfg: &'a Config, Spanned { span, .. }: Spanned<()>,
        _gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
    ) -> Result<Self> {
        Ok(PseudoBeamerPageBreakGen { cfg, span })
    }

    fn finish(
        self, gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        let PseudoBeamerPageBreakGen { cfg, span } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        let events: Vec<_> = backend.slides.close_until(2, &mut out, span, diagnostics)?;
        backend.current_frame.advance_with(events);
        let events: Vec<_> = backend.slides.open_until(2, cfg, &mut out, span, diagnostics)?;
        backend.current_frame.advance_with(events);
        Ok(())
    }
}

#[derive(Debug)]
pub struct PseudoBeamerHeaderGen<'a> {
    cfg: &'a Config,
    level: i32,
    label: Spanned<Cow<'a, str>>,
    span: Span,
}

impl<'a> StatefulCodeGenUnit<'a, SlidesFfmpegEspeak, Header<'a>> for PseudoBeamerHeaderGen<'a> {
    fn new(
        cfg: &'a Config, header: Spanned<Header<'a>>,
        gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
    ) -> Result<Self> {
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        let Spanned { value: Header { label, level }, span } = header;

        // close old slide / beamerboxesrounded
        let events: Vec<_> = backend.slides.close_until(level, &mut out, span, diagnostics)?;
        backend.current_frame.advance_with(events);

        write!(out, "\\{}section{{", "sub".repeat(level as usize - 1))?;

        Ok(PseudoBeamerHeaderGen { cfg, level, label, span })
    }

    fn finish(
        self, gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        let PseudoBeamerHeaderGen { cfg, level, label, span } = self;
        let (diagnostics, backend, mut out) = gen.backend_and_out();
        writeln!(out, "}}\\label{{{}}}\n", label.value)?;

        let events: Vec<_> = backend.slides.open_until(level, cfg, &mut out, span, diagnostics)?;
        backend.current_frame.advance_with(events);
        Ok(())
    }
}

#[derive(Debug)]
pub enum CodeBlockGen {
    Speech(File),
    Normal(<Beamer as Backend<'static>>::CodeBlock),
}

impl<'a> StatefulCodeGenUnit<'a, SlidesFfmpegEspeak, CodeBlock<'a>> for CodeBlockGen {
    fn new(
        cfg: &'a Config, code_block: Spanned<CodeBlock<'a>>,
        gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: CodeBlock { label: _, caption: _, language, basicstyle: _ }, .. } = &code_block;

        if let Some(Spanned { value: language, .. }) = language {
            if language.as_ref() == "espeak" {
                let (diagnostics, backend, _) = gen.backend_and_out();
                return backend.create_speech_file(cfg, diagnostics).map(CodeBlockGen::Speech);
            }
        }

        CodeGenUnit::new(cfg, code_block, gen)
            .map(CodeBlockGen::Normal)
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        match self {
            CodeBlockGen::Speech(file) => Some(file),
            CodeBlockGen::Normal(inner) => CodeGenUnit::output_redirect(inner),
        }
    }

    fn finish(
        self, gen: &mut Generator<'a, SlidesFfmpegEspeak, impl Write>,
        peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        match self {
            CodeBlockGen::Normal(inner) => CodeGenUnit::finish(inner, gen, peek),
            CodeBlockGen::Speech(_) => Ok(()),
        }
    }
}
