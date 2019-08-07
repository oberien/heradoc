use std::io::Write;

use super::StackElement;
use crate::backend::Backend;
use crate::diagnostics::Diagnostics;

pub struct Stack<'a: 'b, 'b, B: Backend<'a>, W: Write> {
    default_out: &'b mut W,
    stack: &'b mut [StackElement<'a, B>],
}

impl<'a: 'b, 'b, B: Backend<'a> + 'b, W: Write> Stack<'a, 'b, B, W> {
    pub(super) fn new(default_out: &'b mut W, stack: &'b mut [StackElement<'a, B>]) -> Self {
        Stack { default_out, stack }
    }

    pub fn iter(&self) -> impl Iterator<Item = &StackElement<'a, B>> {
        self.stack.iter().rev()
    }

    pub fn get_out(&mut self) -> &mut dyn Write {
        self.stack
            .iter_mut()
            .rev()
            .filter_map(|state| state.output_redirect())
            .next()
            .unwrap_or(self.default_out)
    }

    #[allow(dead_code)]
    pub fn diagnostics(&mut self) -> &Diagnostics<'a> {
        self.stack
            .iter_mut()
            .rev()
            .filter_map(|state| match state {
                StackElement::Context(_, diagnostics) => Some(diagnostics),
                _ => None,
            })
            .next()
            .unwrap()
    }
}
