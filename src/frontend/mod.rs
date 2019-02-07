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
        if let Some(evt) = self.try_end_tag() {
            return Some(evt);
        }

        let evt = self.parser.next()?;
        dbg!(&evt);
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
            state: State::Nothing,
            marker: PhantomData,
        }
    }

    fn convert_event(&mut self, evt: CmarkEvent<'a>) {
        match evt {
            CmarkEvent::Start(CmarkTag::Code) => Some(self.handle_inline_code()),
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => Some(self.handle_code_block(lang)),
            CmarkEvent::Start(tag) => self.convert_tag(tag),
            CmarkEvent::End(_) => panic!("End tag should be consumed when handling the start tag"),
            evt => self.convert_event(evt)
        }
        match evt {
            CmarkEvent::Start(tag) => self.convert_tag(tag, true, None),
            CmarkEvent::End(tag) => self.convert_tag(tag, false, None),
            CmarkEvent::Text(text) => Some(Event::Text(text)),
            CmarkEvent::Html(html) => Some(Event::Html(html)),
            CmarkEvent::InlineHtml(html) => Some(Event::InlineHtml(html)),
            CmarkEvent::FootnoteReference(label) => Some(Event::FootnoteReference(FootnoteReference {
                label,
            })),
            CmarkEvent::SoftBreak => Some(Event::SoftBreak),
            CmarkEvent::HardBreak => Some(Event::HardBreak),
        }
    }

    fn convert_until_end(&mut self) {
        let mut depth = 0;
        loop {
            match self.parser.peek().unwrap() {
                
            }
        }
    }

    fn convert_code(&mut self) {
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
        self.convert_until_end();
        self.buffer.push_back(tag);
    }

    /// Returns None if an Event was ignored, but no further Event is in the `parser`
    fn convert_tag(&mut self, tag: CmarkTag<'a>, start: bool, mut cskvp: Option<Cskvp<'a>>) -> Option<Event<'a>> {
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
                        evt => Some(self.convert_event(evt).unwrap().into()),
                    });
                // assume no Link / Image in alt-text of image
                gen.visit_event(self.convert_event(evt).unwrap().into(), peek.as_ref())
                    .expect("writing to Vec<u8> shouldn't fail");
            }
            String::from_utf8(out).expect("invalid utf8")
        };

        Some(f(match tag {
            CmarkTag::Paragraph => {
                match self.try_label(f) {
                    Some(evt) => return Some(evt),
                    // got label, but couldn't be applied, ignore it and continue
                    None => return self.convert_event(self.parser.next()?)
                }
            },
            CmarkTag::Rule => Tag::Rule,
            CmarkTag::Header(level) => {
                assert_eq!(start, true, "Header end should be consumed when parsing start");
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
                let mut text = String::new();
                loop {
                    let evt = self.parser.next().unwrap();

                    match &evt {
                        CmarkEvent::Text(t) => text.push_str(t),
                        CmarkEvent::End(CmarkTag::Header(level)) => {
                            // consume end event
                            self.buffer.push_back(Event::End(Tag::Header(Header {
                                // dummy data, end event can be broken
                                label: Cow::Borrowed(""),
                                level: *level,
                            })));
                            break;
                        }
                        _ => (),
                    }
                    self.buffer.push_back(self.convert_event(evt).unwrap());
                }

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
                    prefix.map(|label| Cow::Borrowed(label))
                        .or_else(|| inline.map(|inline| Cow::Owned(inline.to_string())))
                        .unwrap_or_else(|| Cow::Owned(autogenerated))
                };

                Tag::Header(Header {
                    label,
                    level
                })
            },
            CmarkTag::BlockQuote => Tag::BlockQuote,
            CmarkTag::CodeBlock(language) => unreachable!(),
            CmarkTag::List(start_number) if start_number.is_none() => Tag::List,
            CmarkTag::List(start_number) => Tag::Enumerate(Enumerate {
                start_number: start_number.unwrap()
            }),
            CmarkTag::Item => Tag::Item,
            CmarkTag::FootnoteDefinition(label) => Tag::FootnoteDefinition(FootnoteDefinition {
                label
            }),
            CmarkTag::Table(alignment) => Tag::Table(Table {
                label: cskvp.as_mut().and_then(|cskvp| cskvp.take_label()).map(Cow::Borrowed),
                alignment,
            }),
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
                    LinkOrText::Link(link) => return Some(Event::Link(link)),
                    LinkOrText::Text(text) => return Some(Event::Text(text)),
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
                return Some(Event::Include(Include {
                    label: cskvp.as_mut().and_then(|cskvp| cskvp.take_label()).map(Cow::Borrowed),
                    dst,
                    width: None,
                    height: None,
                    caption
                }))
            },
        }))
    }

    fn try_label(&mut self, f: fn(Tag<'a>) -> Event) -> Option<Event<'a>> {
        // check for label/config (Start(Paragraph), Text("{#foo,config...}"), End(Paragraph))
        if let Some(CmarkEvent::Text(text)) = self.parser.peek() {
            let text = text.trim();
            if text.starts_with('{') && text.ends_with('}') {
                let text = self.parser.next().unwrap();
                if let Some(CmarkEvent::End(CmarkTag::Paragraph)) = self.parser.peek() {
                    let text = match text {
                        CmarkEvent::Text(text) => text,
                        _ => unreachable!()
                    };
                    // label
                    let _ = self.parser.next().unwrap();
                    let cskvp = Cskvp::new(&text[1..text.len()-1]);
                    // if next element could have a label, convert that element with the label
                    match self.parser.peek() {
                        Some(CmarkEvent::Start(CmarkTag::Header(_)))
                        | Some(CmarkEvent::Start(CmarkTag::CodeBlock(_)))
                        | Some(CmarkEvent::Start(CmarkTag::Table(_)))
                        | Some(CmarkEvent::Start(CmarkTag::Image(..))) => {
                            if let CmarkEvent::Start(tag) =  self.parser.next().unwrap() {
                                return self.convert_tag(tag, true, Some(cskvp))
                            } else {
                                unreachable!()
                            }
                        }
                        _ => {
                            // TODO error
                            println!("got label / config, but there wasn't an element to\
                             apply it to: {:?}", text);
                            return None;
                        }
                    }
                } else {
                    // just unlucky, reset everything
                    self.buffer.push_back(self.convert_event(text).unwrap());
                    return Some(f(Tag::Paragraph));
                }
            }
        }
        return Some(f(Tag::Paragraph))
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
                self.convert_event(evt)
            }
            // ignore everything in code blocks
            State::CodeBlock | State::Equation | State::NumberedEquation | State::Graphviz(_) => {
                let evt = self.parser.next().unwrap();
                if let CmarkEvent::End(CmarkTag::CodeBlock(_)) = &evt {
                    let state = mem::replace(&mut self.state, State::Nothing);
                    match state {
                        State::Nothing | State::Math => unreachable!(),
                        State::CodeBlock => return self.convert_event(evt),
                        State::Equation => return Some(Event::End(Tag::Equation)),
                        State::NumberedEquation => return Some(Event::End(Tag::NumberedEquation)),
                        State::Graphviz(g) => return Some(Event::End(Tag::Graphviz(g))),
                    }
                }
                self.convert_event(evt)
            }
        }
    }

    fn handle_inline_code(&mut self) -> Event<'a> {
    }

    fn handle_code_block(&mut self, lang: Cow<'a, str>) -> Event<'a> {
        let lang = match lang {
            Cow::Borrowed(s) => s,
            Cow::Owned(_) => unreachable!(),
        };
        if let Some(pos) = lang.find(',') {
            if cskvp.is_some() {
                // TODO: error
                println!("Code has both prefix and inline style labels / config, ignoring both");
                // don't print warnings about unused properties
                // will be cleaned up as it's on the stack anyways
                mem::forget(cskvp.take());
            } else {
                cskvp = Some(Cskvp::new(&language[pos+1..]));
            }
        }
        Tag::CodeBlock(CodeBlock {
            label: cskvp.as_mut().and_then(|cskvp| cskvp.take_label()).map(Cow::Borrowed),
            language: Cow::Borrowed(&language[..language.find(',').unwrap_or(language.len())]),
        })
        let mut cskvp = Cskvp::new(lang);
        let res = match cskvp.take_single_by_index(0) {
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
                    label: cskvp.take_label(),
                    scale: cskvp.take_double("scale"),
                    width: cskvp.take_double("width"),
                    height: cskvp.take_double("height"),
                    caption: cskvp.take_double("caption"),
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

