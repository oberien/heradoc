use std::iter::Peekable;
use std::borrow::Cow;

use str_concat;
use super::convert_cow::Event;

pub struct Concat<'a, I: Iterator<Item = Event<'a>>>(Peekable<I>);

impl<'a, I: Iterator<Item = Event<'a>>> Concat<'a, I> {
    pub fn new(i: I) -> Self {
        Concat(i.peekable())
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for Concat<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut t = match self.0.next() {
            None => return None,
            Some(Event::Text(t)) => t,
            Some(evt) => return Some(evt),
        };

        while let Some(Event::Text(_)) = self.0.peek() {
            let next = match self.0.next() {
                Some(Event::Text(t)) => t,
                _ => unreachable!()
            };

            match t {
                Cow::Borrowed(b) => match next {
                    Cow::Borrowed(next) => match str_concat::concat(b, next) {
                        Ok(res) => t = Cow::Borrowed(res),
                        Err(_) => t = Cow::Owned(b.to_string() + next),
                    }
                    Cow::Owned(mut next) => {
                        next.insert_str(0, b);
                        t = Cow::Owned(next);
                    }
                }
                Cow::Owned(ref mut o) => o.push_str(&next),
            }
        }
        Some(Event::Text(t))
    }
}
