use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag, Alignment};

use crate::gen::Generator;
use crate::gen::peek::Peek;

mod list;
mod table;

pub use self::list::ListGenerator;
pub use self::table::{gen_table, gen_table_cell, gen_table_head, gen_table_row};

pub fn gen_text<'a>(text: &str, out: &mut impl Write) -> Result<()> {
    write!(out, "{}", text)
}

pub fn gen_footnote_reference<'a>(fnote: &str, out: &mut impl Write) -> Result<()> {
    write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote)
}

pub fn gen_soft_break<'a>(out: &mut impl Write) -> Result<()> {
    // soft breaks are only used to split up text in lines in the source file
    // so it's nothing we should translate, but for better readability keep them
    writeln!(out)
}

pub fn gen_hard_break<'a>(out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\par")
}

pub fn gen_par(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    handle_until!(gen, events, out, Tag::Paragraph);
    // TODO: improve readability (e.g. no newline between list items)
    match events.peek() {
        Some(Event::Text(_))
        | Some(Event::Html(_))
        | Some(Event::InlineHtml(_))
        | Some(Event::Start(Tag::Paragraph))
        // those shouldn't occur after a par, but better safe than sorry
        | Some(Event::Start(Tag::Emphasis))
        | Some(Event::Start(Tag::Strong))
        | Some(Event::Start(Tag::Code))
        | Some(Event::Start(Tag::Link(..)))
        | Some(Event::Start(Tag::Image(..))) => writeln!(out, "\\\\\n\\\\")?,
        _ => writeln!(out)?,
    }
    Ok(())
}

pub fn gen_rule(events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    // TODO: find out why text after the hrule is indented
    writeln!(out)?;
    writeln!(out, "\\vspace{{1em}}")?;
    writeln!(out, "\\hrule")?;
    writeln!(out, "\\vspace{{1em}}")?;
    writeln!(out)?;
    match events.next().unwrap() {
        Event::End(Tag::Rule) => (),
        _ => unreachable!("rule shouldn't have anything between start and end")
    }
    Ok(())
}

pub fn gen_header(gen: &mut impl Generator<'a>, level: i32, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    let section = read_until!(gen, events, Tag::Header(_));
    let replaced = section.chars().map(|c| match c {
        'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
        _ => '-',
    }).collect::<String>();
    writeln!(out, "\\{}section{{{}}}\\label{{{}}}\n", "sub".repeat(level as usize - 1), section, replaced)
}

pub fn gen_block_quote(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    let quote = read_until!(gen, events, Tag::BlockQuote);

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
        writeln!(out, "\\begin{{aquote}}{{{}}}", source)?;
    } else {
        writeln!(out, "\\begin{{quote}}")?;
    }
    write!(out, "{}", quote)?;
    if source.is_some() {
        writeln!(out, "\\end{{aquote}}")?;
    } else {
        writeln!(out, "\\end{{quote}}")?;
    }

    Ok(())
}

pub fn gen_code_block(gen: &mut impl Generator<'a>, lang: &str, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    write!(out, "\\begin{{lstlisting}}")?;
    if !lang.is_empty() {
        write!(out, "[")?;
        let parts = lang.split(",");
        for (i, part) in parts.enumerate() {
            if i == 0 {
                if !part.contains("=") {
                    // TODO: language translation (use correct language, e.g. `Rust` instead of `rust` if that matters)
                    match lang {
                        // TODO: sequence and stuff generation
                        "sequence" => (),
                        lang => write!(out, "language={}", lang)?,
                    }
                    continue;
                }
            }

            if !part.contains("=") {
                panic!("any code-block argument except the first one (language) must be of format `key=value`");
            }
            write!(out, "{}", part)?;
        }
        write!(out, "]")?;
    }
    writeln!(out)?;

    handle_until!(gen, events, out, Tag::CodeBlock(_));
    writeln!(out, "\\end{{lstlisting}}")
}

// https://github.com/google/pulldown-cmark/issues/20#issuecomment-410453631
pub fn gen_footnote_definition(gen: &mut impl Generator<'a>, fnote: &str, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    // TODO: Add pass to get all definitions to put definition on the same site as the first reference
    write!(out, "\\footnotetext{{\\label{{fnote:{}}}", fnote)?;
    handle_until!(gen, events, out, Tag::FootnoteDefinition(..));
    writeln!(out, "}}")
}

pub fn gen_emphasized(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    write!(out, "\\emph{{")?;
    handle_until!(gen, events, out, Tag::Emphasis);
    write!(out, "}}")
}

pub fn gen_strong(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    write!(out, "\\textbf{{")?;
    handle_until!(gen, events, out, Tag::Strong);
    write!(out, "}}")
}

pub fn gen_code(gen: &mut impl Generator<'a>, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    write!(out, "\\texttt{{")?;
    handle_until!(gen, events, out, Tag::Code);
    write!(out, "}}")
}

pub fn gen_link(gen: &mut impl Generator<'a>, dst: &str, _title: &str, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
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
    let text = read_until!(gen, events, Tag::Link(..));

    let uppercase = dst.chars().nth(0).unwrap().is_ascii_uppercase();
    let dst = dst.to_ascii_lowercase();
    let dst_eq_text = dst == text.to_ascii_lowercase();

    if dst.starts_with('#') || dst.starts_with("img:") || dst.starts_with("fig:") {
        let dst = if dst.starts_with('#') { &dst[1..] } else { dst.as_str() };
        let text = if text.starts_with('#') { &text[1..] } else { text.as_str() };

        if text.is_empty() || dst_eq_text {
            if uppercase {
                write!(out, "\\Cref{{{}}}", dst)
            } else {
                write!(out, "\\cref{{{}}}", dst)
            }
        } else {
            write!(out, "\\hyperref[{}]{{{}}}", dst, text)
        }
    } else {
        if text.is_empty() || dst_eq_text {
            write!(out, "\\url{{{}}}", dst)
        } else {
            write!(out, "\\href{{{}}}{{{}}}", dst, text)
        }
    }
}

pub fn gen_image(gen: &mut impl Generator<'a>, dst: &str, title: &str, events: &mut impl Peek<Item = Event<'a>>, out: &mut impl Write) -> Result<()> {
    writeln!(out, "\\begin{{figure}}")?;
    writeln!(out, "\\includegraphics{{{}}}", dst)?;
    let caption = read_until!(gen, events, Tag::Image(..));
    if !caption.is_empty() {
        writeln!(out, "\\caption{{{}}}", caption)?;
    }
    if !title.is_empty() {
        writeln!(out, "\\label{{img:{}}}", title)?;
    }
    writeln!(out, "\\end{{figure}}")
}
