use std::borrow::Cow;

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

impl<A, B> VecExt<A> for Vec<(A, B)> {
    type Output = (A, B);
    fn remove_element<U>(&mut self, element: &U) -> Option<(A, B)>
    where
        A: PartialEq<U>,
        U: ?Sized,
    {
        let pos = self.iter().position(|(a, _)| *a == *element)?;
        Some(self.remove(pos))
    }
}

pub trait CowExt<'a> {
    fn trim_inplace(&mut self);
    fn trim_start_inplace(&mut self);
    fn trim_end_inplace(&mut self);
    fn truncate_start(&mut self, num: usize);
    fn truncate_end(&mut self, num: usize);
    fn truncate(&mut self, len: usize);
    fn make_ascii_lowercase_inplace(&mut self);
    fn split_at(self, pos: usize) -> (Cow<'a, str>, Cow<'a, str>);
    fn split_off(&mut self, pos: usize) -> Cow<'a, str>;
    fn map_inplace(
        &mut self, borrowed: impl FnOnce(&'a str) -> &'a str, owned: impl FnOnce(&mut String),
    );
    fn map_inplace_return<R>(
        &mut self, borrowed: impl FnOnce(&'a str) -> (&'a str, R),
        owned: impl FnOnce(&mut String) -> R,
    ) -> R;
    fn map<R: 'a>(self, borrowed: impl FnOnce(&'a str) -> R, owned: impl FnOnce(String) -> R) -> R;
}

impl<'a> CowExt<'a> for Cow<'a, str> {
    fn trim_inplace(&mut self) {
        self.map_inplace(
            |s| s.trim(),
            |s| {
                let trimmed = s.trim();
                let start = trimmed.as_ptr() as usize - s.as_ptr() as usize;
                let end = start + trimmed.len();
                s.truncate(end);
                s.drain(..start);
            },
        );
    }

    fn trim_start_inplace(&mut self) {
        self.map_inplace(|s| s.trim_start(), |s| drop(s.drain(..s.len() - s.trim_start().len())));
    }

    fn trim_end_inplace(&mut self) {
        self.map_inplace(|s| s.trim_end(), |s| s.truncate(s.trim_end().len()));
    }

    fn truncate_start(&mut self, num: usize) {
        self.map_inplace(|s| &s[num..], |s| drop(s.drain(..num)));
    }

    fn truncate_end(&mut self, num: usize) {
        self.truncate(self.len() - num);
    }

    fn truncate(&mut self, len: usize) {
        self.map_inplace(|s| &s[..len], |s| s.truncate(len))
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

    fn split_at(self, pos: usize) -> (Cow<'a, str>, Cow<'a, str>) {
        match self {
            Cow::Borrowed(s) => (Cow::Borrowed(&s[..pos]), Cow::Borrowed(&s[pos..])),
            Cow::Owned(mut s) => {
                let s2 = s.split_off(pos);
                (Cow::Owned(s), Cow::Owned(s2))
            },
        }
    }

    fn split_off(&mut self, pos: usize) -> Cow<'a, str> {
        match self {
            Cow::Borrowed(s) => {
                let start = &s[..pos];
                let end = &s[pos..];
                *s = start;
                Cow::Borrowed(end)
            },
            Cow::Owned(s) => Cow::Owned(s.split_off(pos)),
        }
    }

    fn map_inplace(
        &mut self, borrowed: impl FnOnce(&'a str) -> &'a str, owned: impl FnOnce(&mut String),
    ) {
        match self {
            Cow::Borrowed(s) => *self = borrowed(s).into(),
            Cow::Owned(ref mut s) => owned(s),
        }
    }

    fn map_inplace_return<R>(
        &mut self, borrowed: impl FnOnce(&'a str) -> (&'a str, R),
        owned: impl FnOnce(&mut String) -> R,
    ) -> R {
        match self {
            Cow::Borrowed(s) => {
                let (val, ret) = borrowed(s);
                *self = val.into();
                ret
            },
            Cow::Owned(ref mut s) => owned(s),
        }
    }

    fn map<R>(self, borrowed: impl FnOnce(&'a str) -> R, owned: impl FnOnce(String) -> R) -> R {
        match self {
            Cow::Borrowed(s) => borrowed(s),
            Cow::Owned(s) => owned(s),
        }
    }
}
