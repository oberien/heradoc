use std::borrow::Cow;
use std::collections::VecDeque;
use std::str::FromStr;
use std::fs::File;
use std::io::Write;
use diagnostic::{Span, Spanned};

use lazy_static::lazy_static;
use pulldown_cmark::{BrokenLink, CowStr, Options as CmarkOptions, Parser as CmarkParser};
use regex::Regex;
use itertools::structs::MultiPeek;

mod concat;
mod convert_cow;
mod event;
mod refs;
mod size;
mod table_layout;

pub use self::event::*;
pub use self::size::*;
pub use self::refs::LinkType;

use self::concat::Concat;
use self::convert_cow::{ConvertCow, Event as CmarkEvent, Tag as CmarkTag};
use self::refs::ReferenceParseResult;
use crate::config::Config;
use crate::cskvp::Cskvp;
use crate::error::{DiagnosticCode, Diagnostics};
use crate::ext::{CowExt, StrExt};
use crate::resolve::{Command, ResolveSecurity};
use crate::util::ToUnix;

pub struct Frontend<'a> {
    cfg: &'a Config,
    diagnostics: &'a Diagnostics,
    parser: MultiPeek<Concat<'a>>,
    buffer: VecDeque<Spanned<Event<'a>>>,
    svgbob_index: u64,
}

impl<'a> Iterator for Frontend<'a> {
    type Item = Spanned<Event<'a>>;

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

fn broken_link_callback<'a>(broken_link: BrokenLink<'a>) -> Option<(CowStr<'a>, CowStr<'a>)> {
    let trimmed = broken_link.reference.trim();
    if trimmed.starts_with_ignore_ascii_case("include")
        || Command::from_str(trimmed).is_ok()
        || trimmed.starts_with('#')
        || trimmed.starts_with('@')
    {
        Some((broken_link.reference.clone(), broken_link.reference))
    } else {
        None
    }
}

impl<'a> Frontend<'a> {
    pub fn new(cfg: &'a Config, markdown: Spanned<&'a str>, diagnostics: &'a Diagnostics) -> Frontend<'a> {
        let parser = CmarkParser::new_with_broken_link_callback(
            &markdown.value[markdown.span.start..markdown.span.end],
            CmarkOptions::ENABLE_FOOTNOTES
                | CmarkOptions::ENABLE_TABLES
                | CmarkOptions::ENABLE_STRIKETHROUGH
                | CmarkOptions::ENABLE_TASKLISTS,
            Some(Box::leak(Box::new(broken_link_callback))),
        )
        .into_offset_iter();
        Frontend {
            cfg,
            diagnostics,
            parser: itertools::multipeek(Concat::new(ConvertCow(markdown.span, parser))),
            buffer: VecDeque::new(),
            svgbob_index: 0,
        }
    }

    fn convert_event(&mut self, evt: Spanned<CmarkEvent<'a>>) {
        let span = evt.span;
        let evt = evt.map(|evt| {
            match evt {
                CmarkEvent::Text(text) => Some(Event::Text(text)),
                CmarkEvent::Code(cow) => {
                    self.convert_inline_code(Spanned::new(cow, span));
                    None
                },
                CmarkEvent::Html(html) => Some(self.convert_html(html)),
                CmarkEvent::FootnoteReference(label) => {
                    Some(Event::FootnoteReference(FootnoteReference { label }))
                },
                CmarkEvent::SoftBreak => Some(Event::SoftBreak),
                CmarkEvent::HardBreak => Some(Event::HardBreak),
                CmarkEvent::Rule => Some(Event::Rule),
                CmarkEvent::TaskListMarker(checked) => {
                    Some(Event::TaskListMarker(TaskListMarker { checked }))
                },

                // TODO: make this not duplicate
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

                CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => {
                    self.convert_code_block(Spanned::new(lang, span), None);
                    None
                },
                CmarkEvent::Start(CmarkTag::Paragraph) => {
                    self.convert_paragraph(Spanned::new((), span));
                    None
                },
                CmarkEvent::Start(CmarkTag::Header(level)) => {
                    self.convert_header(Spanned::new(level, span), None);
                    None
                },
                CmarkEvent::Start(CmarkTag::Table(alignment)) => {
                    self.convert_table(Spanned::new(alignment, span), None);
                    None
                },
                CmarkEvent::Start(CmarkTag::Link(typ, dst, title)) => {
                    self.convert_link(typ, dst, title, span);
                    None
                },
                CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => {
                    self.convert_image(typ, dst, title, span, None);
                    None
                },

                CmarkEvent::End(CmarkTag::CodeBlock(_))
                | CmarkEvent::End(CmarkTag::Paragraph)
                | CmarkEvent::End(CmarkTag::Header(_))
                | CmarkEvent::End(CmarkTag::Table(_))
                | CmarkEvent::End(CmarkTag::Link(..))
                | CmarkEvent::End(CmarkTag::Image(..)) => {
                    panic!("End tag should be consumed when handling the start tag: {:?}", evt)
                },
            }
        });

        if let Spanned { value: Some(evt), span } = evt {
            self.buffer.push_back(Spanned::new(evt, span));
        }
    }

    fn convert_html(&mut self, html: Cow<'a, str>) -> Event<'a> {
        // TODO: proper HTML tag parsing
        match html.as_ref() {
            "<br>" | "<br/>" | "<br />" => Event::HardBreak,
            _ => Event::Html(html),
        }
    }

    /// Consumes and converts all elements until the next End event is received.
    /// Returns a concatenation of all text events (unrendered).
    #[inline]
    fn convert_until_end_inclusive(
        &mut self, f: impl Fn(&CmarkTag<'_>) -> bool,
    ) -> (String, Option<Span>) {
        let mut text = String::new();
        let mut span: Option<Span> = None;
        loop {
            let Spanned { value: evt, span: evt_span } = self.parser.next().unwrap();
            if let Some(span) = &mut span {
                span.end = evt_span.end;
            } else {
                span = Some(evt_span);
            }
            match &evt {
                CmarkEvent::End(ref tag) if f(tag) => return (text, span),
                CmarkEvent::Text(t) => {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(t);
                },
                _ => (),
            }

            self.convert_event(Spanned::new(evt, evt_span));
        }
    }

    /// Consumes all elements until the End event is received.
    #[inline]
    fn consume_until_end_inclusive(&mut self) {
        let mut nest = 0;
        loop {
            match self.parser.next().unwrap().value {
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
            match &el.value {
                CmarkEvent::Start(_) => nest += 1,
                CmarkEvent::End(_) if nest > 0 => nest -= 1,
                CmarkEvent::End(_) => break,
                CmarkEvent::Text(text) => s += &text,
                CmarkEvent::Code(text) => { s += "`"; s += &text; s += "`" },
                CmarkEvent::Html(html) => s += &html,
                CmarkEvent::SoftBreak | CmarkEvent::HardBreak => s += " ",
                CmarkEvent::Rule => (),
                CmarkEvent::FootnoteReference(_) => (),
                CmarkEvent::TaskListMarker(_) => (),
            }
        }
        s
    }

    fn convert_inline_code(&mut self, Spanned { value: mut text, span }: Spanned<Cow<'a, str>>) {
        // check if code is math mode
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
                    self.buffer.push_back(Spanned { value: Event::Latex(text), span });
                    return;
                },
                _ => Tag::InlineCode,
            }
        } else {
            Tag::InlineCode
        };
        self.buffer.push_back(Spanned { value: Event::Start(tag.clone()), span });
        self.buffer.push_back(Spanned { value: Event::Text(text), span });
        self.buffer.push_back(Spanned { value: Event::End(tag), span });
    }

    fn convert_code_block(
        &mut self, Spanned { value: lang, span }: Spanned<Cow<'a, str>>, mut cskvp: Option<Cskvp<'a>>,
    ) {
        let code_block_cskvp_span = self.diagnostics.first_line(span);
        let code_block_cskvp_span = Span {
            file: span.file,
            start: code_block_cskvp_span.end - lang.len(),
            end: code_block_cskvp_span.end,
        };

        // check if language has label/config
        let language;
        let language_span;
        if let Some(pos) = lang.find(',') {
            let (l, mut rest) = lang.split_at(pos);
            // get rid of comma
            rest.truncate_start(1);
            language = l;
            language_span = code_block_cskvp_span.with_len(language.len());
            let code_block_cskvp_content_span = Span {
                file: code_block_cskvp_span.file,
                start: code_block_cskvp_span.start + pos + 1,
                end: code_block_cskvp_span.end,
            };
            let inline_cskvp = Cskvp::new(
                rest,
                code_block_cskvp_span,
                code_block_cskvp_content_span,
                self.diagnostics,
            );
            if let Some(c) = &mut cskvp {
                self.diagnostics
                    .error(DiagnosticCode::CodeHasMultipleConfigs)
                    .with_info_label(c.span(), "prefix config defined here")
                    .with_info_label(code_block_cskvp_span, "inline config defined here")
                    .with_note("ignoring both")
                    .with_note("try removing one of them")
                    .emit();
                c.clear();
            } else {
                // check for figure and handle it
                self.handle_cskvp(
                    inline_cskvp,
                    CmarkEvent::Start(CmarkTag::CodeBlock(language)),
                    span,
                );
                return;
            }
        } else {
            language = lang;
            language_span = code_block_cskvp_span;
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
                let Spanned { value: evt, span: latex_span } = self.parser.next().unwrap();
                let latex = match evt {
                    CmarkEvent::Text(text) => text,
                    _ => unreachable!(),
                };
                // consume end tag
                match self.parser.next().unwrap().value {
                    CmarkEvent::End(CmarkTag::CodeBlock(_)) => (),
                    _ => unreachable!(),
                }

                self.buffer.push_back(Spanned::new(Event::Latex(latex), latex_span));
                return;
            },
            "svgbob" => {
                let Spanned { value: evt, span: svgbob_span } = self.parser.next().unwrap();
                let content = match evt {
                    CmarkEvent::Text(content) => content,
                    _ => unreachable!(),
                };
                // consume end tag
                match self.parser.next().unwrap().value {
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

                self.buffer.push_back(Spanned::new(Event::Include(Include {
                    resolve_security: ResolveSecurity::SkipChecks,
                    label: cskvp.take_label(),
                    caption: cskvp.take_caption(),
                    title: cskvp.take_double("title").map(|Spanned { value: title, .. }| title),
                    alt_text: cskvp.take_double("alt_text").map(|Spanned { value: title, .. }| title.into()),
                    dst: format!("file://{}", path.to_unix()
                        .expect(&format!("non-utf8 path: {:?}", path))).into(),
                    scale: cskvp.take_double("scale"),
                    width: cskvp.take_double("width"),
                    height: cskvp.take_double("height"),
                }), svgbob_span));
                return;
            },
            _ => Tag::CodeBlock(CodeBlock {
                label: cskvp.take_label(),
                caption: cskvp.take_caption(),
                language: if language.is_empty() {
                    None
                } else if language == "sequence" {
                    self.diagnostics
                        .warning(DiagnosticCode::NotYetImplemented)
                        .with_error_label(span, "sequence is not yet implemented")
                        .emit();
                    None
                } else {
                    Some(Spanned::new(language, language_span))
                },
            }),
        };

        self.buffer.push_back(Spanned::new(Event::Start(tag.clone()), span));
        self.convert_until_end_inclusive(
            |t| if let CmarkTag::CodeBlock(_) = t { true } else { false },
        );
        self.buffer.push_back(Spanned::new(Event::End(tag), span));
    }

    fn convert_paragraph(&mut self, Spanned { value: (), span }: Spanned<()>) {
        if self.check_convert_pagebreak(span) {
            return;
        }
        // check for label/config (Start(Paragraph), Text("{#foo,config...}"), End(Paragraph)/SoftBreak)

        macro_rules! handle_normal {
            () => {{
                self.buffer.push_back(Spanned::new(Event::Start(Tag::Paragraph), span));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Paragraph = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back(Spanned::new(Event::End(Tag::Paragraph), span));
                return;
            }};
        }

        let text = match self.parser.peek().map(|t| t.as_ref().value) {
            Some(CmarkEvent::Text(text)) => text, // continue
            _ => handle_normal!(),
        };

        let trimmed = text.trim();
        if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
            handle_normal!();
        }
        // consume text
        let Spanned { value: evt, span: text_span } = self.parser.next().unwrap();
        let mut text = match evt {
            CmarkEvent::Text(text) => text,
            _ => unreachable!(),
        };

        // TODO: look ahead further to enable Start(Paragraph), Label, End(Paragraph),
        // Start(Paragraph), Image, …
        let end_paragraph = match self.parser.peek().map(|t| t.as_ref().value) {
            Some(CmarkEvent::End(CmarkTag::Paragraph)) => true,
            Some(CmarkEvent::SoftBreak) => false,
            _ => {
                // not a label, reset our look-ahead and generate original
                self.buffer.push_back(Spanned::new(Event::Start(Tag::Paragraph), span));
                self.buffer.push_back(Spanned::new(Event::Text(text), text_span));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Paragraph = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back(Spanned::new(Event::End(Tag::Paragraph), span));
                return;
            },
        };

        // consume end / soft break
        let _ = self.parser.next().unwrap();

        // if it's a label at the beginning of a paragraph, create that paragraph before creating
        // the element
        if !end_paragraph {
            self.buffer.push_back(Spanned::new(Event::Start(Tag::Paragraph), span));
        }

        // parse label
        text.truncate_end(1);
        text.truncate_start(1);
        let cskvp_content_span = Span { file: text_span.file, start: text_span.start + 1, end: text_span.end - 1 };
        let mut cskvp =
            Cskvp::new(text, text_span, cskvp_content_span, self.diagnostics.clone());
        // if next element could have a label, convert that element with the label
        // otherwise create label event
        match self.parser.peek().unwrap() {
            Spanned { value: CmarkEvent::Start(CmarkTag::Header(_)), .. }
            | Spanned { value: CmarkEvent::Start(CmarkTag::CodeBlock(_)), .. }
            | Spanned { value: CmarkEvent::Start(CmarkTag::Table(_)), .. }
            | Spanned { value: CmarkEvent::Start(CmarkTag::Image(..)), .. } => {
                let Spanned { value: next_element, span: next_span } = self.parser.next().unwrap();
                self.handle_cskvp(cskvp, next_element, next_span)
            },
            Spanned { value: evt, span: next_span } => {
                if !cskvp.has_label() {
                    self.diagnostics
                        .error(DiagnosticCode::UnapplicableElementConfig)
                        .with_error_label(cskvp.span(), "found element config here")
                        .with_info_label(*next_span, "but it can't be applied to this element")
                        .with_note(format!("that element is of type {:?}, which doesn't support configs", evt))
                        .emit();
                } else {
                    self.buffer
                        .push_back(Spanned::new(Event::Label(cskvp.take_label().unwrap().value), text_span));
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
            self.buffer.push_back(Spanned::new(Event::End(Tag::Paragraph), span));
        }
    }

    /// Check for pagebreak (Start(Paragraph), Text("[=\s*]{3,}"), End(Paragraph))
    fn check_convert_pagebreak(&mut self, par_span: Span) -> bool {
        let (text, _) = match self.parser.peek().unwrap() {
            Spanned { value: CmarkEvent::Text(text), span } => (text, span),
            _ => { self.parser.reset_peek(); return false },
        };
        if !text.starts_with('=') || !RE.is_match(text) {
            self.parser.reset_peek();
            return false;
        }

        match self.parser.peek().unwrap() {
            Spanned { value: CmarkEvent::End(CmarkTag::Paragraph), .. } => (),
            _ => { self.parser.reset_peek(); return false },
        }
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[=\s*]{3,}").unwrap();
        }
        // consume Text
        let _ = self.parser.next().unwrap();
        // consume End(Paragraph)
        let _ = self.parser.next().unwrap();
        self.buffer.push_back(Spanned::new(Event::PageBreak, par_span));
        return true;
    }

    fn handle_cskvp(
        &mut self, mut cskvp: Cskvp<'a>, next_element: CmarkEvent<'a>, next_span: Span,
    ) {
        // check if we want a figure
        let figure = match cskvp.take_figure().map(|f| f.value).unwrap_or(self.cfg.figures) {
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
            self.buffer.push_back(Spanned::new(Event::Start(figure), next_span));
        }

        match next_element {
            CmarkEvent::Start(CmarkTag::Header(label)) => {
                self.convert_header(Spanned::new(label, next_span), Some(cskvp))
            },
            CmarkEvent::Start(CmarkTag::CodeBlock(lang)) => {
                self.convert_code_block(Spanned::new(lang, next_span), Some(cskvp))
            },
            CmarkEvent::Start(CmarkTag::Table(alignment)) => {
                self.convert_table(Spanned::new(alignment, next_span), Some(cskvp))
            },
            CmarkEvent::Start(CmarkTag::Image(typ, dst, title)) => {
                self.convert_image(typ, dst, title, next_span, Some(cskvp))
            },
            element => panic!("handle_cskvp called with unknown element {:?}", element),
        }

        if let Some(figure) = figure {
            self.buffer.push_back(Spanned::new(Event::End(figure), next_span));
        }
    }

    fn convert_header(&mut self, Spanned { value: level, span }: Spanned<i32>, cskvp: Option<Cskvp<'a>>) {
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
        let (text, text_span) = self.convert_until_end_inclusive(|t| {
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
        let inline_span = group0.map(|group| Span {
            file: text_span.unwrap().file,
            start: text_span.unwrap().start + group.start(),
            end: text_span.unwrap().start + group.end(),
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
                .error(DiagnosticCode::MultipleLabels)
                .with_error_label(span, "this header has both prefix and inline style labels")
                .with_info_label(prefix.unwrap().span, "prefix style defined here")
                .with_info_label(inline_span.unwrap(), "inline style defined here")
                .with_note(format!("using the inline one"))
                .with_note("try removing one of them")
                .emit();

            Spanned::new(Cow::Owned(inline.unwrap().to_string()), inline_span.unwrap())
        } else {
            prefix
                .or_else(|| {
                    inline.map(|inline| Spanned::new(Cow::Owned(inline.to_string()), inline_span.unwrap()))
                })
                .unwrap_or_else(|| Spanned::new(Cow::Owned(autogenerated), span))
        };

        let tag = Tag::Header(Header { label, level });
        self.buffer.insert(current_index, Spanned::new(Event::Start(tag.clone()), span));
        // if there was an inline-label, remove the label from the converted output
        if let Some(_) = inline_span {
            let Spanned { value: evt, mut span } = self.buffer.pop_back().unwrap();
            let mut last_text = match evt {
                Event::Text(text) => text,
                _ => unreachable!("last element of inline-tagged section isn't text: {:?}", evt),
            };
            assert!(last_text.len() >= inline.unwrap().len() + 2);
            let start = RE.find(&last_text).unwrap().start();
            span.end -= start;
            last_text.truncate_end(last_text.len() - start);
            self.buffer.push_back(Spanned::new(Event::Text(last_text), span));
        }
        self.buffer.push_back(Spanned::new(Event::End(tag), span));
    }

    fn convert_table(
        &mut self, Spanned { value: alignment, span }: Spanned<Vec<Alignment>>, cskvp: Option<Cskvp<'a>>,
    ) {
        let mut cskvp = cskvp.unwrap_or_default();

        let mut column_lines = vec![Vec::new(); alignment.len()];
        loop {
            match &self.parser.peek().unwrap().value {
                &CmarkEvent::Start(CmarkTag::TableHead) | &CmarkEvent::Start(CmarkTag::TableRow) => {
                    let mut i = 0;
                    loop {
                        let cell_text = match &self.parser.peek().unwrap().value {
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
        self.buffer.push_back(Spanned::new(Event::Start(tag.clone()), span));
        self.convert_until_end_inclusive(|t| if let CmarkTag::Table(_) = t { true } else { false });
        self.buffer.push_back(Spanned::new(Event::End(tag), span));
    }

    fn convert_link(
        &mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, span: Span,
    ) {
        let evt = match refs::parse_references(
            self.cfg,
            typ,
            dst,
            title,
            span,
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
                self.buffer.push_back(Spanned::new(Event::Start(tag.clone()), span));
                self.convert_until_end_inclusive(|t| {
                    if let CmarkTag::Link(..) = t {
                        true
                    } else {
                        false
                    }
                });
                self.buffer.push_back(Spanned::new(Event::End(tag), span));
            },
            evt => {
                self.buffer.push_back(Spanned::new(evt, span));
                self.consume_until_end_inclusive();
            },
        }
    }

    fn convert_image(
        &mut self, typ: LinkType, dst: Cow<'a, str>, title: Cow<'a, str>, span: Span,
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
        self.buffer.push_back(Spanned::new(
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
            span,
        ))
    }
}
