use std::io::{Result, Write};
use std::borrow::Cow;

use crate::backend::{SimpleCodeGenUnit, MediumCodeGenUnit, Backend};
use crate::generator::event::{FootnoteReference, Link, Image, Pdf, LabelReference};
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
        let mut s = String::with_capacity(text.len() + 20);
        for c in text.chars() {
            match c {
                '_' if in_inline_code => s.push_str("\\char`_"),
                '_' if !in_code_or_math => s.push_str("\\_"),
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
pub struct FootnoteReferenceGen;

impl<'a> SimpleCodeGenUnit<FootnoteReference<'a>> for FootnoteReferenceGen {
    fn gen(fnote: FootnoteReference, out: &mut impl Write) -> Result<()> {
        write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote.label)?;
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
            Link::Url(dst) => write!(out, "\\url{{{}}}", dst)?,
            Link::UrlWithContent(dst, content) => write!(out, "\\href{{{}}}{{{}}}", dst, content)?,
            Link::InterLink(LabelReference { label, uppercase }) => match uppercase {
                true => write!(out, "\\Cref{{{}}}", label)?,
                false => write!(out, "\\cref{{{}}}", label)?,
            }
            Link::InterLinkWithContent(labelref, content)
                => write!(out, "\\hyperref[{}]{{{}}}", labelref.label, content)?,
        })
    }
}

#[derive(Debug)]
pub struct ImageGen;

impl<'a> SimpleCodeGenUnit<Image<'a>> for ImageGen {
    fn gen(image: Image<'a>, out: &mut impl Write) -> Result<()> {
        let Image { path, caption, width, height } = image;

        writeln!(out, "\\begin{{figure}}")?;
        write!(out, "\\includegraphics[")?;
        if let Some(width) = width {
            write!(out, "width={},", width)?;
        }
        if let Some(height) = height {
            write!(out, "height={},", height)?;
        }
        writeln!(out, "]{{{}}}", path.display())?;

        if let Some(caption) = caption {
            writeln!(out, "\\caption{{{}}}", caption)?;
        }
        // TODO: label
//        if !self.title.is_empty() {
//            writeln!(out, "\\label{{img:{}}}", self.title)?;
//        }
        writeln!(out, "\\end{{figure}}")?;
        Ok(())
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

impl SimpleCodeGenUnit<()> for HardBreakGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\\\")?;
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
