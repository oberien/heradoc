use std::io::{Result, Write};
use std::borrow::Cow;

use crate::gen::SimpleCodeGenUnit;
use crate::parser::FootnoteReference;

#[derive(Debug)]
pub struct TextGen;

impl<'a> SimpleCodeGenUnit<Cow<'a, str>> for TextGen {
    fn gen(text: Cow<'a, str>, out: &mut impl Write) -> Result<()> {
        write!(out, "{}", text)?;
        Ok(())
    }

}

#[derive(Debug)]
pub struct FootnoteReferenceGen;

impl<'a> SimpleCodeGenUnit<FootnoteReference<'a>> for FootnoteReferenceGen {
    fn gen(fnote: FootnoteReference, out: &mut impl Write) -> Result<()> {
        write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote.label)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SoftBreakGen;

impl SimpleCodeGenUnit<()> for SoftBreakGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        // soft breaks are only used to split up text in lines in the source file
        // so it's nothing we should translate, but for better readability keep them
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HardBreakGen;

impl SimpleCodeGenUnit<()> for HardBreakGen {
    fn gen((): (), out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\par")?;
        Ok(())
    }

}

