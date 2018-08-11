use std::iter::Peekable;

pub trait Peek: Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;
}

impl<I: Iterator> Peek for Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.peek()
    }
}

impl<I: Iterator> Peek for &mut Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        Peekable::<I>::peek(self)
    }
}
