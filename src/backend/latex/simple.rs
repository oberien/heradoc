use std::io::{Result, Write};
use std::borrow::Cow;

use crate::backend::{SimpleCodeGenUnit, MediumCodeGenUnit, Backend};
use crate::backend::latex::InlineEnvironment;
use crate::generator::event::{FootnoteReference, Link, Image, Pdf, TaskListMarker};
use crate::generator::Stack;
use super::replace::replace;

#[derive(Debug)]
pub struct TextGen;

impl<'a> MediumCodeGenUnit<Cow<'a, str>> for TextGen {
    fn gen<'b, 'c>(text: Cow<'a, str>, stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>) -> Result<()> {
        // TODO: make code-blocks containing unicode allow inline-math
        // handle unicode
        let strfn = if stack.iter().any(|e| e.is_math()) {
            fn a(s: &str) -> &str { &s[1..s.len() - 1] }
            a
        } else {
            fn a(s: &str) -> &str { s }
            a
        };

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
                c if in_graphviz => s.push(c),
                c => match replace(c) {
                    Some(rep) => s.push_str(strfn(rep)),
                    None => s.push(c),
                }
            }
        }
        write!(stack.get_out(), "{}", s)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LatexGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for LatexGen {
    fn gen(latex: Cow<'a, str>, out: &mut impl Write) -> Result<()> {
        write!(out, "{}", latex)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FootnoteReferenceGen;

impl<'a> SimpleCodeGenUnit<FootnoteReference<'a>> for FootnoteReferenceGen {
    fn gen(fnote: FootnoteReference, out: &mut impl Write) -> Result<()> {
        let FootnoteReference { label } = fnote;
        write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", label)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LinkGen;

impl<'a> SimpleCodeGenUnit<Link<'a>> for LinkGen {
    fn gen(link: Link<'a>, out: &mut impl Write) -> Result<()> {
        Ok(match link {
            Link::BiberSingle(reference, rest) => match rest {
                Some(rest) => write!(out, "\\cite[{}]{{{}}}", rest, reference)?,
                None => write!(out, "\\cite{{{}}}", reference)?,
            }
            Link::BiberMultiple(vec) => {
                write!(out, "\\cites")?;
                for (reference, rest) in vec {
                    match rest {
                        Some(rest) => write!(out, "[{}]{{{}}}", rest, reference)?,
                        None => write!(out, "{{{}}}", reference)?,
                    }
                }
            }
            Link::Url(dst, None) => write!(out, "\\url{{{}}}", dst)?,
            Link::Url(dst, Some(title)) => write!(out, "\\pdftooltip{{\\url{{{}}}}}{{{}}}", dst, title)?,
            Link::UrlWithContent(dst, content, None) => write!(out, "\\href{{{}}}{{{}}}", dst, content)?,
            Link::UrlWithContent(dst, content, Some(title)) => write!(out, "\\pdftooltip{{\\href{{{}}}{{{}}}}}{{{}}}", dst, content, title)?,
            Link::InterLink(label, uppercase) => match uppercase {
                true => write!(out, "\\Cref{{{}}}", label)?,
                false => write!(out, "\\cref{{{}}}", label)?,
            }
            Link::InterLinkWithContent(label, _uppercase, content)
                => write!(out, "\\hyperref[{}]{{{}}}", label, content)?,
        })
    }
}

#[derive(Debug)]
pub struct ImageGen;

impl<'a> SimpleCodeGenUnit<Image<'a>> for ImageGen {
    fn gen(image: Image<'a>, out: &mut impl Write) -> Result<()> {
        let Image { label, caption, title, alt_text, path, scale, width, height } = image;
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        inline_fig.write_begin(&mut*out)?;

        if title.is_some() {
            writeln!(out, "\\pdftooltip{{")?;
        }
        if alt_text.is_some() {
            write!(out, "\\imagewithtext[")?;
        } else {
            write!(out, "\\includegraphics[")?;
        }

        if let Some(scale) = scale {
            write!(out, "scale={}", scale)?;
        }
        if let Some(width) = width {
            write!(out, "width={},", width)?;
        }
        if let Some(height) = height {
            write!(out, "height={},", height)?;
        }

        if let Some(alt_text) = alt_text {
            writeln!(out, "]{{{}}}{{{}}}", path.display(), alt_text)?;
        } else {
            writeln!(out, "]{{{}}}", path.display())?;
        }

        if let Some(title) = title {
            writeln!(out, "}}{{{}}}", title)?;
        }

        inline_fig.write_end(out)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LabelGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for LabelGen {
    fn gen(label: Cow<'a, str>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\label{{{}}}", label)
    }
}

#[derive(Debug)]
pub struct PdfGen;

impl SimpleCodeGenUnit<Pdf> for PdfGen {
    fn gen(pdf: Pdf, out: &mut impl Write) -> Result<()> {
        let Pdf { path } = pdf;

        writeln!(out, "\\includepdf[pages=-]{{{}}}", path.display())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SoftBreakGen;

impl SimpleCodeGenUnit<()> for SoftBreakGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        // soft breaks are only used to split up text in lines in the source file
        // so it's nothing we should translate, but for better readability keep them
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HardBreakGen;

impl MediumCodeGenUnit<()> for HardBreakGen {
    fn gen<'b, 'c>((): (), stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>) -> Result<()> {
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

#[derive(Debug)]
pub struct TaskListMarkerGen;

impl SimpleCodeGenUnit<TaskListMarker> for TaskListMarkerGen {
    fn gen(marker: TaskListMarker, out: &mut impl Write) -> Result<()> {
        let TaskListMarker { checked } = marker;
        match checked {
            true => write!(out, r"[$\boxtimes$] ")?,
            false => write!(out, r"[$\square$] ")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct TableOfContentsGen;

impl SimpleCodeGenUnit<()> for TableOfContentsGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\tableofcontents")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BibliographyGen;

impl SimpleCodeGenUnit<()> for BibliographyGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        // TODO: config option if bibliography in toc
        // TODO: config option for title
        writeln!(out, "\\printbibliography[heading=bibintoc]")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfTablesGen;

impl SimpleCodeGenUnit<()> for ListOfTablesGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\listoftables")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfFiguresGen;

impl SimpleCodeGenUnit<()> for ListOfFiguresGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\listoffigures")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfListingsGen;

impl SimpleCodeGenUnit<()> for ListOfListingsGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\lstlistoflistings")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct AppendixGen;

impl SimpleCodeGenUnit<()> for AppendixGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\appendix{{}}")?;
        writeln!(out, "\\renewcommand\\thelstlisting{{\\Alph{{lstlisting}}}}")?;
        writeln!(out, "\\setcounter{{lstlisting}}{{0}}")?;
        Ok(())
    }
}
