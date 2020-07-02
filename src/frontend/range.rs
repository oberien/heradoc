use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for SourceRange {
    fn from(Range { start, end }: Range<usize>) -> Self {
        SourceRange { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithRange<T>(pub T, pub SourceRange);

impl<T> WithRange<T> {
    pub fn element(self) -> T {
        self.0
    }

    pub fn element_ref(&self) -> &T {
        &self.0
    }

    pub fn range(&self) -> SourceRange {
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
