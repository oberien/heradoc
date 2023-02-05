use std::borrow::Cow;
use std::iter::Peekable;
use diagnostic::Spanned;

use str_concat;

use super::convert_cow::{ConvertCow, Event};

pub struct Concat<'a>(Peekable<ConvertCow<'a>>);

impl<'a> Concat<'a> {
    pub fn new(i: ConvertCow<'a>) -> Self {
        Concat(i.peekable())
    }
}

impl<'a> Iterator for Concat<'a> {
    type Item = Spanned<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let Spanned { value: mut t, mut span } = match self.0.next()? {
            Spanned { value: Event::Text(t), span } => Spanned::new(t, span),
            evt => return Some(evt),
        };

        while let Some(Event::Text(_)) = self.0.peek().map(|t| t.as_ref().value) {
            let Spanned { value: evt, span: s } = self.0.next().unwrap();
            // We can't assume that two text events follow each other directly here.
            // For example a quote results in two text events following each other having different
            // offsets in the original markdown content string:
            // ```md
            // > foo
            // > bar
            // ```
            assert!(span.end <= s.start);
            span.end = s.end;

            let next = match evt {
                Event::Text(t) => t,
                _ => unreachable!(),
            };

            match t {
                Cow::Borrowed(b) => match next {
                    // SAFETY: it's from the same allocation, namely the same file-string
                    Cow::Borrowed(next) => match unsafe { str_concat::concat(b, next) } {
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
        Some(Spanned::new(Event::Text(t), span))
    }
}
