use std::borrow::Cow;
use std::str::FromStr;

pub use pulldown_cmark::LinkType;

use crate::config::Config;
use crate::ext::{CowExt, StrExt};
use crate::resolve::Command;

#[derive(Debug, Clone)]
pub enum Link<'a> {
    /// reference, attributes
    BiberSingle(Cow<'a, str>, Option<Cow<'a, str>>),
    /// Vec<(reference, attributes)>
    BiberMultiple(Vec<(Cow<'a, str>, Option<Cow<'a, str>>)>),
    Url(Cow<'a, str>),
    /// destination, content (already converted)
    UrlWithContent(Cow<'a, str>, String),
    /// label, uppercase
    InterLink(Cow<'a, str>, bool),
    /// label, uppercase, content (already converted)
    InterLinkWithContent(Cow<'a, str>, bool, String),
}

#[derive(Debug)]
pub enum ReferenceParseResult<'a> {
    Link(Link<'a>),
    Command(Command),
    ResolveInclude(Cow<'a, str>),
}


pub fn parse_references<'a>(cfg: &'a Config, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, content: String) -> ReferenceParseResult<'a> {
    // ShortcutUnknown and ReferenceUnknown make destination lowercase, but save original case in title
    let mut dst = match typ {
        LinkType::ShortcutUnknown | LinkType::ReferenceUnknown => title,
        _ => dst,
    };

    dst.trim_left_inplace();

    // possible include
    match typ {
        LinkType::ShortcutUnknown => if dst.starts_with_ignore_ascii_case("include ") {
            dst.truncate_left(8);
            return ReferenceParseResult::ResolveInclude(dst);
        } else if let Ok(command) = Command::from_str(&dst) {
            return ReferenceParseResult::Command(command);
        }
        _ => (),
    }

    // biber
    if cfg.bibliography.is_some() && dst.trim_left().starts_with('@') && typ == LinkType::ShortcutUnknown {
        // TODO: parse biber file and warn on unknown references
        // TODO: don't clone here
        if iter_multiple_biber(dst.clone()).nth(1).is_some() {
            return ReferenceParseResult::Link(Link::BiberMultiple(iter_multiple_biber(dst).collect()));
        } else {
            let (r, a) = parse_single_biber(dst);
            return ReferenceParseResult::Link(Link::BiberSingle(r, a));
        }
    }

    // sanity check
    if !dst.trim_left().starts_with('#') {
        match typ {
            // these cases should already be handled above for anything except '#'
            LinkType::ShortcutUnknown
            | LinkType::ReferenceUnknown
            | LinkType::CollapsedUnknown => unreachable!(),
            LinkType::Inline
            | LinkType::Autolink
            | LinkType::Reference
            | LinkType::Collapsed
            | LinkType::Shortcut => (),
        }
    }

    let prefix = dst.chars().next().unwrap();
    assert_ne!(prefix, '^', "Footnotes should be handled by pulldown-cmark already");
    let mut uppercase = None;
    if prefix == '#' {
        dst.truncate_left(1);
        // TODO: don't panic on invalid links (`#`)
        uppercase = Some(dst.chars().next().unwrap().is_uppercase());
        dst.make_ascii_lowercase_inplace();
    }

    match prefix {
        // cref / Cref / hyperlink
        '#' => match typ {
            LinkType::Shortcut | LinkType::ShortcutUnknown
            | LinkType::Collapsed | LinkType::CollapsedUnknown => {
                ReferenceParseResult::Link(Link::InterLink(dst, uppercase.unwrap()))
            },
            LinkType::Reference | LinkType::ReferenceUnknown
            | LinkType::Autolink
            | LinkType::Inline => {
                ReferenceParseResult::Link(Link::InterLinkWithContent(dst, uppercase.unwrap(), content))
            }
        }
        // url
        _ => match typ {
            LinkType::Autolink
            | LinkType::Shortcut | LinkType::ShortcutUnknown
            | LinkType::Collapsed | LinkType::CollapsedUnknown => {
                ReferenceParseResult::Link(Link::Url(dst))
            }
            LinkType::Reference | LinkType::ReferenceUnknown
            | LinkType::Inline => {
                ReferenceParseResult::Link(Link::UrlWithContent(dst, content))
            }
        }
    }
}

fn iter_multiple_biber(s: Cow<'_, str>) -> impl Iterator<Item = (Cow<'_, str>, Option<Cow<'_, str>>)> {
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
fn parse_single_biber(mut s: Cow<'_, str>) -> (Cow<'_, str>, Option<Cow<'_, str>>) {
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
