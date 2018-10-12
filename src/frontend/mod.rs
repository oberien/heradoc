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

    fn convert_tag(&mut self, tag: CmarkTag<'a>, start: bool) -> Event<'a> {
        let f = match start {
            true => Event::Start,
            false => Event::End,
        };

        f(match tag {
            CmarkTag::Paragraph => Tag::Paragraph,
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
                let mut out = Vec::new();
                let mut gen: PrimitiveGenerator<'a, B, _> = PrimitiveGenerator::new(self.cfg, &mut out);
                loop {
                    let evt = self.parser.next().unwrap();
                    // Commonmark doesn't allow nested links, so we can just break on the next one
                    if let CmarkEvent::End(CmarkTag::Link(..)) = evt {
                        break;
                    }
                    let peek = self.parser.peek().cloned()
                        .and_then(|evt| match evt {
                            // if the link end tag would be peeked, use None instead as it isn't translated
                            CmarkEvent::End(CmarkTag::Link(..)) => None,
                            evt => Some(self.convert_event(evt)),
                        });
                    gen.visit_event(self.convert_event(evt), peek.as_ref())
                        .expect("writing to Vec<u8> shouldn't fail");
                }
                let content = String::from_utf8(out).expect("invalid utf8");

                match refs::parse_references(self.cfg, typ, dst, title, content) {
                    LinkOrText::Link(link) => return Event::Link(link),
                    LinkOrText::Text(text) => return Event::Text(text),
                }
            }
            CmarkTag::Image(typ, dst, title) => Tag::Image(Image { typ, dst, title }),
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
        let (single, mut double) = parse_attributes(lang);
        let res = match single[0] {
            "equation" | "math" | "$$" => {
                self.state = State::Equation;
                Event::Start(Tag::Equation)
            }
            "numberedequation" | "$$$" => {
                self.state = State::NumberedEquation;
                Event::Start(Tag::NumberedEquation)
            }
            "graphviz" => {
                let graphviz = Graphviz {
                    scale: double.remove("scale"),
                    width: double.remove("width"),
                    height: double.remove("height"),
                    caption: double.remove("caption"),
                    label: double.remove("label"),
                };
                self.state = State::Graphviz(graphviz.clone());
                Event::Start(Tag::Graphviz(graphviz))
            }
            _ => {
                self.state = State::CodeBlock;
                Event::Start(Tag::CodeBlock(CodeBlock { language: Cow::Borrowed(lang) }))
            }
        };
        for (k, v) in double {
            // TODO: log instead of print
            println!("Unknown attribute `{}={}`", k, v);
        }
        for attr in single.into_iter().skip(1) {
            // TODO: log instead of print
            println!("Unknown attribute `{}`", attr);
        }
        res
    }
}

fn parse_attributes(s: &str) -> (Vec<&str>, HashMap<&str, &str>) {
    let mut single = Vec::new();
    let mut double = HashMap::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.contains("=") {
            let i = part.find('=').unwrap();
            double.insert(&part[..i], &part[(i+1)..]);
        } else {
            single.push(part);
        }
    }
    (single, double)
}

