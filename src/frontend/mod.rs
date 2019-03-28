use std::borrow::Cow;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::str::FromStr;
use std::ops::Range;

use lazy_static::lazy_static;
use pulldown_cmark::{Options as CmarkOptions, Parser as CmarkParser};
use regex::Regex;

mod concat;
mod convert_cow;
mod event;
mod refs;

pub use self::event::*;
pub use self::refs::LinkType;

use self::concat::Concat;
use self::convert_cow::{ConvertCow, Event as CmarkEvent, Tag as CmarkTag};
use self::refs::ReferenceParseResult;
use crate::backend::Backend;
use crate::config::Config;
use crate::cskvp::Cskvp;
use crate::diagnostics::Diagnostics;
use crate::ext::{CowExt, StrExt};
use crate::resolve::Command;

pub struct Frontend<'a, B: Backend<'a>> {
    cfg: &'a Config,
    diagnostics: Diagnostics<'a>,
    parser: Peekable<Concat<'a, ConvertCow<'a>>>,
    buffer: VecDeque<(Event<'a>, Range<usize>)>,
    marker: PhantomData<B>,
}

impl<'a, B: Backend<'a>> Iterator for Frontend<'a, B> {
    type Item = (Event<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((evt, range)) = self.buffer.pop_front() {
            return Some((evt, range));
        }

        let (evt, range) = self.parser.next()?;
        self.convert_event(evt, range);
        self.buffer.pop_front()
    }
}

fn broken_link_callback(normalized_ref: &str, text_ref: &str) -> Option<(String, String)> {
    let trimmed = normalized_ref.trim();
    if trimmed.starts_with_ignore_ascii_case("include")
        || Command::from_str(trimmed).is_ok()
        || trimmed.starts_with('#')
        || trimmed.starts_with('@')
    {
        Some((normalized_ref.to_string(), text_ref.to_string()))
    } else {
        None
    }
}

impl<'a, B: Backend<'a>> Frontend<'a, B> {
    pub fn new(cfg: &'a Config, markdown: &'a str, diagnostics: Diagnostics<'a>) -> Frontend<'a, B> {
        let parser = CmarkParser::new_with_broken_link_callback(
            markdown,
            CmarkOptions::ENABLE_FOOTNOTES
                | CmarkOptions::ENABLE_TABLES
                | CmarkOptions::ENABLE_STRIKETHROUGH
                | CmarkOptions::ENABLE_TASKLISTS,
            Some(&broken_link_callback),
        ).into_offset_iter();
        Frontend {
            cfg,
            diagnostics,
            parser: Concat::new(ConvertCow(parser)).peekable(),
            buffer: VecDeque::new(),
            marker: PhantomData,
        }
    }

    fn convert_event(&mut self, evt: CmarkEvent<'a>, range: Range<usize>) {
        match evt {
            CmarkEvent::Text(text) => self.buffer.push_back((Event::Text(text), range)),
            CmarkEvent::Html(html) => self.buffer.push_back((Event::Html(html), range)),
            CmarkEvent::InlineHtml(html) => self.convert_inline_html(html, range),
            CmarkEvent::FootnoteReference(label) => {
                self.buffer.push_back((Event::FootnoteReference(FootnoteReference { label }), range))
            },
            CmarkEvent::SoftBreak => self.buffer.push_back((Event::SoftBreak, range)),
            CmarkEvent::HardBreak => self.buffer.push_back((Event::HardBreak, range)),
            CmarkEvent::TaskListMarker(checked) => {
                self.buffer.push_back((Event::TaskListMarker(TaskListMarker { checked }), range))
            },

            // TODO: make this not duplicate
            CmarkEvent::Start(CmarkTag::Rule) => self.buffer.push_back((Event::Start(Tag::Rule), range)),
            CmarkEvent::End(CmarkTag::Rule) => self.buffer.push_back((Event::End(Tag::Rule), range)),
            CmarkEvent::Start(CmarkTag::BlockQuote) => {
                self.buffer.push_back((Event::Start(Tag::BlockQuote), range))
            },
            CmarkEvent::End(CmarkTag::BlockQuote) => {
                self.buffer.push_back((Event::End(Tag::BlockQuote), range))
            },
            CmarkEvent::Start(CmarkTag::List(start_number)) if start_number.is_none() => {
                self.buffer.push_back((Event::Start(Tag::List), range))
            },
            CmarkEvent::End(CmarkTag::List(start_number)) if start_number.is_none() => {
                self.buffer.push_back((Event::End(Tag::List), range))
            },
            CmarkEvent::Start(CmarkTag::List(start_number)) => {
                self.buffer.push_back((Event::Start(Tag::Enumerate(Enumerate {
                    start_number: start_number.unwrap(),
                })), range))
            },
            CmarkEvent::End(CmarkTag::List(start_number)) => {
                self.buffer.push_back((Event::End(Tag::Enumerate(Enumerate {
                    start_number: start_number.unwrap(),
                })), range))
            },
            CmarkEvent::Start(CmarkTag::Item) => self.buffer.push_back((Event::Start(Tag::Item), range)),
            CmarkEvent::End(CmarkTag::Item) => self.buffer.push_back((Event::End(Tag::Item), range)),
            CmarkEvent::Start(CmarkTag::FootnoteDefinition(label)) => self
                .buffer
                .push_back((Event::Start(Tag::FootnoteDefinition(FootnoteDefinition { label })), range)),
            CmarkEvent::End(CmarkTag::FootnoteDefinition(label)) => self
                .buffer
                .push_back((Event::End(Tag::FootnoteDefinition(FootnoteDefinition { label })), range)),
            CmarkEvent::Start(CmarkTag::HtmlBlock) => {
                self.buffer.push_back((Event::Start(Tag::HtmlBlock), range))
            },
            CmarkEvent::End(CmarkTag::HtmlBlock) => {
                self.buffer.push_back((Event::End(Tag::HtmlBlock), range))
            },
            CmarkEvent::Start(CmarkTag::TableHead) => {
                self.buffer.push_back((Event::Start(Tag::TableHead), range))
            },
            CmarkEvent::End(CmarkTag::TableHead) => {
                self.buffer.push_back((Event::End(Tag::TableHead), range))
            },
            CmarkEvent::Start(CmarkTag::TableRow) => {
                self.buffer.push_back((Event::Start(Tag::TableRow), range))
            },
            CmarkEvent::End(CmarkTag::TableRow) => self.buffer.push_back((Event::End(Tag::TableRow), range)),
            CmarkEvent::Start(CmarkTag::TableCell) => {
                self.buffer.push_back((Event::Start(Tag::TableCell), range))
            },
            CmarkEvent::End(CmarkTag::TableCell) => {
                self.buffer.push_back((Event::End(Tag::TableCell), range))
            },
            CmarkEvent::Start(CmarkTag::Emphasis) => {
                self.buffer.push_back((Event::Start(Tag::InlineEmphasis), range))
            },
            CmarkEvent::End(CmarkTag::Emphasis) => {
                self.buffer.push_back((Event::End(Tag::InlineEmphasis), range))
            },
            CmarkEvent::Start(CmarkTag::Strong) => {
                self.buffer.push_back((Event::Start(Tag::InlineStrong), range))
            },
            CmarkEvent::End(CmarkTag::Strong) => {
                self.buffer.push_back((Event::End(Tag::InlineStrong), range))
            },
            CmarkEvent::Start(CmarkTag::Strikethrough) => {
                self.buffer.push_back((Event::Start(Tag::InlineStrikethrough), range))
            },
            CmarkEvent::End(CmarkTag::Strikethrough) => {
                self.buffer.push_back((Event::End(Tag::InlineStrikethrough), range))
            },

            CmarkEvent::Start(CmarkTag::Code) => self.convert_inline_code(range),
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => self.convert_code_block(lang, range, None),
            CmarkEvent::Start(CmarkTag::Paragraph) => self.convert_paragraph(range),
            CmarkEvent::Start(CmarkTag::Header(level)) => self.convert_header(level, range, None),
            CmarkEvent::Start(CmarkTag::Table(alignment)) => self.convert_table(alignment, range, None),
            CmarkEvent::Start(CmarkTag::Link(typ, dst, title)) => {
                self.convert_link(typ, dst, title, range)
            },
            CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => {
                self.convert_image(typ, dst, title, range, None)
            },

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

    fn convert_inline_html(&mut self, html: Cow<'a, str>, range: Range<usize>) {
        // TODO: proper HTML tag parsing
        match html.as_ref() {
            "<br>" | "<br/>" | "<br />" => self.buffer.push_back((Event::HardBreak, range)),
            _ => self.buffer.push_back((Event::InlineHtml(html), range)),
        }
    }

    /// Consumes and converts all elements until the next End event is received.
    /// Returns a concatenation of all text events (unrendered).
    #[inline]
    fn convert_until_end_inclusive(&mut self, f: impl Fn(&CmarkTag<'_>) -> bool) -> String {
        let mut text = String::new();
        loop {
            let (evt, range) = self.parser.next().unwrap();
            match &evt {
                CmarkEvent::End(tag) if f(tag) => return text,
                CmarkEvent::Text(t) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(t);
                },
                _ => (),
            }

            self.convert_event(evt, range);
        }
    }

    /// Consumes all elements until the End event is received.
    #[inline]
    fn consume_until_end_inclusive(&mut self) {
        let mut nest = 0;
        loop {
            match self.parser.next().unwrap().0 {
                CmarkEvent::Start(_) => nest += 1,
                CmarkEvent::End(_) if nest > 0 => nest -= 1,
                CmarkEvent::End(_) => return,
                _ => (),
            }
        }
    }

    /// Consumes all events until the End event, concatenating any text-like events.
    #[inline]
    fn concat_until_end_inclusive(&mut self) -> String {
        let mut s = String::with_capacity(100);
        let mut nest = 0;
        loop {
            match self.parser.next().unwrap().0 {
                CmarkEvent::Start(_) => nest += 1,
                CmarkEvent::End(_) if nest > 0 => nest -= 1,
                CmarkEvent::End(_) => break,
                CmarkEvent::Text(text) => s += &text,
                CmarkEvent::Html(_) => (),
                CmarkEvent::InlineHtml(html) => s += &html,
                CmarkEvent::SoftBreak | CmarkEvent::HardBreak => s += " ",
                CmarkEvent::FootnoteReference(_) => (),
                CmarkEvent::TaskListMarker(_) => (),
            }
        }
        s
    }

    fn convert_inline_code(&mut self, range: Range<usize>) {
        // check if code is math mode
        let (evt, _text_range) = self.parser.next().unwrap();
        let mut text = match evt {
            CmarkEvent::Text(text) => text,
            CmarkEvent::End(CmarkTag::Code) => {
                self.buffer.push_back((Event::Start(Tag::InlineCode), range.clone()));
                self.buffer.push_back((Event::End(Tag::InlineCode), range));
                return;
            },
            e => unreachable!(
                "InlineCode should always be followed by Text or End(Code) but was followed by \
                 {:?}",
                e
            ),
        };
        let tag = if text.chars().nth(1).map_or(false, char::is_whitespace) {
            match text.chars().next().unwrap() {
                '$' => {
                    // math
                    text.truncate_start(2);
                    Tag::InlineMath
                },
                '\\' => {
                    // latex
                    text.truncate_start(2);
                    match self.parser.next().unwrap().0 {
                        CmarkEvent::End(CmarkTag::Code) => (),
                        _ => unreachable!("InlineCode should only contain a single text event"),
                    }
                    self.buffer.push_back((Event::Latex(text), range));
                    return;
                },
                _ => Tag::InlineCode,
            }
        } else {
            Tag::InlineCode
        };
        self.buffer.push_back((Event::Start(tag.clone()), range.clone()));
        self.buffer.push_back((Event::Text(text), range.clone()));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Code = t { true } else { false });
        self.buffer.push_back((Event::End(tag), range));
    }

    fn convert_code_block(&mut self, lang: Cow<'a, str>, range: Range<usize>, mut cskvp: Option<(Cskvp<'a>, Range<usize>)>) {
        let lang = match lang {
            Cow::Borrowed(s) => s,
            Cow::Owned(_) => unreachable!("CodeBlock language should be borrowed"),
        };

        // check if language has label/config
        let language;
        if let Some(pos) = lang.find(',') {
            language = &lang[..pos];
            let code_block_cskvp_range = self.diagnostics.first_line(&range);
            if let Some((c, cskvp_range)) = &mut cskvp {
                // TODO: error
                self.diagnostics
                    .error("Code has both prefix and inline style labels / config")
                    .with_section(&code_block_cskvp_range, "config specified here")
                    .with_section(cskvp_range, "but config also specified here")
                    .emit();
                self.diagnostics.note("ignoring both").emit();
                c.clear();
            } else {
                let cskvp = Cskvp::new(Cow::Borrowed(&lang[pos + 1..]));
                // check for figure and handle it
                self.handle_cskvp(
                    cskvp,
                    code_block_cskvp_range,
                    CmarkEvent::Start(CmarkTag::CodeBlock(Cow::Borrowed(language))),
                    range,
                );
                return;
            }
        } else {
            language = lang;
        }

        let mut cskvp = cskvp.map(_0).unwrap_or_default();
        let tag = match language {
            "equation" | "$$" => {
                Tag::Equation(Equation { label: cskvp.take_label(), caption: cskvp.take_caption() })
            },
            "numberedequation" | "$$$" => Tag::NumberedEquation(Equation {
                label: cskvp.take_label(),
                caption: cskvp.take_caption(),
            }),
            "graphviz" => {
                let graphviz = Graphviz {
                    label: cskvp.take_label(),
                    caption: cskvp.take_caption(),
                    scale: cskvp.take_double("scale"),
                    width: cskvp.take_double("width"),
                    height: cskvp.take_double("height"),
                };
                Tag::Graphviz(graphviz)
            },
            "inlinelatex" => {
                // code is just a single block of text
                let (evt, latex_range) = self.parser.next().unwrap();
                let latex = match evt {
                    CmarkEvent::Text(text) => text,
                    _ => unreachable!(),
                };
                // consume end tag
                match self.parser.next().unwrap().0 {
                    CmarkEvent::End(CmarkTag::CodeBlock(_)) => (),
                    _ => unreachable!(),
                }

                self.buffer.push_back((Event::Latex(latex), latex_range));
                return;
            },
            _ => {
                Tag::CodeBlock(CodeBlock {
                    label: cskvp.take_label(),
                    caption: cskvp.take_caption(),
                    language: if language.is_empty() {
                        None
                    } else if language == "sequence" {
                        // TODO
                        println!("sequence is not yet implemented");
                        None
                    } else {
                        Some(Cow::Borrowed(language))
                    },
                })
            },
        };

        self.buffer.push_back((Event::Start(tag.clone()), range.clone()));
        self.convert_until_end_inclusive(
            |t| if let CmarkTag::CodeBlock(_) = t { true } else { false },
        );
        self.buffer.push_back((Event::End(tag), range));
    }

    fn convert_paragraph(&mut self, range: Range<usize>) {
        // check for label/config (Start(Paragraph), Text("{#foo,config...}"),
        // End(Paragraph)/SoftBreak)

        macro_rules! handle_normal {
            () => {{
                self.buffer.push_back((Event::Start(Tag::Paragraph), range.clone()));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Paragraph = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back((Event::End(Tag::Paragraph), range));
                return;
            }};
        };

        let text = match self.parser.peek().map(_0) {
            Some(CmarkEvent::Text(text)) => text, // continue
            _ => handle_normal!(),
        };

        let text = text.trim();
        if !(text.starts_with('{') && text.ends_with('}')) {
            handle_normal!();
        }
        // consume text
        let (evt, text_range) = self.parser.next().unwrap();
        let mut text = match evt {
            CmarkEvent::Text(text) => text,
            _ => unreachable!(),
        };

        // TODO: look ahead further to enable Start(Paragraph), Label, End(Paragraph),
        // Start(Paragraph), Image, …
        let end_paragraph = match self.parser.peek().map(_0) {
            Some(CmarkEvent::End(CmarkTag::Paragraph)) => true,
            Some(CmarkEvent::SoftBreak) => false,
            _ => {
                println!("not a label: {:?}", text);
                // not a label, reset our look-ahead and generate original
                self.buffer.push_back((Event::Start(Tag::Paragraph), range.clone()));
                self.buffer.push_back((Event::Text(text), text_range));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Paragraph = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back((Event::End(Tag::Paragraph), range));
                return;
            },
        };

        // consume end / soft break
        let _ = self.parser.next().unwrap();

        // if it's a label at the beginning of a paragraph, create that paragraph before creating
        // the element
        if !end_paragraph {
            self.buffer.push_back((Event::Start(Tag::Paragraph), range.clone()));
        }

        // parse label
        text.truncate_end(1);
        text.truncate_start(1);
        let mut cskvp = Cskvp::new(text);
        let cskvp_range = Range { start: text_range.start + 1, end: text_range.end - 1 };
        // if next element could have a label, convert that element with the label
        // otherwise create label event
        match self.parser.peek().map(_0) {
            Some(CmarkEvent::Start(CmarkTag::Header(_)))
            | Some(CmarkEvent::Start(CmarkTag::CodeBlock(_)))
            | Some(CmarkEvent::Start(CmarkTag::Table(_)))
            | Some(CmarkEvent::Start(CmarkTag::Image(..))) => {
                let (next_element, next_range) = self.parser.next().unwrap();
                self.handle_cskvp(cskvp, cskvp_range, next_element, next_range)
            },
            _ => {
                if !cskvp.has_label() {
                    // TODO error
                    println!(
                        "got element config, but there wasn't an element to apply it to: {:?}",
                        cskvp
                    );
                }
                self.buffer.push_back((Event::Label(cskvp.take_label().unwrap()), text_range));
            },
        }

        if !end_paragraph {
            self.convert_until_end_inclusive(|t| {
                if let CmarkTag::Paragraph = t {
                    true
                } else {
                    false
                }
            });
            self.buffer.push_back((Event::End(Tag::Paragraph), range));
        }
    }

    fn handle_cskvp(
        &mut self, mut cskvp: Cskvp<'a>, cskvp_range: Range<usize>,
        next_element: CmarkEvent<'a>, next_range: Range<usize>
    ) {
        // check if we want a figure
        let figure = match cskvp.take_figure().unwrap_or(self.cfg.figures) {
            false => None,
            true => match &next_element {
                CmarkEvent::Start(CmarkTag::Table(_)) => Some(Tag::TableFigure(Figure {
                    caption: cskvp.take_caption(),
                    label: cskvp.take_label(),
                })),
                _ => Some(Tag::Figure(Figure {
                    caption: cskvp.take_caption(),
                    label: cskvp.take_label(),
                })),
            },
        };
        if let Some(figure) = figure.clone() {
            self.buffer.push_back((Event::Start(figure), next_range.clone()));
        }

        match next_element {
            CmarkEvent::Start(CmarkTag::Header(label)) => {
                self.convert_header(label, next_range.clone(), Some((cskvp, cskvp_range)))
            },
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => {
                self.convert_code_block(lang, next_range.clone(), Some((cskvp, cskvp_range)))
            },
            CmarkEvent::Start(CmarkTag::Table(alignment)) => {
                self.convert_table(alignment, next_range.clone(), Some((cskvp, cskvp_range)))
            },
            CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => {
                self.convert_image(typ, dst, title, next_range.clone(), Some((cskvp, cskvp_range)))
            },
            element => panic!("handle_cskvp called with unknown element {:?}", element),
        }

        if let Some(figure) = figure {
            self.buffer.push_back((Event::End(figure), next_range));
        }
    }

    fn convert_header(&mut self, level: i32, range: Range<usize>, cskvp: Option<(Cskvp<'a>, Range<usize>)>) {
        let mut cskvp = cskvp.map(_0).unwrap_or_default();
        // header can have 3 different labels:
        // • `{#foo}\n\n# Header`: "prefix" style
        // • `# Header {#foo}: "inline" style
        // • `# Header`: "default" style, autogenerating label `header`
        // If both the first and the second are specified, we error.
        // If neither the first or the second are specified, we use the default one.
        // Otherwise we take the one that's specified.
        let prefix = cskvp.take_label();
        // Consume elements until end of heading to get its text.
        // Convert them and put them into the buffer because the're still needed.
        let current_index = self.buffer.len();
        let text = self.convert_until_end_inclusive(|t| {
            if let CmarkTag::Header(_) = t {
                true
            } else {
                false
            }
        });

        lazy_static! {
            // Matches `{#my-custom-inline-label}` returning `my-custom-inline-label`
            static ref RE: Regex = Regex::new(r"\{#([a-zA-Z0-9-_]+)\}\w*$").unwrap();
        }
        let inline = RE.captures(&text).map(|c| c.get(1).unwrap().as_str());

        let autogenerated = text
            .chars()
            .flat_map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => Some(c.to_ascii_lowercase()),
                ' ' => Some('-'),
                _ => None,
            })
            .collect();

        let label = if prefix.is_some() && inline.is_some() {
            // TODO: error
            println!("Header has both prefix and inline style labels, ignoring both");
            Cow::Owned(autogenerated)
        } else {
            prefix
                .or_else(|| inline.map(|inline| Cow::Owned(inline.to_string())))
                .unwrap_or_else(|| Cow::Owned(autogenerated))
        };

        let tag = Tag::Header(Header { label, level });
        self.buffer.insert(current_index, (Event::Start(tag.clone()), range.clone()));
        self.buffer.push_back((Event::End(tag), range));
    }

    fn convert_table(&mut self, alignment: Vec<Alignment>, range: Range<usize>, cskvp: Option<(Cskvp<'a>, Range<usize>)>) {
        let mut cskvp = cskvp.map(_0).unwrap_or_default();
        let tag = Tag::Table(Table {
            label: cskvp.take_label(),
            caption: cskvp.take_caption(),
            alignment,
        });
        self.buffer.push_back((Event::Start(tag.clone()), range.clone()));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Table(_) = t { true } else { false });
        self.buffer.push_back((Event::End(tag), range));
    }

    fn convert_link(&mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, range: Range<usize>) {
        let evt = match refs::parse_references(self.cfg, typ, dst, title) {
            ReferenceParseResult::BiberReferences(biber) => Event::BiberReferences(biber),
            ReferenceParseResult::InterLink(interlink) => Event::InterLink(interlink),
            ReferenceParseResult::Url(url) => Event::Url(url),
            ReferenceParseResult::InterLinkWithContent(interlink) => {
                Event::Start(Tag::InterLink(interlink))
            },
            ReferenceParseResult::UrlWithContent(url) => Event::Start(Tag::Url(url)),
            ReferenceParseResult::Command(command) => Event::Command(command),
            ReferenceParseResult::ResolveInclude(resolve_include) => {
                Event::ResolveInclude(resolve_include)
            },
            ReferenceParseResult::Text(text) => Event::Text(text),
        };
        match evt {
            Event::Start(tag) => {
                self.buffer.push_back((Event::Start(tag.clone()), range.clone()));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Link(..) = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back((Event::End(tag), range));
            },
            evt => {
                self.buffer.push_back((evt, range));
                self.consume_until_end_inclusive();
            },
        }
    }

    fn convert_image(
        &mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, range: Range<usize>,
        cskvp: Option<(Cskvp<'a>, Range<usize>)>,
    ) {
        // TODO: maybe not concat all text-like events but actually forward events
        // The CommonMark spec says that the parser should produce the correct events, while
        // the html renderer should only render it as text.
        let content = self.concat_until_end_inclusive();
        let alt_text = match typ {
            LinkType::Reference | LinkType::ReferenceUnknown | LinkType::Inline => {
                if content.is_empty() { None } else { Some(content) }
            },
            LinkType::Collapsed
            | LinkType::CollapsedUnknown
            | LinkType::Shortcut
            | LinkType::ShortcutUnknown => None,
            LinkType::Autolink | LinkType::Email => unreachable!("{:?} can be images???", typ),
        };
        let mut cskvp = cskvp.map(_0).unwrap_or_default();
        self.buffer.push_back((Event::Include(Include {
            label: cskvp.take_label(),
            caption: cskvp.take_caption(),
            title: if title.is_empty() { None } else { Some(title) },
            alt_text,
            dst,
            scale: cskvp.take_double("scale"),
            width: cskvp.take_double("width"),
            height: cskvp.take_double("height"),
        }), range))
    }
}

trait Get0 {
    type Output;
    fn get_0(self) -> Self::Output;
}

impl<A, B> Get0 for (A, B) {
    type Output = A;
    fn get_0(self) -> Self::Output {
        self.0
    }
}

impl<'a, A, B> Get0 for &'a (A, B) {
    type Output = &'a A;
    fn get_0(self) -> Self::Output {
        &self.0
    }
}

fn _0<T: Get0>(t: T) -> T::Output {
    t.get_0()
}
