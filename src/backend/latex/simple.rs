use std::borrow::Cow;
use std::io::Write;

use super::replace::replace;
use crate::backend::latex::InlineEnvironment;
use crate::backend::{Backend, MediumCodeGenUnit, SimpleCodeGenUnit};
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{
    BiberReference,
    FootnoteReference,
    Image,
    InterLink,
    Pdf,
    TaskListMarker,
    Url,
};
use crate::generator::Stack;

#[derive(Debug)]
pub struct TextGen;

impl<'a> MediumCodeGenUnit<Cow<'a, str>> for TextGen {
    fn gen<'b, 'c>(
        text: WithRange<Cow<'a, str>>, stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>,
    ) -> Result<()> {
        let WithRange(text, _range) = text;
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

#[derive(Debug)]
pub struct LatexGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for LatexGen {
    fn gen(latex: WithRange<Cow<'a, str>>, out: &mut impl Write) -> Result<()> {
        write!(out, "{}", latex.element())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FootnoteReferenceGen;

impl<'a> SimpleCodeGenUnit<FootnoteReference<'a>> for FootnoteReferenceGen {
    fn gen(fnote: WithRange<FootnoteReference<'a>>, out: &mut impl Write) -> Result<()> {
        let WithRange(FootnoteReference { label }, _range) = fnote;
        write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", label)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BiberReferencesGen;

impl<'a> SimpleCodeGenUnit<Vec<BiberReference<'a>>> for BiberReferencesGen {
    fn gen(
        biber: WithRange<Vec<BiberReference<'a>>>, out: &mut impl Write,
    ) -> Result<()> {
        let WithRange(mut biber, _range) = biber;
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

#[derive(Debug)]
pub struct UrlGen;

impl<'a> SimpleCodeGenUnit<Url<'a>> for UrlGen {
    fn gen(url: WithRange<Url<'a>>, out: &mut impl Write) -> Result<()> {
        let WithRange(Url { destination, title }, _range) = url;
        match title {
            None => write!(out, "\\url{{{}}}", destination)?,
            Some(title) => write!(out, "\\pdftooltip{{\\url{{{}}}}}{{{}}}", destination, title)?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct InterLinkGen;

impl<'a> SimpleCodeGenUnit<InterLink<'a>> for InterLinkGen {
    fn gen(interlink: WithRange<InterLink<'a>>, out: &mut impl Write) -> Result<()> {
        let WithRange(InterLink { label, uppercase }, _range) = interlink;
        match uppercase {
            true => write!(out, "\\Cref{{{}}}", label)?,
            false => write!(out, "\\cref{{{}}}", label)?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ImageGen;

impl<'a> SimpleCodeGenUnit<Image<'a>> for ImageGen {
    fn gen(image: WithRange<Image<'a>>, out: &mut impl Write) -> Result<()> {
        let WithRange(Image { label, caption, title, alt_text, path, scale, width, height }, _range) = image;
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

        if let Some(WithRange(scale, _)) = scale {
            write!(out, "scale={}", scale)?;
        }
        if let Some(WithRange(width, _)) = width {
            write!(out, "width={},", width)?;
        }
        if let Some(WithRange(height, _)) = height {
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
    fn gen(label: WithRange<Cow<'a, str>>, out: &mut impl Write) -> Result<()> {
        let WithRange(label, _range) = label;
        writeln!(out, "\\label{{{}}}", label)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PdfGen;

impl SimpleCodeGenUnit<Pdf> for PdfGen {
    fn gen(pdf: WithRange<Pdf>, out: &mut impl Write) -> Result<()> {
        let WithRange(Pdf { path }, _range) = pdf;

        writeln!(out, "\\includepdf[pages=-]{{{}}}", path.display())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SoftBreakGen;

impl SimpleCodeGenUnit<()> for SoftBreakGen {
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        // soft breaks are only used to split up text in lines in the source file
        // so it's nothing we should translate, but for better readability keep them
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HardBreakGen;

impl MediumCodeGenUnit<()> for HardBreakGen {
    fn gen<'b, 'c>(
        _: WithRange<()>, stack: &mut Stack<'b, 'c, impl Backend<'b>, impl Write>,
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

#[derive(Debug)]
pub struct TaskListMarkerGen;

impl SimpleCodeGenUnit<TaskListMarker> for TaskListMarkerGen {
    fn gen(marker: WithRange<TaskListMarker>, out: &mut impl Write) -> Result<()> {
        let WithRange(TaskListMarker { checked }, _range) = marker;
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
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\tableofcontents")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BibliographyGen;

impl SimpleCodeGenUnit<()> for BibliographyGen {
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        // TODO: config option if bibliography in toc
        // TODO: config option for title
        writeln!(out, "\\printbibliography[heading=bibintoc]")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfTablesGen;

impl SimpleCodeGenUnit<()> for ListOfTablesGen {
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\listoftables")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfFiguresGen;

impl SimpleCodeGenUnit<()> for ListOfFiguresGen {
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\listoffigures")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ListOfListingsGen;

impl SimpleCodeGenUnit<()> for ListOfListingsGen {
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\microtypesetup{{protrusion=false}}")?;
        writeln!(out, "\\lstlistoflistings")?;
        writeln!(out, "\\microtypesetup{{protrusion=true}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct AppendixGen;

impl SimpleCodeGenUnit<()> for AppendixGen {
    fn gen(_: WithRange<()>, out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\appendix{{}}")?;
        writeln!(out, "\\renewcommand\\thelstlisting{{\\Alph{{lstlisting}}}}")?;
        writeln!(out, "\\setcounter{{lstlisting}}{{0}}")?;
        Ok(())
    }
}
