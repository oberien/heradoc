use std::borrow::Cow;
use std::mem;

use crate::frontend::range::WithRange;

pub trait StrExt {
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

// https://github.com/rust-lang/rust/issues/40062
pub trait VecExt<T> {
    type Output;
    fn remove_element<U>(&mut self, element: &U) -> Option<Self::Output>
    where
        T: PartialEq<U>,
        U: ?Sized;
}

impl<T> VecExt<T> for Vec<WithRange<T>> {
    type Output = WithRange<T>;
    fn remove_element<U>(&mut self, element: &U) -> Option<Self::Output>
        where
            T: PartialEq<U>,
            U: ?Sized
    {
        let pos = self.iter().position(|a| *a.as_ref().element() == *element)?;
        Some(self.remove(pos))
    }
}

pub trait CowExt: Sized {
    /// Returns number of `(leading_spaces, trailing_spaces)`.
    ///
    /// If the whole string is just whitespaces, `leading_spaces` will be its length
    /// and `trailing_space` will be `0`.
    fn trim_lengths(&self) -> (usize, usize);
    /// Trims any leading and trailing whitespace
    fn trim_inplace(&mut self);
    /// Trims any leading whitespace
    fn trim_start_inplace(&mut self);
    /// Trims any trailing whitespace
    fn trim_end_inplace(&mut self);
    /// Removes the first `num` bytes
    fn truncate_start(&mut self, num: usize);
    /// Removes the last `num` bytes
    fn truncate_end(&mut self, num: usize);
    /// Convert to ascii lowercase
    fn make_ascii_lowercase_inplace(&mut self);
    /// Returns `([0, at), [at, len))`
    fn split_at(self, at: usize) -> (Self, Self);
    /// Self contains `[0, at)`, returns `[at, len)`
    fn split_off(&mut self, at: usize) -> Self;
    /// Self contains `[at, len)`, returns `[0, at)`
    fn split_to(&mut self, at: usize) -> Self;
}

impl<'a> CowExt for Cow<'a, str> {
    fn trim_lengths(&self) -> (usize, usize) {
        let leading_spaces = self.len() - self.trim_start().len();
        if leading_spaces == self.len() {
            return (leading_spaces, 0);
        }
        let trailing_spaces = self.len() - self.trim_end().len();
        (leading_spaces, trailing_spaces)
    }

    fn trim_inplace(&mut self) {
        match self {
            Cow::Borrowed(s) => *s = s.trim(),
            Cow::Owned(s) => {
                let trimmed = s.trim();
                let start = trimmed.as_ptr() as usize - s.as_ptr() as usize;
                let end = start + trimmed.len();
                s.truncate(end);
                s.drain(..start);
            }
        }
    }

    fn trim_start_inplace(&mut self) {
        match self {
            Cow::Borrowed(s) => *s = s.trim_start(),
            Cow::Owned(s) => drop(s.drain(..s.len() - s.trim_start().len())),
        }
    }

    fn trim_end_inplace(&mut self) {
        match self {
            Cow::Borrowed(s) => *s = s.trim_end(),
            Cow::Owned(s) => s.truncate(s.trim_end().len()),
        }
    }

    fn truncate_start(&mut self, num: usize) {
        match self {
            Cow::Borrowed(s) => *s = &s[num..],
            Cow::Owned(s) => drop(s.drain(..num)),
        }
    }

    fn truncate_end(&mut self, num: usize) {
        let end = self.len() - num;
        match self {
            Cow::Borrowed(s) => *s = &s[..end],
            Cow::Owned(s) => s.truncate(end),
        }
    }

    fn make_ascii_lowercase_inplace(&mut self) {
        match self {
            Cow::Borrowed(s) => {
                if !s.bytes().all(|b| b.is_ascii_lowercase()) {
                    *self = Cow::Owned(s.to_ascii_lowercase());
                }
            },
            Cow::Owned(s) => s.as_mut_str().make_ascii_lowercase(),
        }
    }

    fn split_at(self, at: usize) -> (Self, Self) {
        match self {
            Cow::Borrowed(s) => (Cow::Borrowed(&s[..at]), Cow::Borrowed(&s[at..])),
            Cow::Owned(mut s) => {
                let s2 = s.split_off(at);
                (Cow::Owned(s), Cow::Owned(s2))
            },
        }
    }

    fn split_off(&mut self, at: usize) -> Self {
        match self {
            Cow::Borrowed(s) => {
                let start = &s[..at];
                let end = &s[at..];
                *s = start;
                Cow::Borrowed(end)
            },
            Cow::Owned(s) => Cow::Owned(s.split_off(at)),
        }
    }

    fn split_to(&mut self, at: usize) -> Self {
        match self {
            Cow::Borrowed(s) => {
                let start = &s[..at];
                let end = &s[at..];
                *s = end;
                Cow::Borrowed(start)
            },
            Cow::Owned(s) => {
                let mut other = s.split_off(at);
                mem::swap(&mut other, s);
                Cow::Owned(other)
            },
        }
    }
}
