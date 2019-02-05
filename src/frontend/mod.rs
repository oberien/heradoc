use std::borrow::Cow;
use std::collections::{VecDeque, HashMap};
use std::mem;
use std::marker::PhantomData;
use std::iter::Peekable;

use pulldown_cmark::{Event as CmarkEvent, Tag as CmarkTag, Parser as CmarkParser};

mod refs;
mod event;

pub use self::refs::*;
pub use self::event::*;

use self::refs::LinkOrText;
use crate::config::Config;
use crate::backend::{Backend};
use crate::generator::PrimitiveGenerator;
use crate::cskvp::Cskvp;

#[derive(Debug, Clone)]
enum State<'a> {
    Nothing,
    Math,
    CodeBlock,
    Equation,
    NumberedEquation,
    Graphviz(Graphviz<'a>),
}

pub struct Frontend<'a, B: Backend<'a>> {
    cfg: &'a Config,
    parser: Peekable<CmarkParser<'a>>,
    buffer: VecDeque<Event<'a>>,
    state: State<'a>,
    marker: PhantomData<B>,
}

impl<'a, B: Backend<'a>> Iterator for Frontend<'a, B> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if let Some(evt) = self.buffer.pop_front() {
            return Some(evt);
        }
        if let Some(evt) = self.try_end_tag() {
            return Some(evt);
        }

        let evt = self.parser.next()?;
        match evt {
            CmarkEvent::Start(CmarkTag::Code) => Some(self.handle_inline_code()),
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => Some(self.handle_code_block(lang)),
            evt => Some(self.convert_event(evt))
        }
    }
}


impl<'a, B: Backend<'a>> Frontend<'a, B> {
    pub fn new(cfg: &'a Config, parser: CmarkParser<'a>) -> Frontend<'a, B> {
        Frontend {
            cfg,
            parser: parser.peekable(),
            buffer: VecDeque::new(),
            state: State::Nothing,
            marker: PhantomData,
        }
    }

    fn convert_event(&mut self, evt: CmarkEvent<'a>) -> Event<'a> {
        match evt {
            CmarkEvent::Start(tag) => self.convert_tag(tag, true),
            CmarkEvent::End(tag) => self.convert_tag(tag, false),
            CmarkEvent::Text(text) => Event::Text(text),
            CmarkEvent::Html(html) => Event::Html(html),
            CmarkEvent::InlineHtml(html) => Event::InlineHtml(html),
            CmarkEvent::FootnoteReference(label) => Event::FootnoteReference(FootnoteReference {
                label,
            }),
            CmarkEvent::SoftBreak => Event::SoftBreak,
            CmarkEvent::HardBreak => Event::HardBreak,
        }
    }

    fn convert_tag(&mut self, tag: CmarkTag<'a>, start: bool, cskvp: Option<Cskvp<'a>>) -> Event<'a> {
        let f = match start {
            true => Event::Start,
            false => Event::End,
        };

        // TODO: don't render content but return Vec<Event> instead (less coupling)
        let mut get_content = |f: &dyn Fn(&CmarkTag<'a>) -> bool| {
            let mut out = Vec::new();
            let mut gen: PrimitiveGenerator<'a, B, _> = PrimitiveGenerator::without_context(self.cfg, &mut out);
            loop {
                let evt = self.parser.next().unwrap();
                // Commonmark doesn't allow nested links, so we can just break on the next one
                if let CmarkEvent::End(tag) = &evt {
                    if f(tag) {
                        break;
                    }
                }
                let peek = self.parser.peek().cloned()
                    .and_then(|evt| match evt {
                        // if the end tag would be peeked, use None instead as it isn't transformed
                        CmarkEvent::End(ref tag) if f(tag) => None,
                        evt => Some(self.convert_event(evt).into()),
                    });
                // assume no Link / Image in alt-text of image
                gen.visit_event(self.convert_event(evt).into(), peek.as_ref())
                    .expect("writing to Vec<u8> shouldn't fail");
            }
            String::from_utf8(out).expect("invalid utf8")
        };

        f(match tag {
            CmarkTag::Paragraph => {
                // check for label (Start(Paragraph), Text("{#foo,config...}"), End(Paragraph))
                if let Some(CmarkEvent::Text(text)) = self.parser.peek() {
                    if text.starts_with('{') && text.ends_with('}') {
                        let text = self.parser.next().unwrap();
                        if let Some(CmarkEvent::End(CmarkTag::Paragraph)) = self.parser.peek() {
                            // label
                            let _ = self.parser.next().unwrap();
                            let cskvp = Cskvp::new(&text[1..text.len()-1]);
                            // if next element could have a label, convert that element with the label
                            match self.parser.peek() {
                                CmarkEvent::Start(CmarkTag::Header(_))
                                | CmarkEvent::Start(CmarkTag::CodeBlock(_))
                                | CmarkEvent::Start(CmarkTag::Table(_))
                                | CmarkEvent::Start(CmarkTag::Image(_))
                            }
                        } else {
                            // just unlucky, reset everything
                            self.buffer.push(self.convert_event(text));
                            return Event::Start(Tag::Paragraph);
                        }
                    }
                }
                Tag::Paragraph
            },
            CmarkTag::Rule => Tag::Rule,
            CmarkTag::Header(level) => Tag::Header(Header { level }),
            CmarkTag::BlockQuote => Tag::BlockQuote,
            CmarkTag::CodeBlock(language) => Tag::CodeBlock(CodeBlock { language }),
            CmarkTag::List(start_number) if start_number.is_none() => Tag::List,
            CmarkTag::List(start_number) => Tag::Enumerate(Enumerate {
                start_number: start_number.unwrap()
            }),
            CmarkTag::Item => Tag::Item,
            CmarkTag::FootnoteDefinition(label) => Tag::FootnoteDefinition(FootnoteDefinition {
                label
            }),
            CmarkTag::Table(alignment) => Tag::Table(Table { alignment }),
            CmarkTag::TableHead => Tag::TableHead,
            CmarkTag::TableRow => Tag::TableRow,
            CmarkTag::TableCell => Tag::TableCell,
            CmarkTag::Emphasis => Tag::InlineEmphasis,
            CmarkTag::Strong => Tag::InlineStrong,
            CmarkTag::Code => Tag::InlineCode,
            CmarkTag::Link(typ, dst, title) => {
                assert!(start, "Link is consumed fully at start, there shouldn't ever be an end tag");
                let content = get_content(&|t| if let CmarkTag::Link(..) = t { true } else { false });

                match refs::parse_references(self.cfg, typ, dst, title, content) {
                    LinkOrText::Link(link) => return Event::Link(link),
                    LinkOrText::Text(text) => return Event::Text(text),
                }
            }
            CmarkTag::Image(typ, dst, title) => {
                // always consume the full image including end tag
                let content = get_content(&|t| if let CmarkTag::Image(..) = t { true } else { false });
                let caption = match typ {
                    LinkType::Reference | LinkType::ReferenceUnknown =>
                        Some(content),
                    LinkType::Collapsed | LinkType::CollapsedUnknown
                    | LinkType::Shortcut | LinkType::ShortcutUnknown
                    | LinkType::Inline | LinkType::Autolink => None
                };
                // TODO: parse title to extract other information
                return Event::Include(Include { dst, width: None, height: None, caption })
            },
        })
    }

    fn try_end_tag(&mut self) -> Option<Event<'a>> {
        // match and create proper end tags
        match self.state {
            State::Nothing => None,
            State::Math => {
                let evt = self.parser.next().unwrap();
                if let CmarkEvent::End(CmarkTag::Code) = &evt {
                    self.state = State::Nothing;
                    return Some(Event::End(Tag::InlineMath));
                }
                Some(self.convert_event(evt))
            }
            // ignore everything in code blocks
            State::CodeBlock | State::Equation | State::NumberedEquation | State::Graphviz(_) => {
                let evt = self.parser.next().unwrap();
                if let CmarkEvent::End(CmarkTag::CodeBlock(_)) = &evt {
                    let state = mem::replace(&mut self.state, State::Nothing);
                    match state {
                        State::Nothing | State::Math => unreachable!(),
                        State::CodeBlock => return Some(self.convert_event(evt)),
                        State::Equation => return Some(Event::End(Tag::Equation)),
                        State::NumberedEquation => return Some(Event::End(Tag::NumberedEquation)),
                        State::Graphviz(g) => return Some(Event::End(Tag::Graphviz(g))),
                    }
                }
                Some(self.convert_event(evt))
            }
        }
    }

    fn handle_inline_code(&mut self) -> Event<'a> {
        // peek if code is math mode
        let inner = self.parser.next().unwrap();
        let text = match inner {
            CmarkEvent::Text(text) => text,
            e => unreachable!("InlineCode should always be followed by Text but was fallowed by {:?}", e),
        };
        if text.starts_with("$ ") {
            let text = match text {
                Cow::Borrowed(s) => Cow::Borrowed(&s[2..]),
                Cow::Owned(mut s) => {
                    s.drain(..2);
                    Cow::Owned(s)
                }
            };
            self.buffer.push_back(Event::Text(text));
            self.state = State::Math;
            Event::Start(Tag::InlineMath)
        } else {
            self.buffer.push_back(Event::Text(text));
            Event::Start(Tag::InlineCode)
        }
    }

    fn handle_code_block(&mut self, lang: Cow<'a, str>) -> Event<'a> {
        let lang = match lang {
            Cow::Borrowed(s) => s,
            Cow::Owned(_) => unreachable!(),
        };
        let mut cskvp = Cskvp::new(lang);
        let res = match cskvp.single_remove(0) {
            Some("equation") | Some("$$") => {
                self.state = State::Equation;
                Event::Start(Tag::Equation)
            }
            Some("numberedequation") | Some("$$$") => {
                self.state = State::NumberedEquation;
                Event::Start(Tag::NumberedEquation)
            }
            Some("graphviz") => {
                let graphviz = Graphviz {
                    scale: cskvp.double("scale"),
                    width: cskvp.double("width"),
                    height: cskvp.double("height"),
                    caption: cskvp.double("caption"),
                };
                self.state = State::Graphviz(graphviz.clone());
                Event::Start(Tag::Graphviz(graphviz))
            }
            _ => {
                self.state = State::CodeBlock;
                Event::Start(Tag::CodeBlock(CodeBlock { language: Cow::Borrowed(lang) }))
            }
        };
        res
    }
}

