use std::io::{Write, Result};

use pulldown_cmark::{Event, Tag};

use crate::gen::{Generator, State, Container};
use crate::gen::peek::Peek;

mod list;
mod table;

pub use self::list::ListGenerator;
pub use self::table::{gen_table, gen_table_cell, gen_table_head, gen_table_row};

pub fn gen_text<'a>(text: &str, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    write!(state.out, "{}", text)
}

pub fn gen_footnote_reference<'a>(fnote: &str, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    write!(state.out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote)
}

pub fn gen_soft_break<'a>(state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    // soft breaks are only used to split up text in lines in the source file
    // so it's nothing we should translate, but for better readability keep them
    writeln!(state.out)
}

pub fn gen_hard_break<'a>(state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    writeln!(state.out, "\\par")
}

pub fn gen_par(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::Paragraph);
    handle_until!(gen, state, Tag::Paragraph);
    // TODO: improve readability (e.g. no newline between list items)
    match state.events.peek() {
        Some(Event::Text(_))
        | Some(Event::Html(_))
        | Some(Event::InlineHtml(_))
        | Some(Event::Start(Tag::Paragraph))
        // those shouldn't occur after a par, but better safe than sorry
        | Some(Event::Start(Tag::Emphasis))
        | Some(Event::Start(Tag::Strong))
        | Some(Event::Start(Tag::Code))
        | Some(Event::Start(Tag::Link(..)))
        | Some(Event::Start(Tag::Image(..))) => writeln!(state.out, "\\\\\n\\\\")?,
        _ => writeln!(state.out)?,
    }
    assert_eq!(state.stack.pop(), Some(Container::Paragraph));
    Ok(())
}

pub fn gen_rule(state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    // TODO: find out why text after the hrule is indented
    writeln!(state.out)?;
    writeln!(state.out, "\\vspace{{1em}}")?;
    writeln!(state.out, "\\hrule")?;
    writeln!(state.out, "\\vspace{{1em}}")?;
    writeln!(state.out)?;
    match state.events.next().unwrap() {
        Event::End(Tag::Rule) => (),
        // TODO: check this
        _ => unreachable!("rule shouldn't have anything between start and end")
    }
    Ok(())
}

pub fn gen_header(gen: &mut impl Generator<'a>, level: i32, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::Header);
    let section = read_until!(gen, state, Tag::Header(_));
    let replaced = section.chars().map(|c| match c {
        'a'...'z' | 'A'...'Z' | '0'...'9' => c.to_ascii_lowercase(),
        _ => '-',
    }).collect::<String>();
    writeln!(state.out, "\\{}section{{{}}}\\label{{{}}}\n", "sub".repeat(level as usize - 1), section, replaced)?;
    assert_eq!(state.stack.pop(), Some(Container::Header));
    Ok(())
}

pub fn gen_block_quote(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::BlockQuote);
    let quote = read_until!(gen, state, Tag::BlockQuote);

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
        writeln!(state.out, "\\begin{{aquote}}{{{}}}", source)?;
    } else {
        writeln!(state.out, "\\begin{{quote}}")?;
    }
    write!(state.out, "{}", quote)?;
    if source.is_some() {
        writeln!(state.out, "\\end{{aquote}}")?;
    } else {
        writeln!(state.out, "\\end{{quote}}")?;
    }

    assert_eq!(state.stack.pop(), Some(Container::BlockQuote));
    Ok(())
}

pub fn gen_code_block(gen: &mut impl Generator<'a>, lang: &str, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::CodeBlock);
    write!(state.out, "\\begin{{lstlisting}}")?;
    if !lang.is_empty() {
        write!(state.out, "[")?;
        let parts = lang.split(",");
        for (i, part) in parts.enumerate() {
            if i == 0 {
                if !part.contains("=") {
                    // TODO: language translation (use correct language, e.g. `Rust` instead of `rust` if that matters)
                    match lang {
                        // TODO: sequence and stuff generation
                        "sequence" => (),
                        lang => write!(state.out, "language={}", lang)?,
                    }
                    continue;
                }
            }

            if !part.contains("=") {
                panic!("any code-block argument except the first one (language) must be of format `key=value`");
            }
            write!(state.out, "{}", part)?;
        }
        write!(state.out, "]")?;
    }
    writeln!(state.out)?;

    handle_until!(gen, state, Tag::CodeBlock(_));
    writeln!(state.out, "\\end{{lstlisting}}")?;

    assert_eq!(state.stack.pop(), Some(Container::CodeBlock));
    Ok(())
}

// https://github.com/google/pulldown-cmark/issues/20#issuecomment-410453631
pub fn gen_footnote_definition(gen: &mut impl Generator<'a>, fnote: &str, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::FootnoteDefinition);
    // TODO: Add pass to get all definitions to put definition on the same site as the first reference
    write!(state.out, "\\footnotetext{{\\label{{fnote:{}}}", fnote)?;
    handle_until!(gen, state, Tag::FootnoteDefinition(..));
    writeln!(state.out, "}}")?;
    assert_eq!(state.stack.pop(), Some(Container::FootnoteDefinition));
    Ok(())
}

pub fn gen_emphasized(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::InlineEmphasis);
    write!(state.out, "\\emph{{")?;
    handle_until!(gen, state, Tag::Emphasis);
    write!(state.out, "}}")?;
    assert_eq!(state.stack.pop(), Some(Container::InlineEmphasis));
    Ok(())
}

pub fn gen_strong(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::InlineStrong);
    write!(state.out, "\\textbf{{")?;
    handle_until!(gen, state, Tag::Strong);
    write!(state.out, "}}")?;
    assert_eq!(state.stack.pop(), Some(Container::InlineStrong));
    Ok(())
}

pub fn gen_code(gen: &mut impl Generator<'a>, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::InlineCode);
    write!(state.out, "\\texttt{{")?;
    handle_until!(gen, state, Tag::Code);
    write!(state.out, "}}")?;
    assert_eq!(state.stack.pop(), Some(Container::InlineCode));
    Ok(())
}

pub fn gen_link(gen: &mut impl Generator<'a>, dst: &str, _title: &str, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::Link);
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
    let text = read_until!(gen, state, Tag::Link(..));

    let uppercase = dst.chars().nth(0).unwrap().is_ascii_uppercase();
    let dst = dst.to_ascii_lowercase();
    let dst_eq_text = dst == text.to_ascii_lowercase();

    if dst.starts_with('#') || dst.starts_with("img:") || dst.starts_with("fig:") {
        let dst = if dst.starts_with('#') { &dst[1..] } else { dst.as_str() };
        let text = if text.starts_with('#') { &text[1..] } else { text.as_str() };

        if text.is_empty() || dst_eq_text {
            if uppercase {
                write!(state.out, "\\Cref{{{}}}", dst)?;
            } else {
                write!(state.out, "\\cref{{{}}}", dst)?;
            }
        } else {
            write!(state.out, "\\hyperref[{}]{{{}}}", dst, text)?;
        }
    } else {
        if text.is_empty() || dst_eq_text {
            write!(state.out, "\\url{{{}}}", dst)?;
        } else {
            write!(state.out, "\\href{{{}}}{{{}}}", dst, text)?;
        }
    }
    assert_eq!(state.stack.pop(), Some(Container::Link));
    Ok(())
}

pub fn gen_image(gen: &mut impl Generator<'a>, dst: &str, title: &str, state: &mut State<'a, impl Peek<Item = Event<'a>>, impl Write>) -> Result<()> {
    state.stack.push(Container::Image);
    writeln!(state.out, "\\begin{{figure}}")?;
    writeln!(state.out, "\\includegraphics{{{}}}", dst)?;
    let caption = read_until!(gen, state, Tag::Image(..));
    if !caption.is_empty() {
        writeln!(state.out, "\\caption{{{}}}", caption)?;
    }
    if !title.is_empty() {
        writeln!(state.out, "\\label{{img:{}}}", title)?;
    }
    writeln!(state.out, "\\end{{figure}}")?;
    assert_eq!(state.stack.pop(), Some(Container::Image));
    Ok(())
}
