use std::io::Write;

use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::Result;
use crate::frontend::range::WithRange;
use crate::generator::event::{Event, Proof, ProofKind};
use crate::generator::Generator;

trait ContextName {
    fn context_name(&self) -> &'static str;
}

impl ContextName for ProofKind {
    fn context_name(&self) -> &'static str {
        match self {
            ProofKind::Corollary => "amsthm-corollary",
            ProofKind::Definition => "amsthm-definition",
            ProofKind::Lemma => "amsthm-lemma",
            ProofKind::Proof => "proof",
            ProofKind::Theorem=> "amsthm-theorem",
        }
    }
}

#[derive(Debug)]
pub struct ProofGen {
    kind: ProofKind,
}

impl<'a> CodeGenUnit<'a, Proof<'a>> for ProofGen {
    fn new(
        _cfg: &'a Config, proof: WithRange<Proof<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let WithRange(element, _range) = proof;
        let out = gen.get_out();
        write!(out, "\\begin{{{}}}", element.kind.context_name())?;
        if let Some(WithRange(title, _)) = element.title {
            write!(out, "[{}]", title)?;
        }
        if let Some(WithRange(label, _)) = element.label {
            write!(out, "\\label{{{}}}", label)?;
        }
        Ok(ProofGen {
            kind: element.kind,
        })
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<WithRange<&Event<'a>>>,
    ) -> Result<()> {
        write!(gen.get_out(), "\\end{{{}}}", self.kind.context_name())?;
        Ok(())
    }
}
