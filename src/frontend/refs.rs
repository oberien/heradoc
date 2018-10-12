use std::borrow::Cow;
use std::fmt;

pub use pulldown_cmark::LinkType;

use crate::config::Config;
use crate::cow_ext::CowExt;

#[derive(Debug, Clone)]
pub enum Link<'a> {
    /// reference, attributes
    BiberSingle(Cow<'a, str>, Option<Cow<'a, str>>),
    /// Vec<(reference, attributes)>
    BiberMultiple(Vec<(Cow<'a, str>, Option<Cow<'a, str>>)>),
    Url(Cow<'a, str>),
    /// destination, content (already converted)
    UrlWithContent(Cow<'a, str>, String),
    InterLink(LabelReference<'a>),
    /// label, content (already converted)
    InterLinkWithContent(LabelReference<'a>, String),
}

// TODO: autoparse / autoformat labels

#[derive(Debug, Clone)]
pub struct LabelReference<'a> {
    pub label: Label<'a>,
    /// "in tbl. 42" vs "Tbl. 42 shows"
    pub uppercase: bool,
}

#[derive(Debug, Clone)]
pub struct Label<'a> {
    pub label: Cow<'a, str>,
    pub typ: LabelType,
}

#[derive(Debug, Clone, Copy)]
pub enum LabelType {
    Section,
    Image,
    Figure,
    Footnote,
    Table,
}

trait StrExt {
    fn starts_with_ignore_ascii_case(&self, other: &Self) -> bool;
}

impl StrExt for str {
    fn starts_with_ignore_ascii_case(&self, other: &Self) -> bool {
        // TODO: don't panic if other.len() is not on char boundary of self
        if other.len() >= self.len() {
            return false;
        }
        self[..other.len()].eq_ignore_ascii_case(other)
    }
}

impl LabelType {
    /// length of substring, typ, uppercase
    fn from_str(s: &str) -> Option<(usize, LabelType, bool)> {
        let (len, typ) = match s {
            s if s.starts_with_ignore_ascii_case("#") =>
                // TODO: don't unwrap but handle errornous reference (e.g. "#")
                return Some((1, LabelType::Section, s.chars().nth(1).unwrap().is_uppercase())),
            s if s.starts_with_ignore_ascii_case("sec:") => (4, LabelType::Section),
            s if s.starts_with_ignore_ascii_case("img:") => (4, LabelType::Image),
            s if s.starts_with_ignore_ascii_case("fig:") => (4, LabelType::Figure),
            s if s.starts_with_ignore_ascii_case("fnote:") => (6, LabelType::Footnote),
            s if s.starts_with_ignore_ascii_case("tbl:") => (4, LabelType::Table),
            _ => return None,
        };
        Some((len, typ, s.chars().next().unwrap().is_uppercase()))
    }
}

impl fmt::Display for LabelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LabelType::Section => write!(f, "sec:"),
            LabelType::Image => write!(f, "img:"),
            LabelType::Figure => write!(f, "fig:"),
            LabelType::Footnote => write!(f, "fnote:"),
            LabelType::Table => write!(f, "tbl:"),
        }
    }
}

// TODO: proper label infrastructure (pass `Label` wherever a label should be generated)
impl<'a> Label<'a> {
    fn from_cow(s: Cow<'a, str>) -> Either<Label<'a>, Cow<'a, str>> {
        Either::Left(match LabelReference::from_cow(s) {
            Either::Left(r) => r.label,
            Either::Right(l) => return Either::Right(l),
        })
    }
}

impl<'a> fmt::Display for Label<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.typ.fmt(f)?;
        write!(f, "{}", self.label)
    }
}

enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<'a> LabelReference<'a> {
    fn from_cow(mut s: Cow<'a, str>) -> Either<LabelReference<'a>, Cow<'a, str>> {
        let (len, typ, uppercase) = match LabelType::from_str(&s) {
            Some(l) => l,
            None => return Either::Right(s),
        };
        s.truncate_left(len);
        s.make_ascii_lowercase_inplace();
        Either::Left(LabelReference {
            label: Label {
                label: s,
                typ,
            },
            uppercase,
        })
    }
}

#[derive(Debug)]
pub enum LinkOrText<'a> {
    Link(Link<'a>),
    Text(Cow<'a, str>),
}


pub fn parse_references<'a>(cfg: &'a Config, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, content: String) -> LinkOrText<'a> {
    // ShortcutUnknown and ReferenceUnknown make destination lowercase, but save original
    // case in title
    let dst = match typ {
        LinkType::ShortcutUnknown | LinkType::ReferenceUnknown => title,
        _ => dst,
    };

    // biber
    if cfg.bibliography.is_some() && dst.trim_left().starts_with('@') && typ == LinkType::ShortcutUnknown {
        // TODO: parse biber file and warn on unknown references
        // TODO: don't clone here
        if iter_multiple_biber(dst.clone()).nth(1).is_some() {
            return LinkOrText::Link(Link::BiberMultiple(iter_multiple_biber(dst).collect()));
        } else {
            let (r, a) = parse_single_biber(dst);
            return LinkOrText::Link(Link::BiberSingle(r, a));
        }
    }

    match LabelReference::from_cow(dst) {
        // cref / Cref / hyperlink
        Either::Left(labelref) => match typ {
            LinkType::ShortcutUnknown
            | LinkType::CollapsedUnknown
            | LinkType::Shortcut
            | LinkType::Collapsed => LinkOrText::Link(Link::InterLink(labelref)),
            LinkType::Reference
            | LinkType::ReferenceUnknown
            | LinkType::Autolink
            | LinkType::Inline => LinkOrText::Link(Link::InterLinkWithContent(labelref, content)),
        }
        Either::Right(dst) => match typ {
            LinkType::ShortcutUnknown
            | LinkType::CollapsedUnknown => {
                // TODO: warn on unknown reference and hint to proper syntax `\[foo\]`
                LinkOrText::Text(Cow::Owned(format!("[{}]", content)))
            }
            LinkType::ReferenceUnknown => {
                // TODO: warn on unknown reference and hint to proper syntax `\[foo\]`
                LinkOrText::Text(Cow::Owned(format!("[{}][{}]", content, dst)))
            }
            LinkType::Shortcut | LinkType::Collapsed | LinkType::Autolink
                => LinkOrText::Link(Link::Url(dst)),
            LinkType::Reference | LinkType::Inline
                => LinkOrText::Link(Link::UrlWithContent(dst, content)),
        }
    }
}

fn iter_multiple_biber<'a>(s: Cow<'a, str>) -> impl Iterator<Item = (Cow<'a, str>, Option<Cow<'a, str>>)> {
    struct Iter<'a> {
        s: Cow<'a, str>,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = (Cow<'a, str>, Option<Cow<'a, str>>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.s.is_empty() {
                return None;
            }

            // skip leading whitespace at first reference (`[ @foo...`)
            let leading_whitespace = self.s.chars()
                .take_while(|c| c.is_whitespace())
                .count();
            let single_start = leading_whitespace;
            assert_eq!(&self.s[single_start..single_start+1], "@");

            let next_at = self.s[single_start + 1..].find('@')
                .map(|i| i + single_start + 1)
                .unwrap_or(self.s.len());
            let single_end = self.s[single_start..next_at].rfind(',').unwrap_or(self.s.len());
            let single = self.s.map_inplace_return(
                |s| {
                    let single = Cow::Borrowed(&s[single_start..single_end]);
                    (&s[next_at..], single)
                },
                |s| {
                    let mut single = s.split_off(single_end);
                    ::std::mem::swap(&mut single, s);
                    single.drain(..single_start);
                    s.drain(..next_at - single_end);
                    Cow::Owned(single)
                }
            );
            Some(parse_single_biber(single))
        }
    }

    Iter {
        s,
    }
}

/// Returns (reference, Option<options>)
fn parse_single_biber<'a>(mut s: Cow<'a, str>) -> (Cow<'a, str>, Option<Cow<'a, str>>) {
    s.trim_inplace();
    assert_eq!(&s[..1], "@", "Expected a biber reference starting with `@`, found {:?}", s);

    let spacepos = s.find(' ');
    s.map(
        |s| {
            let reference = &s[1..spacepos.unwrap_or(s.len())];
            let rest = spacepos.map(|pos| Cow::Borrowed(&s[(pos + 1)..]));
            (Cow::Borrowed(reference), rest)
        },
        |mut s| {
            let rest = spacepos.map(|pos| Cow::Owned(s.split_off(pos + 1)));
            let mut reference = s;
            if let Some(pos) = spacepos {
                reference.truncate(pos);
            }
            reference.drain(..1);
            (Cow::Owned(reference), rest)
        }
    )
}
