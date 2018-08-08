use std::io::{Write, Result};
use std::iter::Peekable;

use pulldown_cmark::{Event, Tag, Alignment};

mod preamble;

pub fn generate<'a>(events: impl IntoIterator<Item = Event<'a>>, out: impl Write) -> Result<()> {
    Generator::new(events, out).gen()
}

pub struct Generator<'a, I: Iterator<Item = Event<'a>>, W: Write> {
    events: Peekable<I>,
    out: W,
    enumerate_depth: usize,
}

macro_rules! read_until {
    ($self:expr, $pat:pat) => {{
        let mut text = Vec::new();
        // taken from take_mut and modified to allow forwarding errors
        unsafe {
            let old_self = ::std::ptr::read($self);
            let (new_self, err) = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let Generator { events, out, enumerate_depth } = old_self;
                let mut gen = Generator {
                    events,
                    out: &mut text,
                    enumerate_depth,
                };
                let err = loop {
                    match gen.events.next().unwrap() {
                        Event::End($pat) => break None,
                        evt => if let Err(e) = gen.visit_event(evt) {
                            break Some(e)
                        }
                    }
                };
                (Generator {
                    events: gen.events,
                    out,
                    enumerate_depth: gen.enumerate_depth,
                }, err)
            })).unwrap_or_else(|_| ::std::process::abort());
            ::std::ptr::write($self, new_self);
            match err {
                Some(e) => Err(e),
                None => Ok(String::from_utf8(text).expect("invalid UTF-8")),
            }
        }
    }}
}

macro_rules! handle_until {
    ($self:expr, $pat:pat) => {
        loop {
            match $self.events.next().unwrap() {
                Event::End($pat) => break,
                evt => $self.visit_event(evt)?,
            }
        }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>, W: Write> Generator<'a, I, W> {
    pub fn new<II: IntoIterator<Item = Event<'a>, IntoIter = I>>(events: II, out: W) -> Generator<'a, I, W> {
        Generator {
            events: events.into_iter().peekable(),
            out,
            enumerate_depth: 0,
        }
    }

    pub fn gen(&mut self) -> Result<()> {
        // TODO: parse preamble first if it exists
        self.gen_preamble()?;
        loop {
            match self.events.next() {
                Some(evt) => self.visit_event(evt)?,
                None => break,
            }
        }
        self.gen_prologue()?;
        Ok(())
    }

    fn gen_preamble(&mut self) -> Result<()> {
        // TODO: papersize, documentclass, geometry
        // TODO: itemizespacing
        writeln!(self.out, "\\documentclass[a4paper]{{scrartcl}}")?;
        writeln!(self.out, "\\usepackage[utf8]{{inputenc}}")?;
        writeln!(self.out)?;
        // TODO: include rust highlighting
        // TODO: use minted instead of lstlistings?
        // TODO: lstset
        writeln!(self.out, "\\usepackage{{listings}}")?;
        writeln!(self.out, "\\usepackage[usenames, dvipsnames]{{color}}")?;
        writeln!(self.out, "\\usepackage{{xcolor}}")?;
        writeln!(self.out, "{}", preamble::lstset)?;
        writeln!(self.out, "{}", preamble::lstdefineasm)?;
        writeln!(self.out, "{}", preamble::lstdefinerust)?;
        // TODO: graphicspath
        writeln!(self.out, "\\usepackage{{graphicx}}")?;
        writeln!(self.out, "\\usepackage{{hyperref}}")?;
        // TODO: cleveref options
        writeln!(self.out, "\\usepackage{{cleveref}}")?;
        writeln!(self.out, "\\usepackage{{refcount}}")?;
        writeln!(self.out, "\\usepackage{{array}}")?;
        writeln!(self.out, "{}", preamble::thickhline)?;
        writeln!(self.out)?;
        writeln!(self.out, "{}", preamble::aquote)?;
        writeln!(self.out)?;
        writeln!(self.out, "\\begin{{document}}")?;
        writeln!(self.out)?;
        Ok(())
    }

    fn gen_prologue(&mut self) -> Result<()> {
        writeln!(self.out, "\\end{{document}}")?;
        Ok(())
    }

    fn visit_event(&mut self, event: Event<'a>) -> Result<()> {
        match event {
            // primitives
            Event::Text(text) => self.gen_text(&text),
            Event::Html(html) => unimplemented!(),
            Event::InlineHtml(html) => unimplemented!(),
            Event::FootnoteReference(fnote) => self.gen_footnote_reference(&fnote),
            Event::SoftBreak => self.gen_soft_break(),
            Event::HardBreak => self.gen_hard_break(),
            // complex
            Event::Start(Tag::Paragraph) => self.gen_par(),
            Event::Start(Tag::Rule) => self.gen_rule(),
            Event::Start(Tag::Header(level)) => self.gen_header(level),
            Event::Start(Tag::BlockQuote) => self.gen_block_quote(),
            Event::Start(Tag::CodeBlock(lang)) => self.gen_code_block(&lang),
            Event::Start(Tag::List(start)) => self.gen_list(start),
            Event::Start(Tag::Item) => self.gen_item(),
            Event::Start(Tag::FootnoteDefinition(fnote)) => self.gen_footnote_definition(&fnote),
            Event::Start(Tag::Table(align)) => self.gen_table(align),
            Event::Start(Tag::TableHead) => self.gen_table_head(),
            Event::Start(Tag::TableRow) => self.gen_table_row(),
            Event::Start(Tag::TableCell) => self.gen_table_cell(),
            Event::Start(Tag::Emphasis) => self.gen_emphasized(),
            Event::Start(Tag::Strong) => self.gen_strong(),
            Event::Start(Tag::Code) => self.gen_code(),
            Event::Start(Tag::Link(dst, title)) => self.gen_link(&dst, &title),
            Event::Start(Tag::Image(dst, title)) => self.gen_image(&dst, &title),
            Event::End(_) => unreachable!("end should be handled by gen_* functions"),
        }
    }

    fn gen_text(&mut self, text: &str) -> Result<()> {
        write!(self.out, "{}", text)
    }

    fn gen_footnote_reference(&mut self, fnote: &str) -> Result<()> {
        write!(self.out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote)
    }

    fn gen_soft_break(&mut self) -> Result<()> {
        // soft breaks are only used to split up text in lines in the source file
        // so it's nothing we should translate, but for better readability keep them
        writeln!(self.out)
    }

    fn gen_hard_break(&mut self) -> Result<()> {
        writeln!(self.out, "\\par")
    }

    fn gen_par(&mut self) -> Result<()> {
        handle_until!(self, Tag::Paragraph);
        // TODO: improve readability (e.g. no newline between list items
        match self.events.peek() {
            Some(Event::Text(_))
            | Some(Event::Html(_))
            | Some(Event::InlineHtml(_))
            | Some(Event::Start(Tag::Paragraph))
            // those shouldn't occur after a par, but better safe than sorry
            | Some(Event::Start(Tag::Emphasis))
            | Some(Event::Start(Tag::Strong))
            | Some(Event::Start(Tag::Code))
            | Some(Event::Start(Tag::Link(..)))
            | Some(Event::Start(Tag::Image(..))) => writeln!(self.out, "\\\\\n\\\\")?,
            _ => writeln!(self.out)?,
        }
        Ok(())
    }

    fn gen_rule(&mut self) -> Result<()> {
        // TODO: find out why text after the hrule is indented
        writeln!(self.out)?;
        writeln!(self.out, "\\vspace{{1em}}")?;
        writeln!(self.out, "\\hrule")?;
        writeln!(self.out, "\\vspace{{1em}}")?;
        writeln!(self.out)?;
        match self.events.next().unwrap() {
            Event::End(Tag::Rule) => (),
            _ => unreachable!("rule shouldn't have anything between start and end")
        }
        Ok(())
    }

    fn gen_header(&mut self, level: i32) -> Result<()> {
        let section = read_until!(self, Tag::Header(_))?;
        let replaced = section.chars().map(|c| match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
            _ => '-',
        }).collect::<String>();
        writeln!(self.out, "\\{}section{{{}}}\\label{{{}}}\n", "sub".repeat(level as usize - 1), section, replaced)
    }

    fn gen_block_quote(&mut self) -> Result<()> {
        let quote = read_until!(self, Tag::BlockQuote)?;

        let mut quote = quote.as_str();

        // check if last line of quote is source of quote
        let mut source = None;
        if let Some(pos) = quote.trim_right().rfind("\n") {
            let src = &quote[pos+1..];
            if src.starts_with("--") {
                let src = src.trim_left_matches("-");
                source = Some(src.trim());
                quote = &quote[..pos+1];
            }
        }
        if let Some(source) = source {
            writeln!(self.out, "\\begin{{aquote}}{{{}}}", source)?;
        } else {
            writeln!(self.out, "\\begin{{quote}}")?;
        }
        write!(self.out, "{}", quote)?;
        if source.is_some() {
            writeln!(self.out, "\\end{{aquote}}")?;
        } else {
            writeln!(self.out, "\\end{{quote}}")?;
        }

        Ok(())
    }

    fn gen_code_block(&mut self, lang: &str) -> Result<()> {
        write!(self.out, "\\begin{{lstlisting}}")?;
        if !lang.is_empty() {
            write!(self.out, "[")?;
            let parts = lang.split(",");
            for (i, part) in parts.enumerate() {
                if i == 0 {
                    if !part.contains("=") {
                        // TODO: language translation (use correct language, e.g. `Rust` instead of `rust` if that matters)
                        match lang {
                            // TODO: sequence and stuff generation
                            "sequence" => (),
                            lang => write!(self.out, "language={}", part)?,
                        }
                        continue;
                    }
                }

                if !part.contains("=") {
                    panic!("any code-block argument except the first one (language) must be of format `key=value`");
                }
                write!(self.out, "{}", part)?;
            }
            write!(self.out, "]")?;
        }
        writeln!(self.out)?;

        handle_until!(self, Tag::CodeBlock(_));
        writeln!(self.out, "\\end{{lstlisting}}")
    }

    fn gen_list(&mut self, start: Option<usize>) -> Result<()> {
        if let Some(start) = start {
            let start = start as i32 - 1;
            self.enumerate_depth += 1;
            writeln!(self.out, "\\begin{{enumerate}}")?;
            writeln!(self.out, "\\setcounter{{enum{}}}{{{}}}", "i".repeat(self.enumerate_depth), start)?;
        } else {
            writeln!(self.out, "\\begin{{itemize}}")?;
        }
        handle_until!(self, Tag::List(_));
        if start.is_some() {
            writeln!(self.out, "\\end{{enumerate}}")?;
            self.enumerate_depth -= 1;
        } else {
            writeln!(self.out, "\\end{{itemize}}")?;
        }
        Ok(())
    }

    fn gen_item(&mut self) -> Result<()> {
        write!(self.out, "\\item ")?;
        handle_until!(self, Tag::Item);
        writeln!(self.out)
    }

    // https://github.com/google/pulldown-cmark/issues/20#issuecomment-410453631
    fn gen_footnote_definition(&mut self, fnote: &str) -> Result<()> {
        // TODO: Add pass to get all definitions to put definition on the same site as the first reference
        write!(self.out, "\\footnotetext{{\\label{{fnote:{}}}", fnote)?;
        handle_until!(self, Tag::FootnoteDefinition(..));
        writeln!(self.out, "}}")
    }

    fn gen_table(&mut self, align: Vec<Alignment>) -> Result<()> {
        // TODO: in-cell linebreaks
        // TODO: merging columns
        // TODO: merging rows
        // TODO: easier custom formatting
        write!(self.out, "\\begin{{tabular}}{{|")?;
        for align in align {
            match align {
                Alignment::None | Alignment::Left => write!(self.out, " l |")?,
                Alignment::Center => write!(self.out, " c |")?,
                Alignment::Right => write!(self.out, " r |")?,
            }
        }
        writeln!(self.out, "}}")?;
        writeln!(self.out, "\\hline")?;
        handle_until!(self, Tag::Table(_));
        writeln!(self.out, "\\end{{tabular}}")?;
        Ok(())
    }

    fn gen_table_head(&mut self) -> Result<()> {
        handle_until!(self, Tag::TableHead);
        writeln!(self.out, "\\\\ \\thickhline")
    }

    fn gen_table_row(&mut self) -> Result<()> {
        handle_until!(self, Tag::TableRow);
        writeln!(self.out, "\\\\ \\hline")
    }

    fn gen_table_cell(&mut self) -> Result<()> {
        handle_until!(self, Tag::TableCell);
        if let Event::Start(Tag::TableCell) = self.events.peek().unwrap() {
            write!(self.out, "&")?;
        }
        Ok(())
    }

    fn gen_emphasized(&mut self) -> Result<()> {
        write!(self.out, "\\emph{{")?;
        handle_until!(self, Tag::Emphasis);
        write!(self.out, "}}")
    }

    fn gen_strong(&mut self) -> Result<()> {
        write!(self.out, "\\textbf{{")?;
        handle_until!(self, Tag::Strong);
        write!(self.out, "}}")
    }

    fn gen_code(&mut self) -> Result<()> {
        write!(self.out, "\\texttt{{")?;
        handle_until!(self, Tag::Code);
        write!(self.out, "}}")
    }

    fn gen_link(&mut self, dst: &str, _title: &str) -> Result<()> {
        // TODO: handle all links properly
        // Markdown Types of links: https://github.com/google/pulldown-cmark/issues/141
        //

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
        let text = read_until!(self, Tag::Link(..))?;

        let uppercase = dst.chars().nth(0).unwrap().is_ascii_uppercase();
        let dst = dst.to_ascii_lowercase();
        let dst_eq_text = dst == text.to_ascii_lowercase();

        if dst.starts_with('#') || dst.starts_with("img:") || dst.starts_with("fig:") {
            let dst = if dst.starts_with('#') { &dst[1..] } else { dst.as_str() };
            let text = if text.starts_with('#') { &text[1..] } else { text.as_str() };

            if text.is_empty() || dst_eq_text {
                if uppercase {
                    write!(self.out, "\\Cref{{{}}}", dst)
                } else {
                    write!(self.out, "\\cref{{{}}}", dst)
                }
            } else {
                write!(self.out, "\\hyperref[{}]{{{}}}", dst, text)
            }
        } else {
            if text.is_empty() || dst_eq_text {
                write!(self.out, "\\url{{{}}}", dst)
            } else {
                write!(self.out, "\\href{{{}}}{{{}}}", dst, text)
            }
        }
    }

    fn gen_image(&mut self, dst: &str, title: &str) -> Result<()> {
        writeln!(self.out, "\\begin{{figure}}")?;
        writeln!(self.out, "\\includegraphics{{{}}}", dst)?;
        let caption = read_until!(self, Tag::Image(..))?;
        if !caption.is_empty() {
            writeln!(self.out, "\\caption{{{}}}", caption)?;
        }
        if !title.is_empty() {
            writeln!(self.out, "\\label{{img:{}}}", title)?;
        }
        writeln!(self.out, "\\end{{figure}}")
    }
}

