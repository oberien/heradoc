use std::io::Write;

use crate::backend::{Backend, CodeGenUnits};

pub struct Stack<'a: 'b, 'b, D: Backend<'a> + 'b, W: Write + 'b> {
    default_out: &'b mut W,
    stack: &'b mut [CodeGenUnits<'a, D>],
}

impl<'a: 'b, 'b, D: Backend<'a> + 'b, W: Write> Stack<'a, 'b, D, W> {
    pub(super) fn new(default_out: &'b mut W, stack: &'b mut [CodeGenUnits<'a, D>]) -> Self {
        Stack {
            default_out,
            stack,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &CodeGenUnits<'a, D>> {
        self.stack.iter()
    }

    // TODO
    #[allow(unused)]
    pub fn get_out(&mut self) -> &mut dyn Write {
        self.stack.iter_mut().rev()
            .filter_map(|state| state.output_redirect()).next()
            .unwrap_or(self.default_out)
    }
}
