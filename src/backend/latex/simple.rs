use std::io::{Result, Write};
use std::borrow::Cow;

use crate::backend::SimpleCodeGenUnit;
use crate::generator::event::{FootnoteReference, Link, Image, Pdf, LabelReference};

#[derive(Debug)]
pub struct TextGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for TextGen {
    fn gen(text: Cow<'a, str>, out: &mut impl Write) -> Result<()> {
        write!(out, "{}", text)?;
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
        writeln!(out, "\\par")?;
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
        writeln!(out, "\\printbibliography")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfTablesGen;

impl SimpleCodeGenUnit<()> for ListOfTablesGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\listoftables")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfFiguresGen;

impl SimpleCodeGenUnit<()> for ListOfFiguresGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\listoffigures")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfListingsGen;

impl SimpleCodeGenUnit<()> for ListOfListingsGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\lstlistoflistings")?;
        Ok(())
    }
}
