use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscapedRange {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for EscapedRange {
    fn from(Range { start, end }: Range<usize>) -> Self {
        EscapedRange { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithRange<T>(pub T, pub EscapedRange);

impl<T> WithRange<T> {
    pub fn element(self) -> T {
        self.0
    }

    pub fn range(&self) -> EscapedRange {
        self.1
    }

    pub fn as_ref(&self) -> WithRange<&T> {
        WithRange(&self.0, self.1)
    }

    pub fn map<R>(self, f: impl FnOnce(T) -> R) -> WithRange<R> {
        let WithRange(element, range) = self;
        WithRange(f(element), range)
    }
}
