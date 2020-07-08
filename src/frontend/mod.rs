use std::borrow::Cow;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use std::fs::File;
use std::io::Write;

use lazy_static::lazy_static;
use pulldown_cmark::{Options as CmarkOptions, Parser as CmarkParser};
use regex::Regex;
use itertools::structs::MultiPeek;

mod concat;
mod convert_cow;
mod event;
pub mod range;
mod refs;
mod size;
mod table_layout;

pub use self::event::*;
pub use self::size::*;
pub use self::refs::LinkType;

use self::concat::Concat;
use self::convert_cow::{ConvertCow, Event as CmarkEvent, Tag as CmarkTag};
use self::range::{WithRange, SourceRange};
use self::refs::ReferenceParseResult;
use crate::config::Config;
use crate::cskvp::Cskvp;
use crate::diagnostics::Diagnostics;
use crate::ext::{CowExt, StrExt};
use crate::resolve::{Command, ResolveSecurity};
use crate::util::ToUnix;

pub struct Frontend<'a> {
    cfg: &'a Config,
    diagnostics: Arc<Diagnostics<'a>>,
    parser: MultiPeek<Concat<'a>>,
    buffer: VecDeque<WithRange<Event<'a>>>,
    svgbob_index: u64,
}

impl<'a> Iterator for Frontend<'a> {
    type Item = WithRange<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(evt) = self.buffer.pop_front() {
                return Some(evt);
            }

            let evt = self.parser.next()?;
            self.convert_event(evt);
        }
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

impl<'a> Frontend<'a> {
    pub fn new(cfg: &'a Config, markdown: &'a str, diagnostics: Arc<Diagnostics<'a>>) -> Frontend<'a> {
        let parser = CmarkParser::new_with_broken_link_callback(
            markdown,
            CmarkOptions::ENABLE_FOOTNOTES
                | CmarkOptions::ENABLE_TABLES
                | CmarkOptions::ENABLE_STRIKETHROUGH
                | CmarkOptions::ENABLE_TASKLISTS,
            Some(&broken_link_callback),
        )
        .into_offset_iter();
        Frontend {
            cfg,
            diagnostics,
            parser: itertools::multipeek(Concat::new(ConvertCow(parser))),
            buffer: VecDeque::new(),
            svgbob_index: 0,
        }
    }

    fn convert_event(&mut self, evt: WithRange<CmarkEvent<'a>>) {
        let range = evt.range();
        let evt = evt.map(|evt| {
            match evt {
                CmarkEvent::Text(text) => Some(Event::Text(text)),
                CmarkEvent::Html(html) => Some(Event::Html(html)),
                CmarkEvent::FootnoteReference(label) => {
                    Some(Event::FootnoteReference(FootnoteReference { label }))
                },
                CmarkEvent::SoftBreak => Some(Event::SoftBreak),
                CmarkEvent::HardBreak => Some(Event::HardBreak),
                CmarkEvent::TaskListMarker(checked) => {
                    Some(Event::TaskListMarker(TaskListMarker { checked }))
                },

                // TODO: make this not duplicate
                CmarkEvent::Start(CmarkTag::Rule) => Some(Event::Start(Tag::Rule)),
                CmarkEvent::End(CmarkTag::Rule) => Some(Event::End(Tag::Rule)),
                CmarkEvent::Start(CmarkTag::BlockQuote) => Some(Event::Start(Tag::BlockQuote)),
                CmarkEvent::End(CmarkTag::BlockQuote) => Some(Event::End(Tag::BlockQuote)),
                CmarkEvent::Start(CmarkTag::List(start_number)) if start_number.is_none() => {
                    Some(Event::Start(Tag::List))
                },
                CmarkEvent::End(CmarkTag::List(start_number)) if start_number.is_none() => {
                    Some(Event::End(Tag::List))
                },
                CmarkEvent::Start(CmarkTag::List(start_number)) => {
                    Some(Event::Start(Tag::Enumerate(Enumerate { start_number: start_number.unwrap() })))
                },
                CmarkEvent::End(CmarkTag::List(start_number)) => {
                    Some(Event::End(Tag::Enumerate(Enumerate { start_number: start_number.unwrap() })))
                },
                CmarkEvent::Start(CmarkTag::Item) => Some(Event::Start(Tag::Item)),
                CmarkEvent::End(CmarkTag::Item) => Some(Event::End(Tag::Item)),
                CmarkEvent::Start(CmarkTag::FootnoteDefinition(label)) => {
                    Some(Event::Start(Tag::FootnoteDefinition(FootnoteDefinition { label })))
                },
                CmarkEvent::End(CmarkTag::FootnoteDefinition(label)) => {
                    Some(Event::End(Tag::FootnoteDefinition(FootnoteDefinition { label })))
                },
                CmarkEvent::Start(CmarkTag::HtmlBlock) => Some(Event::Start(Tag::HtmlBlock)),
                CmarkEvent::End(CmarkTag::HtmlBlock) => Some(Event::End(Tag::HtmlBlock)),
                CmarkEvent::Start(CmarkTag::TableHead) => Some(Event::Start(Tag::TableHead)),
                CmarkEvent::End(CmarkTag::TableHead) => Some(Event::End(Tag::TableHead)),
                CmarkEvent::Start(CmarkTag::TableRow) => Some(Event::Start(Tag::TableRow)),
                CmarkEvent::End(CmarkTag::TableRow) => Some(Event::End(Tag::TableRow)),
                CmarkEvent::Start(CmarkTag::TableCell) => Some(Event::Start(Tag::TableCell)),
                CmarkEvent::End(CmarkTag::TableCell) => Some(Event::End(Tag::TableCell)),
                CmarkEvent::Start(CmarkTag::Emphasis) => Some(Event::Start(Tag::InlineEmphasis)),
                CmarkEvent::End(CmarkTag::Emphasis) => Some(Event::End(Tag::InlineEmphasis)),
                CmarkEvent::Start(CmarkTag::Strong) => Some(Event::Start(Tag::InlineStrong)),
                CmarkEvent::End(CmarkTag::Strong) => Some(Event::End(Tag::InlineStrong)),
                CmarkEvent::Start(CmarkTag::Strikethrough) => Some(Event::Start(Tag::InlineStrikethrough)),
                CmarkEvent::End(CmarkTag::Strikethrough) => Some(Event::End(Tag::InlineStrikethrough)),

                CmarkEvent::InlineHtml(html) => {
                    self.convert_inline_html(WithRange(html, range));
                    None
                },
                CmarkEvent::Start(CmarkTag::Code) => {
                    self.convert_inline_code(WithRange((), range));
                    None
                },
                CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => {
                    self.convert_code_block(WithRange(lang, range), None);
                    None
                },
                CmarkEvent::Start(CmarkTag::Paragraph) => {
                    self.convert_paragraph(WithRange((), range));
                    None
                },
                CmarkEvent::Start(CmarkTag::Header(level)) => {
                    self.convert_header(WithRange(level, range), None);
                    None
                },
                CmarkEvent::Start(CmarkTag::Table(alignment)) => {
                    self.convert_table(WithRange(alignment, range), None);
                    None
                },
                CmarkEvent::Start(CmarkTag::Link(typ, dst, title)) => {
                    self.convert_link(typ, dst, title, range);
                    None
                },
                CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => {
                    self.convert_image(typ, dst, title, range, None);
                    None
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
        });

        if let WithRange(Some(evt), range) = evt {
            self.buffer.push_back(WithRange(evt, range));
        }
    }

    fn convert_inline_html(&mut self, html: WithRange<Cow<'a, str>>) {
        let evt = html.map(|html| {
            // TODO: proper HTML tag parsing
            match html.as_ref() {
                "<br>" | "<br/>" | "<br />" => Event::HardBreak,
                _ => Event::InlineHtml(html),
            }
        });
        self.buffer.push_back(evt);
    }

    /// Consumes and converts all elements until the next End event is received.
    /// Returns a concatenation of all text events (unrendered).
    #[inline]
    fn convert_until_end_inclusive(
        &mut self, f: impl Fn(&CmarkTag<'_>) -> bool,
    ) -> (String, Option<SourceRange>) {
        let mut text = String::new();
        let mut range: Option<SourceRange> = None;
        loop {
            let WithRange(evt, evt_range) = self.parser.next().unwrap();
            if let Some(range) = &mut range {
                range.end = evt_range.end;
            } else {
                range = Some(evt_range);
            }
            match &evt {
                CmarkEvent::End(ref tag) if f(tag) => return (text, range),
                CmarkEvent::Text(t) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(t);
                },
                _ => (),
            }

            self.convert_event(WithRange(evt, evt_range));
        }
    }

    /// Consumes all elements until the End event is received.
    #[inline]
    fn consume_until_end_inclusive(&mut self) {
        let mut nest = 0;
        loop {
            match self.parser.next().unwrap().element() {
                CmarkEvent::Start(_) => nest += 1,
                CmarkEvent::End(_) if nest > 0 => nest -= 1,
                CmarkEvent::End(_) => return,
                _ => (),
            }
        }
    }

    /// Consumes all events until the End event, concatenating any text-like events.
    /// Takes a function which generates the events to be consumed (allowing to pass a consuming,
    /// or a multi-peeking function).
    #[inline]
    fn concat_until_end_inclusive(&mut self, consume: bool) -> String {
        let mut s = String::with_capacity(100);
        let mut nest = 0;
        loop {
            let evt;
            let el = if consume {
                evt = self.parser.next();
                evt.as_ref().unwrap()
            } else {
                self.parser.peek().unwrap()
            };
            match el.element_ref() {
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

    fn convert_inline_code(&mut self, WithRange((), range): WithRange<()>) {
        // check if code is math mode
        let WithRange(evt, _text_range) = self.parser.next().unwrap();
        let mut text = match evt {
            CmarkEvent::Text(text) => text,
            CmarkEvent::End(CmarkTag::Code) => {
                self.buffer.push_back(WithRange(Event::Start(Tag::InlineCode), range));
                self.buffer.push_back(WithRange(Event::End(Tag::InlineCode), range));
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
                    self.buffer.push_back(WithRange(Event::Latex(text), range));
                    return;
                },
                _ => Tag::InlineCode,
            }
        } else {
            Tag::InlineCode
        };
        self.buffer.push_back(WithRange(Event::Start(tag.clone()), range));
        self.buffer.push_back(WithRange(Event::Text(text), range));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Code = t { true } else { false });
        self.buffer.push_back(WithRange(Event::End(tag), range));
    }

    fn convert_code_block(
        &mut self, WithRange(lang, range): WithRange<Cow<'a, str>>, mut cskvp: Option<Cskvp<'a>>,
    ) {
        let code_block_cskvp_range = self.diagnostics.first_line(range);
        let code_block_cskvp_range = SourceRange {
            start: code_block_cskvp_range.end - lang.len(),
            end: code_block_cskvp_range.end,
        };

        // check if language has label/config
        let language;
        let language_range;
        if let Some(pos) = lang.find(',') {
            let (l, mut rest) = lang.split_at(pos);
            // get rid of comma
            rest.truncate_start(1);
            language = l;
            language_range = SourceRange {
                start: code_block_cskvp_range.start,
                end: code_block_cskvp_range.start + language.len(),
            };
            let code_block_cskvp_content_range = SourceRange {
                start: code_block_cskvp_range.start + pos + 1,
                end: code_block_cskvp_range.end,
            };
            let inline_cskvp = Cskvp::new(
                rest,
                code_block_cskvp_range,
                code_block_cskvp_content_range,
                Arc::clone(&self.diagnostics),
            );
            if let Some(c) = &mut cskvp {
                self.diagnostics
                    .error("code has both prefix and inline style config")
                    .with_info_section(c.range(), "prefix config defined here")
                    .with_info_section(code_block_cskvp_range, "inline config defined here")
                    .note("ignoring both")
                    .help("try removing one of them")
                    .emit();
                c.clear();
            } else {
                // check for figure and handle it
                self.handle_cskvp(
                    inline_cskvp,
                    CmarkEvent::Start(CmarkTag::CodeBlock(language)),
                    range,
                );
                return;
            }
        } else {
            language = lang;
            language_range = code_block_cskvp_range;
        }

        let mut cskvp = cskvp.unwrap_or_default();
        let tag = match &*language {
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
                let WithRange(evt, latex_range) = self.parser.next().unwrap();
                let latex = match evt {
                    CmarkEvent::Text(text) => text,
                    _ => unreachable!(),
                };
                // consume end tag
                match self.parser.next().unwrap().0 {
                    CmarkEvent::End(CmarkTag::CodeBlock(_)) => (),
                    _ => unreachable!(),
                }

                self.buffer.push_back(WithRange(Event::Latex(latex), latex_range));
                return;
            },
            "svgbob" => {
                let WithRange(evt, svgbob_range) = self.parser.next().unwrap();
                let content = match evt {
                    CmarkEvent::Text(content) => content,
                    _ => unreachable!(),
                };
                // consume end tag
                match self.parser.next().unwrap().0 {
                    CmarkEvent::End(CmarkTag::CodeBlock(_)) => (),
                    _ => unreachable!(),
                }

                // render svg
                let filename = format!("svgbob{}.svg", self.svgbob_index);
                self.svgbob_index += 1;
                let path = self.cfg.temp_dir.join(filename);
                let mut file = File::create(&path).expect(&format!("can't create temporary svgbob file {:?}", path));
                let svg = svgbob::to_svg(&content);
                writeln!(file, "{}", svg).expect(&format!("can't write to temporary svgbob file {:?}", path));

                self.buffer.push_back(WithRange(Event::Include(Include {
                    resolve_security: ResolveSecurity::SkipChecks,
                    label: cskvp.take_label(),
                    caption: cskvp.take_caption(),
                    title: cskvp.take_double("title").map(|WithRange(title, _)| title),
                    alt_text: cskvp.take_double("alt_text").map(|WithRange(title, _)| title.into()),
                    dst: format!("file://{}", path.to_unix()
                        .expect(&format!("non-utf8 path: {:?}", path))).into(),
                    scale: cskvp.take_double("scale"),
                    width: cskvp.take_double("width"),
                    height: cskvp.take_double("height"),
                }), svgbob_range));
                return;
            },
            _ => Tag::CodeBlock(CodeBlock {
                label: cskvp.take_label(),
                caption: cskvp.take_caption(),
                language: if language.is_empty() {
                    None
                } else if language == "sequence" {
                    self.diagnostics
                        .warning("sequence is not yet implemented")
                        .with_error_section(range, "")
                        .emit();
                    None
                } else {
                    Some(WithRange(language, language_range))
                },
            }),
        };

        self.buffer.push_back(WithRange(Event::Start(tag.clone()), range));
        self.convert_until_end_inclusive(
            |t| if let CmarkTag::CodeBlock(_) = t { true } else { false },
        );
        self.buffer.push_back(WithRange(Event::End(tag), range));
    }

    fn convert_paragraph(&mut self, WithRange((), range): WithRange<()>) {
        if self.check_convert_pagebreak(range) {
            return;
        }
        // check for label/config (Start(Paragraph), Text("{#foo,config...}"), End(Paragraph)/SoftBreak)

        macro_rules! handle_normal {
            () => {{
                self.buffer.push_back(WithRange(Event::Start(Tag::Paragraph), range));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Paragraph = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back(WithRange(Event::End(Tag::Paragraph), range));
                return;
            }};
        };

        let text = match self.parser.peek().map(|t| t.as_ref().element()) {
            Some(CmarkEvent::Text(text)) => text, // continue
            _ => handle_normal!(),
        };

        let trimmed = text.trim();
        if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
            handle_normal!();
        }
        // consume text
        let WithRange(evt, text_range) = self.parser.next().unwrap();
        let mut text = match evt {
            CmarkEvent::Text(text) => text,
            _ => unreachable!(),
        };

        // TODO: look ahead further to enable Start(Paragraph), Label, End(Paragraph),
        // Start(Paragraph), Image, …
        let end_paragraph = match self.parser.peek().map(|t| t.as_ref().element()) {
            Some(CmarkEvent::End(CmarkTag::Paragraph)) => true,
            Some(CmarkEvent::SoftBreak) => false,
            _ => {
                // not a label, reset our look-ahead and generate original
                self.buffer.push_back(WithRange(Event::Start(Tag::Paragraph), range));
                self.buffer.push_back(WithRange(Event::Text(text), text_range));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Paragraph = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back(WithRange(Event::End(Tag::Paragraph), range));
                return;
            },
        };

        // consume end / soft break
        let _ = self.parser.next().unwrap();

        // if it's a label at the beginning of a paragraph, create that paragraph before creating
        // the element
        if !end_paragraph {
            self.buffer.push_back(WithRange(Event::Start(Tag::Paragraph), range));
        }

        // parse label
        text.truncate_end(1);
        text.truncate_start(1);
        let cskvp_content_range = SourceRange { start: text_range.start + 1, end: text_range.end - 1 };
        let mut cskvp =
            Cskvp::new(text, text_range, cskvp_content_range, self.diagnostics.clone());
        // if next element could have a label, convert that element with the label
        // otherwise create label event
        match self.parser.peek().unwrap() {
            WithRange(CmarkEvent::Start(CmarkTag::Header(_)), _)
            | WithRange(CmarkEvent::Start(CmarkTag::CodeBlock(_)), _)
            | WithRange(CmarkEvent::Start(CmarkTag::Table(_)), _)
            | WithRange(CmarkEvent::Start(CmarkTag::Image(..)), _) => {
                let WithRange(next_element, next_range) = self.parser.next().unwrap();
                self.handle_cskvp(cskvp, next_element, next_range)
            },
            &WithRange(_, next_range) => {
                if !cskvp.has_label() {
                    self.diagnostics
                        .error("found element config, but there wasn't an element ot apply it to")
                        .with_error_section(cskvp.range(), "found element config here")
                        .with_info_section(next_range, "but it can't be applied to this element")
                        .emit();
                } else {
                    self.buffer
                        .push_back(WithRange(Event::Label(cskvp.take_label().unwrap().element()), text_range));
                }
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
            self.buffer.push_back(WithRange(Event::End(Tag::Paragraph), range));
        }
    }

    /// Check for pagebreak (Start(Paragraph), Text("[=\s*]{3,}"), End(Paragraph))
    fn check_convert_pagebreak(&mut self, par_range: SourceRange) -> bool {
        let (text, _) = match self.parser.peek().unwrap() {
            WithRange(CmarkEvent::Text(text), range) => (text, range),
            _ => { self.parser.reset_peek(); return false },
        };
        if !text.starts_with('=') || !RE.is_match(text) {
            self.parser.reset_peek();
            return false;
        }

        match self.parser.peek().unwrap() {
            WithRange(CmarkEvent::End(CmarkTag::Paragraph), _) => (),
            _ => { self.parser.reset_peek(); return false },
        }
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[=\s*]{3,}").unwrap();
        }
        // consume Text
        let _ = self.parser.next().unwrap();
        // consume End(Paragraph)
        let _ = self.parser.next().unwrap();
        self.buffer.push_back(WithRange(Event::PageBreak, par_range));
        return true;
    }

    fn handle_cskvp(
        &mut self, mut cskvp: Cskvp<'a>, next_element: CmarkEvent<'a>, next_range: SourceRange,
    ) {
        // check if we want a figure
        let figure = match cskvp.take_figure().map(|f| f.element()).unwrap_or(self.cfg.figures) {
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
            self.buffer.push_back(WithRange(Event::Start(figure), next_range));
        }

        match next_element {
            CmarkEvent::Start(CmarkTag::Header(label)) => {
                self.convert_header(WithRange(label, next_range), Some(cskvp))
            },
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => {
                self.convert_code_block(WithRange(lang, next_range), Some(cskvp))
            },
            CmarkEvent::Start(CmarkTag::Table(alignment)) => {
                self.convert_table(WithRange(alignment, next_range), Some(cskvp))
            },
            CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => {
                self.convert_image(typ, dst, title, next_range, Some(cskvp))
            },
            element => panic!("handle_cskvp called with unknown element {:?}", element),
        }

        if let Some(figure) = figure {
            self.buffer.push_back(WithRange(Event::End(figure), next_range));
        }
    }

    fn convert_header(&mut self, WithRange(level, range): WithRange<i32>, cskvp: Option<Cskvp<'a>>) {
        let mut cskvp = cskvp.unwrap_or_default();
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
        let (text, text_range) = self.convert_until_end_inclusive(|t| {
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
        let captures = RE.captures(&text);
        let inline = captures.as_ref().map(|c| c.get(1).unwrap().as_str());
        let group0 = captures.as_ref().map(|c| c.get(0).unwrap());
        let inline_range = group0.map(|group| SourceRange {
            start: text_range.unwrap().start + group.start(),
            end: text_range.unwrap().start + group.end(),
        });

        let autogenerated = text
            .chars()
            .flat_map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => Some(c.to_ascii_lowercase()),
                ' ' => Some('-'),
                _ => None,
            })
            .collect();

        let label = if prefix.is_some() && inline.is_some() {
            self.diagnostics
                .error("header has both prefix and inline style labels")
                .with_error_section(range, "header defined here")
                .with_info_section(prefix.unwrap().range(), "prefix style defined here")
                .with_info_section(inline_range.unwrap(), "inline style defined here")
                .note(format!("using the inline one"))
                .help("try removing one of them")
                .emit();

            WithRange(Cow::Owned(inline.unwrap().to_string()), inline_range.unwrap())
        } else {
            prefix
                .or_else(|| {
                    inline.map(|inline| WithRange(Cow::Owned(inline.to_string()), inline_range.unwrap()))
                })
                .unwrap_or_else(|| WithRange(Cow::Owned(autogenerated), range))
        };

        let tag = Tag::Header(Header { label, level });
        self.buffer.insert(current_index, WithRange(Event::Start(tag.clone()), range));
        // if there was an inline-label, remove the label from the converted output
        if let Some(range) = inline_range {
            let WithRange(evt, mut range) = self.buffer.pop_back().unwrap();
            let mut last_text = match evt {
                Event::Text(text) => text,
                _ => unreachable!("last element of inline-tagged section isn't text: {:?}", evt),
            };
            assert!(last_text.len() >= inline.unwrap().len() + 2);
            let start = RE.find(&last_text).unwrap().start();
            range.end -= start;
            last_text.truncate_end(last_text.len() - start);
            self.buffer.push_back(WithRange(Event::Text(last_text), range));
        }
        self.buffer.push_back(WithRange(Event::End(tag), range));
    }

    fn convert_table(
        &mut self, WithRange(alignment, range): WithRange<Vec<Alignment>>, cskvp: Option<Cskvp<'a>>,
    ) {
        let mut cskvp = cskvp.unwrap_or_default();

        let mut column_lines = vec![Vec::new(); alignment.len()];
        loop {
            match &self.parser.peek().unwrap().0 {
                &CmarkEvent::Start(CmarkTag::TableHead) | &CmarkEvent::Start(CmarkTag::TableRow) => {
                    let mut i = 0;
                    loop {
                        let cell_text = match &self.parser.peek().unwrap().0 {
                            &CmarkEvent::Start(CmarkTag::TableCell) => self.concat_until_end_inclusive(false),
                            &CmarkEvent::End(CmarkTag::TableHead) | &CmarkEvent::End(CmarkTag::TableRow) => break,
                            e => unreachable!("We are parsing table cells, but got something other than a cell or row-end: {:?}", e),
                        };
                        column_lines[i].extend(cell_text.lines().map(str::to_owned));
                        i += 1;
                    }
                }
                &CmarkEvent::End(CmarkTag::TableHead) | &CmarkEvent::End(CmarkTag::TableRow) => (),
                &CmarkEvent::End(CmarkTag::Table(_)) => break,
                e => unreachable!("We are parsing a table, but got something other than a table-row or the table-end: {:?}", e),
            }
        }
        self.parser.reset_peek();

        let widths = table_layout::column_widths(column_lines);
        assert_eq!(widths.len(), alignment.len());

        let tag = Tag::Table(Table {
            label: cskvp.take_label(),
            caption: cskvp.take_caption(),
            columns: alignment.into_iter().zip(widths.into_iter().map(|w| ColumnWidthPercent(w))).collect(),
        });
        self.buffer.push_back(WithRange(Event::Start(tag.clone()), range));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Table(_) = t { true } else { false });
        self.buffer.push_back(WithRange(Event::End(tag), range));
    }

    fn convert_link(
        &mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, range: SourceRange,
    ) {
        let evt = match refs::parse_references(
            self.cfg,
            typ,
            dst,
            title,
            range,
            &mut self.diagnostics,
        ) {
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
                self.buffer.push_back(WithRange(Event::Start(tag.clone()), range));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Link(..) = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back(WithRange(Event::End(tag), range));
            },
            evt => {
                self.buffer.push_back(WithRange(evt, range));
                self.consume_until_end_inclusive();
            },
        }
    }

    fn convert_image(
        &mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, range: SourceRange,
        cskvp: Option<Cskvp<'a>>,
    ) {
        // TODO: maybe not concat all text-like events but actually forward events
        // The CommonMark spec says that the parser should produce the correct events, while
        // the html renderer should only render it as text.
        let content = self.concat_until_end_inclusive(true);
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
        let mut cskvp = cskvp.unwrap_or_default();
        self.buffer.push_back(WithRange(
            Event::Include(Include {
                resolve_security: ResolveSecurity::Default,
                label: cskvp.take_label(),
                caption: cskvp.take_caption(),
                title: if title.is_empty() { None } else { Some(title) },
                alt_text,
                dst,
                scale: cskvp.take_double("scale"),
                width: cskvp.take_double("width"),
                height: cskvp.take_double("height"),
            }),
            range,
        ))
    }
}
