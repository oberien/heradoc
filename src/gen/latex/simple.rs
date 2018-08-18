use std::io::{Result, Write};

use crate::gen::Simple;

#[derive(Debug)]
pub struct SimpleGen;

impl Simple for SimpleGen {
    fn gen_text(text: &str, out: &mut impl Write) -> Result<()> {
        write!(out, "{}", text)?;
        Ok(())
    }

    fn gen_footnote_reference(fnote: &str, out: &mut impl Write) -> Result<()> {
        write!(out, "\\footnotemark[\\getrefnumber{{fnote:{}}}]", fnote)?;
        Ok(())
    }

    fn gen_soft_break(out: &mut impl Write) -> Result<()> {
        // soft breaks are only used to split up text in lines in the source file
        // so it's nothing we should translate, but for better readability keep them
        writeln!(out)?;
        Ok(())
    }

    fn gen_hard_break(out: &mut impl Write) -> Result<()> {
        writeln!(out, "\\par")?;
        Ok(())
    }

}

