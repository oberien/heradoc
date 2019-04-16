use std::ops::Range;

macro_rules! make_range {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub start: usize,
            pub end: usize,
        }

        impl $name {
            pub fn len(&self) -> usize {
                self.end - self.start
            }
        }
    };
}

make_range! {
    /// Source-code range including all escape sequences.
    ///
    /// This should be used in diagnostics to display the source code corresponding to an event.
    EscapedRange
}

make_range! {
    /// Range of an unescaped source-code portion used during text processing.
    ///
    /// Given an EscapedRange it's non-trivial to map text processing operations to the range, because
    /// `n` characters in the unescaped string might be up to `2n` characters in the escaped source-code.
    /// Thus during text processing an `UnescapedRange` can be used, whose length is equal to the
    /// unescaped source code, but allows mapping back to the original `EscapedRange` via the transform*
    /// functions on `CowWrapper`.
    UnescapedRange
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
