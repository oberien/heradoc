use std::borrow::Cow;
use std::collections::{VecDeque, HashMap};
use std::mem;

use pulldown_cmark::{Alignment, LinkType, Event as CmarkEvent, Tag as CmarkTag, Parser as CmarkParser};

#[derive(Debug, Clone)]
enum State<'a> {
    Nothing,
    Math,
    CodeBlock,
    Equation,
    NumberedEquation,
    Graphviz(Graphviz<'a>),
}

pub struct Parser<'a> {
    parser: CmarkParser<'a>,
    buffer: VecDeque<Event<'a>>,
    state: State<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(parser: CmarkParser<'a>) -> Parser {
        Parser {
            parser,
            buffer: VecDeque::new(),
            state: State::Nothing,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if let Some(evt) = self.buffer.pop_front() {
            return Some(evt);
        }
        // match and create proper end tags
        match self.state {
            State::Nothing => (),
            State::Math => {
                let evt = self.parser.next().unwrap();
                if let CmarkEvent::End(CmarkTag::Code) = &evt {
                    self.state = State::Nothing;
                    return Some(Event::End(Tag::InlineMath));
                }
                return Some(evt.into());
            }
            // ignore everything in code blocks
            State::CodeBlock | State::Equation | State::NumberedEquation | State::Graphviz(_) => {
                let evt = self.parser.next().unwrap();
                if let CmarkEvent::End(CmarkTag::CodeBlock(_)) = &evt {
                    let state = mem::replace(&mut self.state, State::Nothing);
                    match state {
                        State::Nothing | State::Math => unreachable!(),
                        State::CodeBlock => return Some(evt.into()),
                        State::Equation => return Some(Event::End(Tag::Equation)),
                        State::NumberedEquation => return Some(Event::End(Tag::NumberedEquation)),
                        State::Graphviz(g) => return Some(Event::End(Tag::Graphviz(g))),
                    }
                }
                return Some(evt.into());
            }
        }

        let evt = self.parser.next()?;
        match evt {
            CmarkEvent::Start(CmarkTag::Code) => {
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
                    Some(Event::Start(Tag::InlineMath))
                } else {
                    self.buffer.push_back(Event::Text(text));
                    Some(Event::Start(Tag::InlineCode))
                }
            }
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => {
                let lang = match lang {
                    Cow::Borrowed(s) => s,
                    Cow::Owned(_) => unreachable!(),
                };
                let (single, mut double) = parse_attributes(lang);
                let res = match single[0] {
                    "equation" | "math" | "$$" => {
                        self.state = State::Equation;
                        Some(Event::Start(Tag::Equation))
                    }
                    "numberedequation" | "$$$" => {
                        self.state = State::NumberedEquation;
                        Some(Event::Start(Tag::NumberedEquation))
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
                        Some(Event::Start(Tag::Graphviz(graphviz)))
                    }
                    _ => Some(Event::Start(Tag::CodeBlock(CodeBlock { language: Cow::Borrowed(lang) })))
                };
                for (k, v) in double {
                    println!("Unknown attribute `{}={}`", k, v);
                }
                for attr in single.into_iter().skip(1) {
                    println!("Unknown attribute `{}`", attr);
                }
                res
            }
            evt => Some(evt.into())
        }
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


// extension of pulldown_cmark::Event with custom types
#[derive(Debug)]
pub enum Event<'a> {
    Start(Tag<'a>),
    End(Tag<'a>),
    Text(Cow<'a, str>),
    Html(Cow<'a, str>),
    InlineHtml(Cow<'a, str>),
    FootnoteReference(FootnoteReference<'a>),
    SoftBreak,
    HardBreak,
}

#[derive(Debug)]
pub struct FootnoteReference<'a> {
    pub label: Cow<'a, str>,
}

// extension of pulldown_cmark::Tag with custom types
#[derive(Debug)]
pub enum Tag<'a> {
    Paragraph,
    Rule,
    Header(Header),
    BlockQuote,
    CodeBlock(CodeBlock<'a>),
    List,
    Enumerate(Enumerate),
    Item,
    FootnoteDefinition(FootnoteDefinition<'a>),

    Table(Table),
    TableHead,
    TableRow,
    TableCell,

    InlineEmphasis,
    InlineStrong,
    InlineCode,
    InlineMath,

    Link(Link<'a>),
    Image(Link<'a>),

    Equation,
    NumberedEquation,
    Graphviz(Graphviz<'a>),
}

#[derive(Debug)]
pub struct Header {
    pub level: i32,
}

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub language: Cow<'a, str>,
}

#[derive(Debug)]
pub struct Enumerate {
    pub start_number: usize,
}

#[derive(Debug)]
pub struct FootnoteDefinition<'a> {
    pub label: Cow<'a, str>,
}

#[derive(Debug)]
pub struct Table {
    pub alignment: Vec<Alignment>,
}

#[derive(Debug)]
pub struct Link<'a> {
    pub typ: LinkType,
    pub dst: Cow<'a, str>,
    pub title: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub struct Graphviz<'a> {
    pub scale: Option<&'a str>,
    pub width: Option<&'a str>,
    pub height: Option<&'a str>,
    pub caption: Option<&'a str>,
    pub label: Option<&'a str>,
}

impl<'a> From<CmarkEvent<'a>> for Event<'a> {
    fn from(evt: CmarkEvent<'a>) -> Self {
        match evt {
            CmarkEvent::Start(tag) => Event::Start(tag.into()),
            CmarkEvent::End(tag) => Event::End(tag.into()),
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
}

impl<'a> From<CmarkTag<'a>> for Tag<'a> {
    fn from(tag: CmarkTag<'a>) -> Self {
        match tag {
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
            CmarkTag::Link(typ, dst, title) => Tag::Link(Link { typ, dst, title }),
            CmarkTag::Image(typ, dst, title) => Tag::Image(Link { typ, dst, title }),
        }
    }
}
