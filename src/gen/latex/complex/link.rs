use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::{Tag, Event, LinkType};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct Link<'a> {
    typ: LinkType,
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    text: Vec<u8>,
}

impl<'a> State<'a> for Link<'a> {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        let (typ, dst, title) = match tag {
            Tag::Link(typ, dst, title) => (typ, dst, title),
            _ => unreachable!(),
        };
        Ok(Link {
            typ,
            dst,
            title,
            text: Vec::new(),
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.text)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
        let text = String::from_utf8(self.text).expect("invalid UTF8");
        // TODO: use title

        // ShortcutUnknown and ReferenceUnknown make destination lowercase, but save original
        // case in title
        let dst = match self.typ {
            LinkType::ShortcutUnknown | LinkType::ReferenceUnknown => &self.title,
            _ => &self.dst,
        };
        let dstlower = dst.to_ascii_lowercase();

        // biber
        // TODO: only check for biber references if there is actually a biber file
        if dst.starts_with('@') && self.typ == LinkType::ShortcutUnknown {
            // TODO: parse biber file and warn on unknown references
            let spacepos = dst.find(' ');
            let reference = &dst[1..spacepos.unwrap_or(dst.len())];
            let rest = spacepos.map(|pos| &dst[(pos + 1)..]);

            // TODO: make space before cite nobreakspace (`~`)
            if let Some(rest) = rest {
                write!(out, "\\cite[{}]{{{}}}", rest, reference)?;
            } else {
                write!(out, "\\cite{{{}}}", reference)?;
            }
            return Ok(());
        }

        // cref / Cref / hyperlink
        let (label, uppercase): (Cow<'_, str>, _) = if dst.starts_with('#') {
            // section
            (format!("sec:{}", &dstlower[1..]).into(), dst[1..].chars().next().unwrap().is_uppercase())
        } else if dstlower.starts_with("sec:")
            || dstlower.starts_with("img:")
            || dstlower.starts_with("fig:")
            || dstlower.starts_with("fnote:")
        {
            (dstlower.into(), dst.chars().next().unwrap().is_uppercase())
        } else {
            // nothing special, handle as url / href
            match self.typ {
                LinkType::ShortcutUnknown
                | LinkType::CollapsedUnknown => {
                    // TODO: warn for unknown reference and hint to proper syntax `\[foo\]`
                    write!(out, "[{}]", text)?;
                }
                LinkType::ReferenceUnknown => {
                    // TODO: warn for unknown reference and hint to proper syntax `\[foo\]`
                    write!(out, "[{}][{}]", text, self.dst)?;
                }
                LinkType::Shortcut | LinkType::Collapsed | LinkType::Autolink =>
                    write!(out, "\\url{{{}}}", self.dst)?,
                LinkType::Reference | LinkType::Inline =>
                    write!(out, "\\href{{{}}}{{{}}}", self.dst, text)?,
            }
            return Ok(());
        };

        match self.typ {
            LinkType::ShortcutUnknown
            | LinkType::CollapsedUnknown
            | LinkType::ReferenceUnknown
            | LinkType::Shortcut
            | LinkType::Collapsed => if uppercase {
                write!(out, "\\Cref{{{}}}", label)?;
            } else {
                write!(out, "\\cref{{{}}}", label)?;
            }
            LinkType::Reference | LinkType::Autolink | LinkType::Inline =>
                write!(out, "\\hyperref[{}]{{{}}}", label, text)?,
        }
        Ok(())
    }
}
