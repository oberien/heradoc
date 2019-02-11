use std::borrow::Cow;
use std::collections::VecDeque;
use std::mem;
use std::marker::PhantomData;
use std::iter::Peekable;

use pulldown_cmark::{Event as CmarkEvent, Tag as CmarkTag, Parser as CmarkParser};
use regex::Regex;
use lazy_static::lazy_static;

mod refs;
mod event;

pub use self::refs::*;
pub use self::event::*;

use self::refs::LinkOrText;
use crate::config::Config;
use crate::backend::{Backend};
use crate::generator::PrimitiveGenerator;
use crate::cskvp::Cskvp;
use crate::ext::CowExt;

pub struct Frontend<'a, B: Backend<'a>> {
    cfg: &'a Config,
    parser: Peekable<CmarkParser<'a>>,
    buffer: VecDeque<Event<'a>>,
    marker: PhantomData<B>,
}

impl<'a, B: Backend<'a>> Iterator for Frontend<'a, B> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if let Some(evt) = self.buffer.pop_front() {
            return Some(evt);
        }

        let evt = self.parser.next()?;
        self.convert_event(evt);
        self.buffer.pop_front()
    }
}


impl<'a, B: Backend<'a>> Frontend<'a, B> {
    pub fn new(cfg: &'a Config, parser: CmarkParser<'a>) -> Frontend<'a, B> {
        Frontend {
            cfg,
            parser: parser.peekable(),
            buffer: VecDeque::new(),
            marker: PhantomData,
        }
    }

    fn convert_event(&mut self, evt: CmarkEvent<'a>) {
        match evt {
            CmarkEvent::Text(text) => self.buffer.push_back(Event::Text(text)),
            CmarkEvent::Html(html) => self.buffer.push_back(Event::Html(html)),
            CmarkEvent::InlineHtml(html) => self.buffer.push_back(Event::InlineHtml(html)),
            CmarkEvent::FootnoteReference(label) => self.buffer.push_back(Event::FootnoteReference(FootnoteReference {
                label,
            })),
            CmarkEvent::SoftBreak => self.buffer.push_back(Event::SoftBreak),
            CmarkEvent::HardBreak => self.buffer.push_back(Event::HardBreak),

            // TODO: make this not duplicate
            CmarkEvent::Start(CmarkTag::Rule) => self.buffer.push_back(Event::Start(Tag::Rule)),
            CmarkEvent::End(CmarkTag::Rule) => self.buffer.push_back(Event::End(Tag::Rule)),
            CmarkEvent::Start(CmarkTag::BlockQuote) => self.buffer.push_back(Event::Start(Tag::BlockQuote)),
            CmarkEvent::End(CmarkTag::BlockQuote) => self.buffer.push_back(Event::End(Tag::BlockQuote)),
            CmarkEvent::Start(CmarkTag::List(start_number)) if start_number.is_none() => {
                self.buffer.push_back(Event::Start(Tag::List))
            },
            CmarkEvent::End(CmarkTag::List(start_number)) if start_number.is_none() => {
                self.buffer.push_back(Event::End(Tag::List))
            },
            CmarkEvent::Start(CmarkTag::List(start_number)) => {
                self.buffer.push_back(Event::Start(Tag::Enumerate(Enumerate {
                    start_number: start_number.unwrap()
                })))
            },
            CmarkEvent::End(CmarkTag::List(start_number)) => {
                self.buffer.push_back(Event::End(Tag::Enumerate(Enumerate {
                    start_number: start_number.unwrap()
                })))
            },
            CmarkEvent::Start(CmarkTag::Item) => self.buffer.push_back(Event::Start(Tag::Item)),
            CmarkEvent::End(CmarkTag::Item) => self.buffer.push_back(Event::End(Tag::Item)),
            CmarkEvent::Start(CmarkTag::FootnoteDefinition(label)) => {
                self.buffer.push_back(Event::Start(Tag::FootnoteDefinition(FootnoteDefinition { label })))
            },
            CmarkEvent::End(CmarkTag::FootnoteDefinition(label)) => {
                self.buffer.push_back(Event::End(Tag::FootnoteDefinition(FootnoteDefinition { label })))
            },
            CmarkEvent::Start(CmarkTag::TableHead) => self.buffer.push_back(Event::Start(Tag::TableHead)),
            CmarkEvent::End(CmarkTag::TableHead) => self.buffer.push_back(Event::End(Tag::TableHead)),
            CmarkEvent::Start(CmarkTag::TableRow) => self.buffer.push_back(Event::Start(Tag::TableRow)),
            CmarkEvent::End(CmarkTag::TableRow) => self.buffer.push_back(Event::End(Tag::TableRow)),
            CmarkEvent::Start(CmarkTag::TableCell) => self.buffer.push_back(Event::Start(Tag::TableCell)),
            CmarkEvent::End(CmarkTag::TableCell) => self.buffer.push_back(Event::End(Tag::TableCell)),
            CmarkEvent::Start(CmarkTag::Emphasis) => self.buffer.push_back(Event::Start(Tag::InlineEmphasis)),
            CmarkEvent::End(CmarkTag::Emphasis) => self.buffer.push_back(Event::End(Tag::InlineEmphasis)),
            CmarkEvent::Start(CmarkTag::Strong) => self.buffer.push_back(Event::Start(Tag::InlineStrong)),
            CmarkEvent::End(CmarkTag::Strong) => self.buffer.push_back(Event::End(Tag::InlineStrong)),

            CmarkEvent::Start(CmarkTag::Code) => self.convert_inline_code(),
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => self.convert_code_block(lang, None),
            CmarkEvent::Start(CmarkTag::Paragraph) => self.convert_paragraph(),
            CmarkEvent::Start(CmarkTag::Header(level)) => self.convert_header(level, None),
            CmarkEvent::Start(CmarkTag::Table(alignment)) => self.convert_table(alignment, None),
            CmarkEvent::Start(CmarkTag::Link(typ, dst, title)) => self.convert_link(typ, dst, title),
            CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => self.convert_image(typ, dst, title, None),

            CmarkEvent::End(CmarkTag::Code)
            | CmarkEvent::End(CmarkTag::CodeBlock(_))
            | CmarkEvent::End(CmarkTag::Paragraph)
            | CmarkEvent::End(CmarkTag::Header(_))
            | CmarkEvent::End(CmarkTag::Table(_))
            | CmarkEvent::End(CmarkTag::Link(..))
            | CmarkEvent::End(CmarkTag::Image(..)) => {
                panic!("End tag should be consumed when handling the start tag: {:?}", evt)
            },
        }
    }

    /// Consumes and converts all elements until the next End event is received.
    /// Returns a concatenation of all text events (unrendered).
    #[inline]
    fn convert_until_end_inclusive(&mut self, f: impl Fn(&CmarkTag) -> bool) -> String {
        let mut text = String::new();
        loop {
            let evt = self.parser.next().unwrap();
            match &evt {
                CmarkEvent::End(tag) if f(tag) => {
                    return text
                },
                CmarkEvent::Text(t) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(t);
                }
                _ => (),
            }

            self.convert_event(evt);
        }
    }


    /// Consumes all events, rendering their result.
    // TODO: don't render content but return Vec<Event> instead (less coupling with generator)
    fn render_until_end_inclusive(&mut self, f: impl Fn(&CmarkTag) -> bool) -> String {
        let mut out = Vec::new();
        let mut gen: PrimitiveGenerator<'a, B, _> = PrimitiveGenerator::without_context(self.cfg, &mut out);
        let buffer = mem::replace(&mut self.buffer, VecDeque::new());
        loop {
            let evt = self.parser.next().unwrap();
            match &evt {
                CmarkEvent::End(tag) if f(tag) => {
                    break
                },
                _ => {},
            }
            self.convert_event(evt);
        }
        let buffer = mem::replace(&mut self.buffer, buffer);
        let mut iter = buffer.into_iter();
        let mut next = iter.next().map(|e| e.into());
        let mut peek = iter.next().map(|e| e.into());
        while let Some(evt) = next {
            gen.visit_event(evt, peek.as_ref())
                .expect("writing to Vec<u8> shouldn't fail");
            next = peek;
            peek = iter.next().map(|e| e.into());
        }
        String::from_utf8(out).expect("invalid utf8")

    }

    fn convert_inline_code(&mut self) {
        // check if code is math mode
        let mut text = match self.parser.next().unwrap() {
            CmarkEvent::Text(text) => text,
            CmarkEvent::End(CmarkTag::Code) => {
                self.buffer.push_back(Event::Start(Tag::InlineCode));
                self.buffer.push_back(Event::End(Tag::InlineCode));
                return;
            }
            e => unreachable!("InlineCode should always be followed by Text or End(Code) but was fallowed by {:?}", e),
        };
        let tag = if text.starts_with("$ ") {
            text.truncate_left(2);
            Tag::InlineMath
        } else {
            Tag::InlineCode
        };
        self.buffer.push_back(Event::Start(tag.clone()));
        self.buffer.push_back(Event::Text(text));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Code = t { true } else { false });
        self.buffer.push_back(Event::End(tag));
    }

    fn convert_code_block(&mut self, lang: Cow<'a, str>, mut cskvp: Option<Cskvp<'a>>) {
        let lang = match lang {
            Cow::Borrowed(s) => s,
            Cow::Owned(_) => unreachable!("CodeBlock language should be borrowed"),
        };

        // check if language has label/config
        let language;
        if let Some(pos) = lang.find(',') {
            language = &lang[..pos];
            if cskvp.is_some() {
                // TODO: error
                println!("Code has both prefix and inline style labels / config, ignoring both");
                // don't print warnings about unused properties
                // will be cleaned up as it's on the stack anyways
                mem::forget(cskvp.take());
            } else {
                cskvp = Some(Cskvp::new(&lang[pos+1..]));
            }
        } else {
            language = lang;
        }

        let label = cskvp.as_mut().and_then(|cskvp| cskvp.take_label());
        let language = Cow::Borrowed(language);

        let tag = match language.as_ref() {
            "equation" | "$$" => {
                Tag::Equation(Equation { label })
            }
            "numberedequation" | "$$$" => {
                Tag::NumberedEquation(Equation { label })
            }
            "graphviz" => {
                let graphviz = Graphviz {
                    label: cskvp.as_mut().and_then(|cskvp| cskvp.take_label()),
                    scale: cskvp.as_mut().and_then(|cskvp| cskvp.take_double("scale")),
                    width: cskvp.as_mut().and_then(|cskvp| cskvp.take_double("width")),
                    height: cskvp.as_mut().and_then(|cskvp| cskvp.take_double("height")),
                    caption: cskvp.as_mut().and_then(|cskvp| cskvp.take_double("caption")),
                };
                Tag::Graphviz(graphviz)
            }
            _ => {
                Tag::CodeBlock(CodeBlock {
                    label,
                    language,
                })
            }
        };

        self.buffer.push_back(Event::Start(tag.clone()));
        self.convert_until_end_inclusive(|t| if let CmarkTag::CodeBlock(_) = t { true } else { false });
        self.buffer.push_back(Event::End(tag));
    }

    fn convert_paragraph(&mut self) {
        // check for label/config (Start(Paragraph), Text("{#foo,config...}"), End(Paragraph))
        // TODO: make this not ugly
        if let Some(CmarkEvent::Text(text)) = self.parser.peek() {
            let text = text.trim();
            if text.starts_with('{') && text.ends_with('}') {
                let text = match self.parser.next().unwrap() {
                    CmarkEvent::Text(Cow::Borrowed(text)) => text,
                    CmarkEvent::Text(Cow::Owned(_)) => panic!("CmarkEvent::Text contains Cow::Owned"),
                    _ => unreachable!()
                };
                match self.parser.peek() {
                    Some(CmarkEvent::End(CmarkTag::Paragraph))
                    | Some(CmarkEvent::SoftBreak) => {
                        let _ = self.parser.next().unwrap();
                        // label
                        let mut cskvp = Cskvp::new(&text[1..text.len()-1]);
                        // if next element could have a label, convert that element with the label
                        match self.parser.peek() {
                            Some(CmarkEvent::Start(CmarkTag::Header(_)))
                            | Some(CmarkEvent::Start(CmarkTag::CodeBlock(_)))
                            | Some(CmarkEvent::Start(CmarkTag::Table(_)))
                            | Some(CmarkEvent::Start(CmarkTag::Image(..))) => match self.parser.next().unwrap() {
                                CmarkEvent::Start(CmarkTag::Header(label)) => self.convert_header(label, Some(cskvp)),
                                CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => self.convert_code_block(lang, Some(cskvp)),
                                CmarkEvent::Start(CmarkTag::Table(alignment)) => self.convert_table(alignment, Some(cskvp)),
                                CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => self.convert_image(typ, dst, title, Some(cskvp)),
                                _ => unreachable!(),
                            }
                            _ => {
                                if !cskvp.has_label() {
                                    // TODO error
                                    println!("got element config, but there wasn't an element to\
                             apply it to: {:?}", text);
                                    return;
                                }
                                self.buffer.push_back(Event::Label(cskvp.take_label().unwrap()));
                                return;
                            }
                        }
                    }
                    _ => {
                        // not a label, reset our look-ahead and generate original
                        self.buffer.push_back(Event::Start(Tag::Paragraph));
                        self.buffer.push_back(Event::Text(Cow::Borrowed(text)));
                        self.convert_until_end_inclusive(|t| if let CmarkTag::Paragraph = t { true } else { false });
                        self.buffer.push_back(Event::End(Tag::Paragraph));
                        return;
                    }
                }
            }
        }
        self.buffer.push_back(Event::Start(Tag::Paragraph));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Paragraph = t { true } else { false });
        self.buffer.push_back(Event::End(Tag::Paragraph));
    }

    fn convert_header(&mut self, level: i32, mut cskvp: Option<Cskvp<'a>>) {
        // header can have 3 different labels:
        // • `{#foo}\n\n# Header`: "prefix" style
        // • `# Header {#foo}: "inline" style
        // • `# Header`: "default" style, autogenerating label `header`
        // If both the first and the second are specified, we error.
        // If neither the first or the second are specified, we use the default one.
        // Otherwise we take the one that's specified.
        let prefix = cskvp.as_mut().and_then(|cskvp| cskvp.take_label());
        // Consume elements until end of heading to get its text.
        // Convert them and put them into the buffer because the're still needed.
        let current_index = self.buffer.len();
        let text = self.convert_until_end_inclusive(|t| if let CmarkTag::Header(_) = t { true } else { false });

        lazy_static! {
            static ref RE: Regex = Regex::new(r"\{(#[a-zA-Z0-9-_]+\})\w*$").unwrap();
        }
        let inline = RE.captures(&text).map(|c| c.get(1).unwrap().as_str());

        let autogenerated = text.chars().flat_map(|c| match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => Some(c.to_ascii_lowercase()),
            ' ' => Some('-'),
            _ => None,
        }).collect();

        let label = if prefix.is_some() && inline.is_some() {
            // TODO: error
            println!("Header has both prefix and inline style labels, ignoring both");
            Cow::Owned(autogenerated)
        } else {
            prefix.or_else(|| inline.map(|inline| Cow::Owned(inline.to_string())))
                .unwrap_or_else(|| Cow::Owned(autogenerated))
        };

        let tag = Tag::Header(Header { label, level });
        self.buffer.insert(current_index, Event::Start(tag.clone()));
        self.buffer.push_back(Event::End(tag));
    }

    fn convert_table(&mut self, alignment: Vec<Alignment>, mut cskvp: Option<Cskvp<'a>>) {
        let tag = Tag::Table(Table {
            label: cskvp.as_mut().and_then(|cskvp| cskvp.take_label()),
            alignment,
        });
        self.buffer.push_back(Event::Start(tag.clone()));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Table(_) = t { true } else { false });
        self.buffer.push_back(Event::End(tag));
    }

    fn convert_link(&mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>) {
        let content = self.render_until_end_inclusive(|t| if let CmarkTag::Link(..) = t { true } else { false });
        let evt = match refs::parse_references(self.cfg, typ, dst, title, content) {
            LinkOrText::Link(link) => Event::Link(link),
            LinkOrText::Text(text) => Event::Text(text),
        };
        self.buffer.push_back(evt);
    }

    fn convert_image(&mut self, typ: LinkType, dst: Cow<'a, str>, _title: Cow<'a, str>, mut cskvp: Option<Cskvp<'a>>) {
        let content = self.render_until_end_inclusive(|t| if let CmarkTag::Image(..) = t { true } else { false });
        // TODO: do something with title and alt-text (see issue #18)
        let _alt_text = match typ {
            LinkType::Reference | LinkType::ReferenceUnknown =>
                Some(content),
            LinkType::Collapsed | LinkType::CollapsedUnknown
            | LinkType::Shortcut | LinkType::ShortcutUnknown
            | LinkType::Inline | LinkType::Autolink => None
        };
        self.buffer.push_back(Event::Include(Include {
            label: cskvp.as_mut().and_then(|cskvp| cskvp.take_label()),
            dst,
            width: None,
            height: None,
        }))
    }
}

