use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::{Tag, Event};

use super::{State, States, Generator, read_until};

#[derive(Debug)]
pub struct Link<'a> {
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    events: Vec<Event<'a>>,
}

impl<'a> State<'a> for Link<'a> {
    fn new(tag: Tag<'a>, stack: &[States], out: &mut impl Write) -> Result<Self> {
        let (dst, title) = match tag {
            Tag::Link(dst, title) => (dst, title),
            _ => unreachable!(),
        };
        Ok(Link {
            dst,
            title,
            events: Vec::new(),
        })
    }

    fn intercept_event(&mut self, e: Event<'a>, out: &mut impl Write) -> Result<Option<Event<'a>>> {
        self.events.push(e);
        Ok(None)
    }

    fn finish(self, gen: &mut Generator<'a>, peek: Option<&Event<'a>>, out: &mut impl Write) -> Result<()> {
        // TODO: handle all links properly
        // Markdown Types of links: https://github.com/google/pulldown-cmark/issues/141

        // * [@foo]: biber reference (transformed in main.rs:refsolve)
        // * [#foo]: \cref (reference to section)
        //     * dst="#foo", title="#foo", text="#foo"
        // * [#Foo]: \Cref (capital reference to section)
        //     * dst="#foo", title="#Foo", text="#Foo"
        // * [img/fig/tbl/fnote:bar]: \cref (reference to images / figures / footnotes)
        //     * dst="img/fig/fnote:bar", title="img/fig/fnote:bar", text="img/fig/tbl/fnote:bar"
        // * [Img/Fig/Tbl/Fnote:bar]: \cref (capital reference to images / figures / footnotes)
        //     * dst="img/fig/fnote:bar", title="Img/Fig/Fnote:bar", text="Img/Fig/Tbl/Fnote:bar"
        // * [bar] (with bar defined): Handle link as above
        //     * dst="link", title="title", text="bar"
        // * [text](link "title"): handle link as in previous examples, but use hyperref
        //     * dst="link", title="title", text="text"
        // * [text][ref]: same as [text](link "title")
        //     * dst="link", title="title", text="text"
        // TODO: use title
        let text = read_until(gen, self.events, peek)?;

        let uppercase = self.dst.chars().nth(0).unwrap().is_ascii_uppercase();
        let dst = self.dst.to_ascii_lowercase();
        let dst_eq_text = dst == text.to_ascii_lowercase();

        if dst.starts_with('#') || dst.starts_with("img:") || dst.starts_with("fig:") {
            let dst = if dst.starts_with('#') { &dst[1..] } else { dst.as_str() };
            let text = if text.starts_with('#') { &text[1..] } else { text.as_str() };

            if text.is_empty() || dst_eq_text {
                if uppercase {
                    write!(out, "\\Cref{{{}}}", dst)?;
                } else {
                    write!(out, "\\cref{{{}}}", dst)?;
                }
            } else {
                write!(out, "\\hyperref[{}]{{{}}}", dst, text)?;
            }
        } else {
            if text.is_empty() || dst_eq_text {
                write!(out, "\\url{{{}}}", dst)?;
            } else {
                write!(out, "\\href{{{}}}{{{}}}", dst, text)?;
            }
        }
        Ok(())
    }
}
