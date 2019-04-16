use std::borrow::Cow;
use std::str::FromStr;

pub use pulldown_cmark::LinkType;

use super::event::{BiberReference, InterLink, Url};
use crate::config::Config;
use crate::diagnostics::Diagnostics;
use crate::ext::{CowExt, StrExt};
use crate::frontend::range::EscapedRange;
use crate::resolve::Command;

#[derive(Debug)]
pub enum ReferenceParseResult<'a> {
    BiberReferences(Vec<BiberReference<'a>>),
    Url(Url<'a>),
    InterLink(InterLink<'a>),
    UrlWithContent(Url<'a>),
    InterLinkWithContent(InterLink<'a>),
    Command(Command),
    ResolveInclude(Cow<'a, str>),
    Text(Cow<'a, str>),
}

pub fn parse_references<'a>(
    cfg: &'a Config, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, range: EscapedRange,
    diagnostics: &Diagnostics<'a>,
) -> ReferenceParseResult<'a> {
    // ShortcutUnknown and ReferenceUnknown make destination lowercase, but save original case in
    // title
    let mut dst = match typ {
        LinkType::ShortcutUnknown | LinkType::ReferenceUnknown => title.clone(),
        _ => dst,
    };
    let title = if title.is_empty() { None } else { Some(title) };

    dst.trim_start_inplace();

    // possible include
    if let LinkType::ShortcutUnknown = typ {
        if dst.starts_with_ignore_ascii_case("include ") {
            dst.truncate_start(8);
            return ReferenceParseResult::ResolveInclude(dst);
        } else if let Ok(command) = Command::from_str(&dst) {
            return ReferenceParseResult::Command(command);
        }
    }

    // biber
    if dst.trim_start().starts_with('@') && typ == LinkType::ShortcutUnknown {
        if cfg.bibliography.is_none() {
            diagnostics
                .error("found biber reference, but no bibliography file found")
                .with_error_section(range, "referenced here")
                .note("rendering as text")
                .emit();
            return ReferenceParseResult::Text(Cow::Owned(format!("[{}]", dst)));
        }
        // TODO: parse biber file and warn on unknown references
        // TODO: don't clone here
        if iter_multiple_biber(dst.clone()).nth(1).is_some() {
            return ReferenceParseResult::BiberReferences(
                iter_multiple_biber(dst)
                    .map(|(r, a)| BiberReference { reference: r, attributes: a })
                    .collect(),
            );
        } else {
            let (r, a) = parse_single_biber(dst);
            return ReferenceParseResult::BiberReferences(vec![BiberReference {
                reference: r,
                attributes: a,
            }]);
        }
    }

    // sanity check
    if !dst.trim_start().starts_with('#') {
        match typ {
            // these cases should already be handled above for anything except '#'
            LinkType::ShortcutUnknown | LinkType::ReferenceUnknown | LinkType::CollapsedUnknown => {
                unreachable!()
            },
            LinkType::Inline
            | LinkType::Autolink
            | LinkType::Email
            | LinkType::Reference
            | LinkType::Collapsed
            | LinkType::Shortcut => (),
        }
    }

    let prefix = dst.chars().next().unwrap();
    assert_ne!(prefix, '^', "Footnotes should be handled by pulldown-cmark already");
    let mut uppercase = None;
    if prefix == '#' {
        dst.truncate_start(1);
        // TODO: don't panic on invalid links (`#`)
        uppercase = Some(dst.chars().next().unwrap().is_uppercase());
        dst.make_ascii_lowercase_inplace();
    }

    match prefix {
        // cref / Cref / hyperlink
        '#' => match typ {
            LinkType::Shortcut
            | LinkType::ShortcutUnknown
            | LinkType::Collapsed
            | LinkType::CollapsedUnknown => ReferenceParseResult::InterLink(InterLink {
                label: dst,
                uppercase: uppercase.unwrap(),
            }),
            LinkType::Reference
            | LinkType::ReferenceUnknown
            | LinkType::Autolink
            | LinkType::Email
            | LinkType::Inline => ReferenceParseResult::InterLinkWithContent(InterLink {
                label: dst,
                uppercase: uppercase.unwrap(),
            }),
        },
        // url
        _ => match typ {
            LinkType::Autolink
            | LinkType::Email
            | LinkType::Shortcut
            | LinkType::ShortcutUnknown
            | LinkType::Collapsed
            | LinkType::CollapsedUnknown => {
                ReferenceParseResult::Url(Url { destination: dst, title })
            },
            LinkType::Reference | LinkType::ReferenceUnknown | LinkType::Inline => {
                ReferenceParseResult::UrlWithContent(Url { destination: dst, title })
            },
        },
    }
}

fn iter_multiple_biber(
    s: Cow<'_, str>,
) -> impl Iterator<Item = (Cow<'_, str>, Option<Cow<'_, str>>)> {
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
            self.s.trim_start_inplace();
            assert!(self.s.starts_with("@"));

            let next_at = self.s[1..].find('@').map_or(self.s.len(), |i| i + 1);
            let single_end = self.s[..next_at].rfind(',').unwrap_or(self.s.len());
            let single = self.s.split_to(single_end);
            self.s.truncate_start(next_at - single_end);
            Some(parse_single_biber(single))
        }
    }

    Iter { s }
}

/// Returns (reference, Option<options>)
///
/// Expects input in the form of `"@foo Chapters 10-15,25"`.
/// Returns `("foo", Some("Chapters 10-15, 25"))`.
fn parse_single_biber(mut s: Cow<'_, str>) -> (Cow<'_, str>, Option<Cow<'_, str>>) {
    s.trim_inplace();
    assert!(s.starts_with('@'), "Expected a biber reference starting with `@`, found {:?}", s);

    // get rid of `@`
    s.truncate_start(1);

    let spacepos = s.find(' ');
    let slen = s.len();
    let (reference, mut opts) = s.split_at(spacepos.unwrap_or(slen));
    // get rid of leading / trailing spaces
    opts.trim_inplace();
    let opts = if opts.len() == 0 { None } else { Some(opts) };
    (reference, opts)
}
