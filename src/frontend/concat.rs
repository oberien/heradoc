use std::borrow::Cow;
use std::iter::Peekable;
use std::ops::Range;

use super::convert_cow::{ConvertCow, Event};
use str_concat;

pub struct Concat<'a>(Peekable<ConvertCow<'a>);

impl<'a> Concat<'a> {
    pub fn new(i: I) -> Self {
        Concat(i.peekable())
    }
}

impl<'a, I: Iterator<Item = (Event<'a>, Range<usize>)>> Iterator for Concat<'a, I> {
    type Item = (Event<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let (mut t, mut range) = match self.0.next() {
            None => return None,
            Some((Event::Text(t), range)) => (t, range),
            Some((evt, range)) => return Some((evt, range)),
        };

        while let Some(Event::Text(_)) = self.0.peek().map(|t| &t.0) {
            let (evt, r) = self.0.next().unwrap();
            // TODO: why are both variants needed?
            assert!(range.end == r.start || range.end + 1 == r.start);
            range.end = r.end;

            let next = match evt {
                Event::Text(t) => t,
                _ => unreachable!(),
            };

            match t {
                Cow::Borrowed(b) => match next {
                    Cow::Borrowed(next) => match str_concat::concat(b, next) {
                        Ok(res) => t = Cow::Borrowed(res),
                        Err(_) => t = Cow::Owned(b.to_string() + next),
                    },
                    Cow::Owned(mut next) => {
                        next.insert_str(0, b);
                        t = Cow::Owned(next);
                    },
                },
                Cow::Owned(ref mut o) => o.push_str(&next),
            }
        }
        Some((Event::Text(t), range))
    }
}
