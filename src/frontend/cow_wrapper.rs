use std::borrow::Cow;
use std::ops::Deref;

use crate::frontend::range::{EscapedRange, UnescapedRange};
use crate::ext::CowExt;

/// Wraps possibly concatenated Text events with their respective `EscapedRange`s
/// for range transformations.
#[derive(Debug, Clone)]
pub struct CowWrapper<'a> {
    cow: Cow<'a, str>,
    ranges: Vec<EscapedRange>,
}

impl<'a> Deref for CowWrapper<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.cow
    }
}

impl<'a> CowWrapper<'a> {
    /// Returns the source range of the whole wrapped content including any escape characters.
    pub fn escaped_range(&self) -> EscapedRange {
        EscapedRange { start: self.ranges.first().unwrap().start, end: self.ranges.last().unwrap().end }
    }

    /// Returns the unescaped range representing this Cow without any escape characters.
    pub fn unescaped_range(&self) -> UnescapedRange {
        let start = self.ranges.first().unwrap().start;
        UnescapedRange {
            start,
            end: start + self.cow.len(),
        }
    }

    /// Transforms an escaped range to an UnescapedRange containing no escape characters.
    ///
    /// This can be used as immediate transformation while performing text processing.
    /// The function `transform_unescaped_range` can be used to transform modified ranges back.
    /// See the documentation of [`UnescapedRange`] for more information.
    pub fn transform_escaped_range(&self, escaped: EscapedRange) -> UnescapedRange {
        let mut unescaped_start = None;
        let mut prev = None;
        let mut unescaped_equiv = 0;
        for EscapedRange { start, end } in self.ranges.iter().cloned() {
            if start == escaped.start && prev.is_some() {
                unescaped_start = Some(unescaped_equiv);
            } else if escaped.start >= start && escaped.start <= end {
                unescaped_start = Some(unescaped_equiv + (escaped.start - start));
            }
            prev = Some(EscapedRange { start, end });
            unescaped_equiv += end - start;
        }

        let len: usize = self.ranges.iter().cloned()
            .skip_while(|r| r.end < escaped.start)
            .take_while(|r| r.start <= escaped.end)
            .map(|r| r.end.min(escaped.end) - r.start.max(escaped.start))
            .sum();
        UnescapedRange {
            start: unescaped_start.unwrap(),
            end: unescaped_start.unwrap() + len,
        }
    }

    /// Transforms an `UnescapedRange` back to the corresponding `EscapedRange` including all
    /// escape characters present in the source-code.
    pub fn transform_unescaped_range(&self, unescaped: UnescapedRange) -> EscapedRange {
        // pulldown-cmark: (r"\*Foo\*Bar", 0..10) → (r"", 0..0), (r"*Foo", 1..5), (r"*Bar", 6..10)
        //      (see <https://github.com/raphlinus/pulldown-cmark/issues/273>)
        // escaped: 0..10
        // unescaped: 0..8
        // examples:
        // * full string: 0..8 -> 0..10
        // * start at middle escaped char: 4..8 -> 5..10
        // * from the middle: 2..8 -> 3..10


        let mut escaped_start = None;
        let mut escaped_end = None;
        let mut unescaped_equiv = 0;
        let mut prev_escaped: Option<EscapedRange> = None;
        for EscapedRange { start, end } in self.ranges.iter().cloned() {
            let equiv_start = unescaped_equiv;
            let equiv_end = equiv_start + (end - start);

            if unescaped.start == equiv_start && prev_escaped.is_some() {
                escaped_start = Some(prev_escaped.unwrap().end);
            } else if unescaped.start >= equiv_start && unescaped.start <= equiv_end {
                escaped_start = Some(start + (unescaped.start - equiv_start));
            }

            if unescaped.end >= equiv_start && unescaped.end <= equiv_end {
                escaped_end = Some(start + (unescaped.end - equiv_start));
            }

            unescaped_equiv = equiv_end;
            prev_escaped = Some(EscapedRange { start, end });
        }
        EscapedRange { start: escaped_start.unwrap(), end: escaped_end.unwrap() }
    }

    /// Returns the modified list of text-ranges that describe the source-code location of `subrange`.
    fn subranges(&self, subrange: EscapedRange) -> Vec<EscapedRange> {
        self.ranges.iter().cloned()
            .skip_while(|r| r.end < subrange.start)
            .take_while(|r| r.start <= subrange.end)
            .map(|r| EscapedRange {
                start: r.start.max(subrange.start),
                end: r.end.min(subrange.end),
            }).collect()
    }
}

impl<'a> CowExt for CowWrapper<'a> {
    fn trim_lengths(&self) -> (usize, usize) {
        self.cow.trim_lengths()
    }

    fn trim_inplace(&mut self) {
        let mut unescaped = self.unescaped_range();
        let (left, right) = self.cow.trim_lengths();
        self.cow.trim_inplace();
        unescaped.start += left;
        unescaped.end -= right;
        assert_eq!(unescaped.len(), self.cow.len());
        let escaped = self.transform_unescaped_range(unescaped);
        self.ranges = self.subranges(escaped);
    }

    fn trim_start_inplace(&mut self) {
        let mut unescaped = self.unescaped_range();
        let (left, _) = self.cow.trim_lengths();
        self.cow.trim_start_inplace();
        unescaped.start += left;
        let escaped = self.transform_unescaped_range(unescaped);
        self.ranges = self.subranges(escaped);
    }

    fn trim_end_inplace(&mut self) {
        let mut unescaped = self.unescaped_range();
        let (_, right) = self.cow.trim_lengths();
        self.cow.trim_end_inplace();
        unescaped.end += right;
        let escaped = self.transform_unescaped_range(unescaped);
        self.ranges = self.subranges(escaped);
    }

    fn truncate_start(&mut self, num: usize) {
        let mut unescaped = self.unescaped_range();
        self.cow.truncate_start(num);
        unescaped.start += num;
        let escaped = self.transform_unescaped_range(unescaped);
        self.ranges = self.subranges(escaped);
    }

    fn truncate_end(&mut self, num: usize) {
        let mut unescaped = self.unescaped_range();
        self.cow.truncate_end(num);
        unescaped.end += num;
        let escaped = self.transform_unescaped_range(unescaped);
        self.ranges = self.subranges(escaped);
    }

    fn make_ascii_lowercase_inplace(&mut self) {
        self.cow.make_ascii_lowercase_inplace();
    }

    fn split_at(self, at: usize) -> (Self, Self) {
        let unescaped = self.unescaped_range();
        let unescaped_left = UnescapedRange { start: unescaped.start, end: unescaped.start + at };
        let unescaped_right = UnescapedRange { start: unescaped.start + at, end: unescaped.end };
        let escaped_left = self.transform_unescaped_range(unescaped_left);
        let escaped_right = self.transform_unescaped_range(unescaped_right);
        let subranges_left = self.subranges(escaped_left);
        let subranges_right = self.subranges(escaped_right);
        let (left, right) = self.cow.split_at(at);
        let left = CowWrapper { cow: left, ranges: subranges_left };
        let right = CowWrapper { cow: right, ranges: subranges_right };
        (left, right)
    }

    fn split_off(&mut self, at: usize) -> Self {
        let unescaped = self.unescaped_range();
        let unescaped_left = UnescapedRange { start: unescaped.start, end: unescaped.start + at };
        let unescaped_right = UnescapedRange { start: unescaped.start + at, end: unescaped.end };
        let escaped_left = self.transform_unescaped_range(unescaped_left);
        let escaped_right = self.transform_unescaped_range(unescaped_right);
        let subranges_left = self.subranges(escaped_left);
        let subranges_right = self.subranges(escaped_right);
        let right = self.cow.split_off(at);
        self.ranges = subranges_left;
        CowWrapper { cow: right, ranges: subranges_right }
    }

    fn split_to(&mut self, at: usize) -> Self {
        let unescaped = self.unescaped_range();
        let unescaped_left = UnescapedRange { start: unescaped.start, end: unescaped.start + at };
        let unescaped_right = UnescapedRange { start: unescaped.start + at, end: unescaped.end };
        let escaped_left = self.transform_unescaped_range(unescaped_left);
        let escaped_right = self.transform_unescaped_range(unescaped_right);
        let subranges_left = self.subranges(escaped_left);
        let subranges_right = self.subranges(escaped_right);
        let left = self.cow.split_to(at);
        self.ranges = subranges_right;
        CowWrapper { cow: left, ranges: subranges_left }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Event::*, Tag::*, CowStr::*, Parser};

    #[test]
    fn test_pulldown_cmark_unescaping_events() {
        let mut parser = Parser::new(r"Foo\\Bar").into_offset_iter();
        assert_eq!(parser.next(), Some((Start(Paragraph), 0..8)));
        assert_eq!(parser.next(), Some((Text(Borrowed("Foo")), 0..3)));
        assert_eq!(parser.next(), Some((Text(Borrowed(r"\Bar")), 4..8)));
        assert_eq!(parser.next(), Some((End(Paragraph), 0..8)));
        assert_eq!(parser.next(), None);

        let mut parser = Parser::new(r"\*Foo\*Bar").into_offset_iter();
        assert_eq!(parser.next(), Some((Start(Paragraph), 0..10)));
        assert_eq!(parser.next(), Some((Text(Borrowed("")), 0..0)));
        assert_eq!(parser.next(), Some((Text(Borrowed("*Foo")), 1..5)));
        assert_eq!(parser.next(), Some((Text(Borrowed(r"*Bar")), 6..10)));
        assert_eq!(parser.next(), Some((End(Paragraph), 0..10)));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_transform_escaped_range() {
        let cow = CowWrapper {
            cow: Cow::Borrowed("*Foo*Bar"),
            ranges: vec![
                EscapedRange { start: 0, end: 0 },
                EscapedRange { start: 1, end: 5 },
                EscapedRange { start: 6, end: 10 },
            ],
        };
        assert_eq!(
            cow.transform_escaped_range(EscapedRange { start: 0, end: 10 }),
            UnescapedRange { start: 0, end: 8 },
            "full string",
        );
        assert_eq!(
            cow.transform_escaped_range(EscapedRange { start: 0, end: 2 }),
            UnescapedRange { start: 0, end: 1 },
            "first escaped character",
        );
        assert_eq!(
            cow.transform_escaped_range(EscapedRange { start: 1, end: 10 }),
            UnescapedRange { start: 0, end: 8 },
            "start in beginning escaped character",
        );
        assert_eq!(
            cow.transform_escaped_range(EscapedRange { start: 3, end: 6 }),
            UnescapedRange { start: 2, end: 4 },
            "start in middle end in second escaped character",
        );
        assert_eq!(
            cow.transform_escaped_range(EscapedRange { start: 5, end: 10 }),
            UnescapedRange { start: 4, end: 8 },
            "start before middle escaped character",
        );
    }

    #[test]
    fn test_transform_unescaped_range() {
        let cow = CowWrapper {
            cow: Cow::Borrowed("*Foo*Bar"),
            ranges: vec![
                EscapedRange { start: 0, end: 0 },
                EscapedRange { start: 1, end: 5 },
                EscapedRange { start: 6, end: 10 },
            ],
        };
        assert_eq!(
            cow.transform_unescaped_range(UnescapedRange { start: 0, end: 8 }),
            EscapedRange { start: 0, end: 10 },
            "full string",
        );
        assert_eq!(
            cow.transform_unescaped_range(UnescapedRange { start: 4, end: 8 }),
            EscapedRange { start: 5, end: 10 },
            "start at middle escaped char",
        );
        assert_eq!(
            cow.transform_unescaped_range(UnescapedRange { start: 2, end: 8 }),
            EscapedRange { start: 3, end: 10 },
            "from the middle"
        );

        let cow = CowWrapper {
            cow: Cow::Borrowed("FooBar"),
            ranges: vec![
                EscapedRange { start: 0, end: 3 },
                EscapedRange { start: 7, end: 10 },
            ],
        };
        assert_eq!(
            cow.transform_unescaped_range(UnescapedRange { start: 2, end: 5 }),
            EscapedRange { start: 2, end: 9 },
            "large gap",
        );
    }

    #[test]
    fn test_subranges() {
        let cow = CowWrapper {
            cow: Cow::Borrowed("*Foo*Bar"),
            ranges: vec![
                EscapedRange { start: 0, end: 0 },
                EscapedRange { start: 1, end: 5 },
                EscapedRange { start: 6, end: 10 },
            ],
        };
        assert_eq!(
            cow.subranges(EscapedRange { start: 0, end: 10 }),
            vec![
                EscapedRange { start: 0, end: 0 },
                EscapedRange { start: 1, end: 5 },
                EscapedRange { start: 6, end: 10 },
            ],
            "full string",
        );
        assert_eq!(
            cow.subranges(EscapedRange { start: 0, end: 2 }),
            vec![
                EscapedRange { start: 0, end: 0 },
                EscapedRange { start: 1, end: 2 },
            ],
            "first escaped character",
        );
        assert_eq!(
            cow.subranges(EscapedRange { start: 2, end: 4 }),
            vec![
                EscapedRange { start: 2, end: 4 },
            ],
            "middle in same text-range",
        );
        assert_eq!(
            cow.subranges(EscapedRange { start: 4, end: 8 }),
            vec![
                EscapedRange { start: 4, end: 5 },
                EscapedRange { start: 6, end: 8 },
            ],
            "multiple text-ranges",
        );
    }
}