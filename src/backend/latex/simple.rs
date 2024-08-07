use std::borrow::Cow;
use std::io::Write;
use std::path::Path;
use diagnostic::Spanned;

use super::replace::replace;
use crate::backend::latex::InlineEnvironment;
use crate::backend::{Backend, MediumCodeGenUnit, SimpleCodeGenUnit};
use crate::config::Config;
use crate::error::{Result, Error, DiagnosticCode};
use crate::generator::event::{
    BiberReference,
    FootnoteReference,
    Image,
    Svg,
    InterLink,
    Pdf,
    TaskListMarker,
    Url,
};
use crate::generator::Stack;
use crate::util::ToUnix;

#[derive(Debug, Default)]
pub struct TextGen;

impl<'a> MediumCodeGenUnit<Cow<'a, str>> for TextGen {
    fn gen<'b, 'c>(
        text: Spanned<Cow<'a, str>>, _config: &Config, stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>,
    ) -> Result<()> {
        let Spanned { value: text, .. } = text;
        // TODO: make code-blocks containing unicode allow inline-math
        // handle unicode
        let strfn: fn(&str) -> &str =
            if stack.iter().any(|e| e.is_math()) { |s| &s[1..s.len() - 1] } else { |s| s };

        let in_inline_code = stack.iter().any(|e| e.is_inline_code());
        let in_code_or_math = stack.iter().any(|e| e.is_code() || e.is_math());
        let in_graphviz = stack.iter().any(|e| e.is_graphviz());
        let mut s = String::with_capacity(text.len() + 20);
        for c in text.chars() {
            match c {
                '#' if in_inline_code || !in_code_or_math => s.push_str("\\#"),
                '$' if in_inline_code || !in_code_or_math => s.push_str("\\$"),
                '%' if in_inline_code || !in_code_or_math => s.push_str("\\%"),
                '&' if in_inline_code || !in_code_or_math => s.push_str("\\&"),
                '~' if in_inline_code => s.push_str("\\char`~{}"),
                '~' if !in_code_or_math => s.push_str("\\textasciitilde{}"),
                '_' if in_inline_code => s.push_str("\\char`_{}"),
                '_' if !in_code_or_math => s.push_str("\\_"),
                '^' if in_inline_code => s.push_str("\\char`^{}"),
                '^' if !in_code_or_math => s.push_str("\\textasciicircum{}"),
                '\\' if in_inline_code || !in_code_or_math => s.push_str("\\textbackslash{}"),
                '{' if in_inline_code => s.push_str("\\char`{{}"),
                '{' if !in_code_or_math => s.push_str("\\{"),
                '}' if in_inline_code => s.push_str("\\char`}{}"),
                '}' if !in_code_or_math => s.push_str("\\}"),
                '✔' => s.push_str("\\checkmark{}"),
                '✘' => s.push_str("\\text{X}"),
                '’' => s.push('\''),
                '"' if in_inline_code => s.push_str("\\\""),
                c if in_graphviz => s.push(c),
                c => match replace(c) {
                    Some(rep) => s.push_str(strfn(rep)),
                    None => s.push(c),
                },
            }
        }
        write!(stack.get_out(), "{}", s)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LatexGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for LatexGen {
    fn gen(latex: Spanned<Cow<'a, str>>, out: &mut impl Write) -> Result<()> {
        write!(out, "{}", latex.value)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct FootnoteReferenceGen;

impl<'a> SimpleCodeGenUnit<FootnoteReference<'a>> for FootnoteReferenceGen {
    fn gen(fnote: Spanned<FootnoteReference<'a>>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: FootnoteReference { label }, .. } = fnote;
        write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", label)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct BiberReferencesGen;

impl<'a> SimpleCodeGenUnit<Vec<BiberReference<'a>>> for BiberReferencesGen {
    fn gen(
        biber: Spanned<Vec<BiberReference<'a>>>, out: &mut impl Write,
    ) -> Result<()> {
        let Spanned { value: mut biber, .. } = biber;
        if biber.len() == 1 {
            let BiberReference { reference, attributes } = biber.pop().unwrap();
            match attributes {
                Some(attrs) => write!(out, "\\cite[{}]{{{}}}", attrs, reference)?,
                None => write!(out, "\\cite{{{}}}", reference)?,
            }
        } else {
            write!(out, "\\cites")?;
            for BiberReference { reference, attributes } in biber {
                match attributes {
                    Some(rest) => write!(out, "[{}]{{{}}}", rest, reference)?,
                    None => write!(out, "{{{}}}", reference)?,
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UrlGen;

impl<'a> SimpleCodeGenUnit<Url<'a>> for UrlGen {
    fn gen(url: Spanned<Url<'a>>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: Url { destination, title }, .. } = url;
        match title {
            None => write!(out, "\\url{{{}}}", destination)?,
            Some(title) => write!(out, "\\pdftooltip{{\\url{{{}}}}}{{{}}}", destination, title)?,
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct InterLinkGen;

impl<'a> SimpleCodeGenUnit<InterLink<'a>> for InterLinkGen {
    fn gen(interlink: Spanned<InterLink<'a>>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: InterLink { label, uppercase }, .. } = interlink;
        match uppercase {
            true => write!(out, "\\Cref{{{}}}", label)?,
            false => write!(out, "\\cref{{{}}}", label)?,
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ImageGen;

impl<'a> SimpleCodeGenUnit<Image<'a>> for ImageGen {
    fn gen(image: Spanned<Image<'a>>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: Image { label, caption, title, alt_text, path, scale, width, height }, .. } = image;
        includegraphics(out, label, caption, title, alt_text, path, scale, width, height)?;
        Ok(())
    }
}

fn includegraphics(out: &mut impl Write, label: Option<Spanned<Cow<'_, str>>>,
    caption: Option<Spanned<Cow<'_, str>>>, title: Option<Cow<'_, str>>, alt_text: Option<String>,
    path: impl AsRef<Path>, scale: Option<Spanned<Cow<'_, str>>>, width: Option<Spanned<Cow<'_, str>>>,
    height: Option<Spanned<Cow<'_, str>>>,
) -> Result<()> {
    let inline_fig = InlineEnvironment::new_figure(label, caption);
    inline_fig.write_begin(&mut *out)?;

    if title.is_some() {
        writeln!(out, "\\pdftooltip{{")?;
    }
    if alt_text.is_some() {
        write!(out, "\\imagewithtext[")?;
    } else {
        write!(out, "\\includegraphics[")?;
    }

    if let Some(Spanned { value: scale, .. }) = scale {
        write!(out, "scale={}", scale)?;
    }
    if let Some(Spanned { value: width, .. }) = width {
        write!(out, "width={},", width)?;
    }
    if let Some(Spanned { value: height, .. }) = height {
        write!(out, "height={},", height)?;
    }

    write!(out, "]{{{}}}", path.as_ref().to_unix()
        .expect(&format!("non-utf8 path: {:?}", path.as_ref())))?;

    if let Some(alt_text) = alt_text {
        write!(out, "{{{}}}", alt_text)?;
    }

    writeln!(out)?;

    if let Some(title) = title {
        writeln!(out, "}}{{{}}}", title)?;
    }

    inline_fig.write_end(out)?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct SvgGen;

impl<'a> MediumCodeGenUnit<Svg<'a>> for SvgGen {
    fn gen<'b, 'c>(svg: Spanned<Svg<'a>>, config: &Config, stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>) -> Result<()> {
        let pdf_path = match svg.value.to_pdf_path(&config.out_dir) {
            Ok(path) => path,
            Err(e) => {
                stack.diagnostics().error(DiagnosticCode::SvgConversionError)
                    .with_info_label(svg.span, "can't convert svg to pdf")
                    .with_note(format!("{}", e))
                    .emit();
                return Err(Error::Diagnostic);
            },
        };
        let Spanned { value: Svg { label, caption, title, alt_text, path: _, scale, width, height }, .. } = svg;
        includegraphics(&mut stack.get_out(), label, caption, title, alt_text, pdf_path, scale, width, height)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LabelGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for LabelGen {
    fn gen(label: Spanned<Cow<'a, str>>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: label, .. } = label;
        writeln!(out, "\\label{{{}}}", label)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PdfGen;

impl SimpleCodeGenUnit<Pdf> for PdfGen {
    fn gen(pdf: Spanned<Pdf>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: Pdf { path }, .. } = pdf;

        writeln!(out, "\\includepdf[pages=-]{{{}}}", path.to_unix()
            .expect(&format!("non-utf8 path: {:?}", path)))?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SoftBreakGen;

impl SimpleCodeGenUnit<()> for SoftBreakGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        // soft breaks are only used to split up text in lines in the source file
        // so it's nothing we should translate, but for better readability keep them
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct HardBreakGen;

impl MediumCodeGenUnit<()> for HardBreakGen {
    fn gen<'b, 'c>(
        _: Spanned<()>, _config: &Config, stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>,
    ) -> Result<()> {
        let in_table = stack.iter().any(|e| e.is_table());
        let out = stack.get_out();

        if in_table {
            write!(out, "\\newline")?;
        } else {
            writeln!(out, "\\\\")?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct RuleGen;

impl<'a> SimpleCodeGenUnit<()> for RuleGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out)?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out, "\\hrule")?;
        writeln!(out, "\\vspace{{1em}}")?;
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PageBreakGen;

impl SimpleCodeGenUnit<()> for PageBreakGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\newpage")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct TaskListMarkerGen;

impl SimpleCodeGenUnit<TaskListMarker> for TaskListMarkerGen {
    fn gen(marker: Spanned<TaskListMarker>, out: &mut impl Write) -> Result<()> {
        let Spanned { value: TaskListMarker { checked }, .. } = marker;
        match checked {
            true => write!(out, r"[$\boxtimes$] ")?,
            false => write!(out, r"[$\square$] ")?,
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct TableOfContentsGen;

impl SimpleCodeGenUnit<()> for TableOfContentsGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\tableofcontents")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct BibliographyGen;

impl SimpleCodeGenUnit<()> for BibliographyGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        // TODO: config option if bibliography in toc
        // TODO: config option for title
        writeln!(out, "\\printbibliography[heading=bibintoc]")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ListOfTablesGen;

impl SimpleCodeGenUnit<()> for ListOfTablesGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\listoftables")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ListOfFiguresGen;

impl SimpleCodeGenUnit<()> for ListOfFiguresGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\listoffigures")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ListOfListingsGen;

impl SimpleCodeGenUnit<()> for ListOfListingsGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\lstlistoflistings")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AppendixGen;

impl SimpleCodeGenUnit<()> for AppendixGen {
    fn gen(_: Spanned<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\appendix{{}}")?;
        writeln!(out, "\\renewcommand\\thelstlisting{{\\Alph{{lstlisting}}}}")?;
        writeln!(out, "\\setcounter{{lstlisting}}{{0}}")?;
        Ok(())
    }
}
