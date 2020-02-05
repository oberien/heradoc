use std::borrow::Cow;
use std::iter::Peekable;

use str_concat;

use super::convert_cow::{ConvertCow, Event};
use crate::frontend::range::WithRange;

pub struct Concat<'a>(Peekable<ConvertCow<'a>>);

impl<'a> Concat<'a> {
    pub fn new(i: ConvertCow<'a>) -> Self {
        Concat(i.peekable())
    }
}

impl<'a> Iterator for Concat<'a> {
    type Item = WithRange<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let WithRange(mut t, mut range) = match self.0.next()? {
            WithRange(Event::Text(t), range) => WithRange(t, range),
            evt => return Some(evt),
        };

        while let Some(Event::Text(_)) = self.0.peek().map(|t| t.as_ref().element()) {
            let WithRange(evt, r) = self.0.next().unwrap();
            // We can't assume that two text events follow each other directly here.
            // For example a quote results in two text events following each other having different
            // offsets in the original markdown content string:
            // ```md
            // > foo
            // > bar
            // ```
            assert!(range.end <= r.start);
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
        Some(WithRange(Event::Text(t), range))
    }
}
