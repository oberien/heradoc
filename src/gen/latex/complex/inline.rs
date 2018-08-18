use std::io::{Result, Write};

use pulldown_cmark::{Tag, Event};

use crate::gen::{State, States, Generator, Stack, Document};

#[derive(Debug)]
pub struct InlineEmphasis;

impl<'a> State<'a> for InlineEmphasis {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(stack.get_out(), "\\emph{{")?;
        Ok(InlineEmphasis)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        write!(stack.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineStrong;

impl<'a> State<'a> for InlineStrong {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(stack.get_out(), "\\textbf{{")?;
        Ok(InlineStrong)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        write!(stack.get_out(), "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InlineCode;

impl<'a> State<'a> for InlineCode {
    fn new<'b>(tag: Tag<'a>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<Self> {
        write!(stack.get_out(), "\\texttt{{")?;
        Ok(InlineCode)
    }

    fn finish<'b>(self, peek: Option<&Event<'a>>, mut stack: Stack<'a, 'b, impl Document<'a>, impl Write>) -> Result<()> {
        write!(stack.get_out(), "}}")?;
        Ok(())
    }
}
