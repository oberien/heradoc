use std::io::{Result, Write};
use std::borrow::Cow;

use pulldown_cmark::{Tag, Event, LinkType};

use crate::gen::{State, States, Generator, Document};

#[derive(Debug)]
pub struct Link<'a> {
    typ: LinkType,
    dst: Cow<'a, str>,
    title: Cow<'a, str>,
    text: Vec<u8>,
}

impl<'a> State<'a> for Link<'a> {
    fn new(tag: Tag<'a>, gen: &mut Generator<'a, impl Document<'a>, impl Write>) -> Result<Self> {
        let (typ, dst, title) = match tag {
            Tag::Link(typ, dst, title) => (typ, dst, title),
            _ => unreachable!(),
        };
        Ok(Link {
            typ,
            dst,
            title,
            text: Vec::new(),
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.text)
    }

    fn finish(self, gen: &mut Generator<'a, impl Document<'a>, impl Write>, peek: Option<&Event<'a>>) -> Result<()> {
        let out = gen.get_out();
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
        let text = String::from_utf8(self.text).expect("invalid UTF8");

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
