use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::LinkType;

use crate::gen::{CodeGenUnit, CodeGenUnits, Generator, Backend};
use crate::config::Config;
use crate::parser::{Event, Link};

#[derive(Debug)]
pub struct LinkGen<'a> {
    cfg: &'a Config,
    typ: LinkType,
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    text: Vec<u8>,
}

impl<'a> CodeGenUnit<'a, Link<'a>> for LinkGen<'a> {
    fn new(cfg: &'a Config, link: Link<'a>, gen: &mut Generator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let Link { typ, dst, title } = link;
        Ok(LinkGen {
            cfg,
            typ,
            dst,
            title,
            text: Vec::new(),
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.text)
    }

    fn finish(self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
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
        if self.cfg.bibliography.is_some()
            && dst.starts_with('@')
            && self.typ == LinkType::ShortcutUnknown
        {
            // TODO: parse biber file and warn on unknown references
            // TODO: make space before cite nobreakspace (`~`)
            if iter_multiple_biber(&dst).nth(1).is_some() {
                write!(out, "\\cites")?;
                for (reference, rest) in iter_multiple_biber(&dst) {
                    match rest {
                        Some(rest) => write!(out, "[{}]{{{}}}", rest, reference)?,
                        None => write!(out, "{{{}}}", reference)?,
                    }
                }
            } else {
                let (reference, rest) = parse_single_biber(&dst);
                match rest {
                    Some(rest) => write!(out, "\\cite[{}]{{{}}}", rest, reference)?,
                    None => write!(out, "\\cite{{{}}}", reference)?,
                }
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
            | LinkType::Shortcut
            | LinkType::Collapsed => if uppercase {
                write!(out, "\\Cref{{{}}}", label)?;
            } else {
                write!(out, "\\cref{{{}}}", label)?;
            }
            LinkType::Reference
            | LinkType::ReferenceUnknown
            | LinkType::Autolink
            | LinkType::Inline =>
                write!(out, "\\hyperref[{}]{{{}}}", label, text)?,
        }
        Ok(())
    }
}

fn iter_multiple_biber(s: &str) -> impl Iterator<Item = (&str, Option<&str>)> {
    struct Iter<'a> {
        s: &'a str,
        /// index before the next @
        next_at: usize,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = (&'a str, Option<&'a str>);

        fn next(&mut self) -> Option<<Self as Iterator>::Item> {
            if self.next_at >= self.s.len() {
                return None;
            }

            // skip leading whitespace at first reference (`[ @foo...`)
            let leading_whitespace = (&self.s[self.next_at..]).chars()
                .take_while(|c| c.is_whitespace())
                .count();
            self.next_at += leading_whitespace;
            assert_eq!(&self.s[self.next_at..self.next_at+1], "@");

            let rest = &self.s[self.next_at..];

            let next_at = rest[1..].find('@')
                .map(|i| i + 1)
                .unwrap_or(rest.len());
            let next_comma = rest[..next_at].rfind(',').unwrap_or(rest.len());
            let single = &rest[..next_comma];
            self.next_at += next_at;
            Some(parse_single_biber(single))
        }
    }

    Iter {
        s,
        next_at: 0,
    }
}

/// Returns (reference, Option<options>)
fn parse_single_biber(s: &str) -> (&str, Option<&str>) {
    let s = s.trim();
    assert_eq!(&s[..1], "@");

    let spacepos = s.find(' ');
    let reference = &s[1..spacepos.unwrap_or(s.len())];
    let rest = spacepos.map(|pos| &s[(pos + 1)..]);
    (reference, rest)
}
