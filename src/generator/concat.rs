use std::borrow::Cow;

use str_concat;

use crate::frontend::Event;
use super::Positioned;

/// Concatenates consecutive text events.
///
/// Reimplements parts of `Peekable` but under the requirement of `FusedIterator`, leading to some
/// implementation simplifications.  Also forwards the implementation of `Positioned`.
pub struct Concat<'a, I: Iterator<Item = Event<'a>>> {
    iter: I,
    peeked: Option<Event<'a>>,
}

impl<'a, I: Iterator<Item = Event<'a>>> Concat<'a, I> {
    pub fn new(iter: I) -> Self {
        Concat {
            iter,
            peeked: None,
        }
    }

    /// Like next but only yields text.
    ///
    /// Other events get stored in `peeked`. Only called when `peeked` is None.
    fn next_text(&mut self) -> Option<Cow<'a, str>> {
        use std::mem::replace;

        match self.iter.next() {
            Some(Event::Text(t)) => Some(t),
            Some(other) => {
                let none = replace(&mut self.peeked, Some(other));

                // This avoids trying to destroy the value and helps the llvm backend.
                match none { None => None, _ => unreachable!(), }
            },
            None => None,
        }
    }

    #[inline]
    fn peeked_or_next(&mut self) -> Option<Event<'a>> {
        self.peeked.take().or_else(|| self.iter.next())
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for Concat<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut first = match self.peeked_or_next() {
            None => return None,
            Some(Event::Text(t)) => t,
            Some(evt) => return Some(evt),
        };

        while let Some(next) = self.next_text() {
            match first {
                Cow::Borrowed(b) => match next {
                    Cow::Borrowed(next) => match str_concat::concat(b, next) {
                        Ok(res) => first = Cow::Borrowed(res),
                        Err(_) => first = Cow::Owned(b.to_string() + next),
                    }
                    Cow::Owned(mut next) => {
                        next.insert_str(0, b);
                        first = Cow::Owned(next);
                    }
                }
                Cow::Owned(ref mut o) => o.push_str(&next),
            }
        }

        Some(Event::Text(first))
    }
}

impl<'a, I> Positioned for Concat<'a, I> 
    where I: Iterator<Item = Event<'a>> + Positioned 
{
    fn current_position(&self) -> usize {
        self.iter.current_position()
    }
}
