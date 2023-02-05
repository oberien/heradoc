use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::Command;
use diagnostic::{Span, Spanned};

use crate::backend::latex::InlineEnvironment;
use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::{DiagnosticCode, Error, Result};
use crate::generator::event::{Event, Graphviz};
use crate::generator::Generator;
use crate::util::ToUnix;

#[derive(Debug)]
pub struct GraphvizGen<'a> {
    path: PathBuf,
    file: File,
    graphviz: Graphviz<'a>,
    span: Span,
}

impl<'a> CodeGenUnit<'a, Graphviz<'a>> for GraphvizGen<'a> {
    fn new(
        cfg: &Config, graphviz: Spanned<Graphviz<'a>>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let Spanned { value: graphviz, span } = graphviz;
        let mut i = 0;
        let (file, path) = loop {
            let p = cfg.out_dir.join(format!("graphviz_{}", i));
            let res = OpenOptions::new().create_new(true).write(true).open(&p);
            match res {
                Ok(file) => break (file, p),
                Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {
                    i += 1;
                    continue;
                },
                Err(e) => {
                    gen.diagnostics()
                        .bug(DiagnosticCode::TempFileError)
                        .with_info_label(span, "can't create temporary file for this graphviz code")
                        .with_info_label(span, format!("cause: {}", e))
                        .with_note("skipping over it")
                        .emit();
                    return Err(Error::Diagnostic);
                },
            }
            // :thonking:
            #[allow(unreachable_code)]
            {
                unreachable!();
            }
        };
        Ok(GraphvizGen { file, path, graphviz, span })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.file)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<Spanned<&Event<'a>>>,
    ) -> Result<()> {
        drop(self.file);
        let Graphviz { label, caption, scale, width, height } = self.graphviz;
        let out = Command::new("dot")
            .args(&["-T", "pdf", "-O"])
            .arg(&self.path)
            .output()
            .expect("Error executing `dot` to generate graphviz output");
        if !out.status.success() {
            let _ = File::create("dot_stdout.log").map(|mut f| f.write_all(&out.stdout));
            let _ = File::create("dot_stderr.log").map(|mut f| f.write_all(&out.stderr));
            // TODO: provide better info about signals
            // TODO: parse the dot output and provide appropriate error messages
            gen.diagnostics()
                .error(DiagnosticCode::GraphvizError)
                .with_error_label(self.span, "error trying to render this graphviz cdoe block")
                .with_note(format!("`dot` returned error code {:?}", out.status.code()))
                .with_note("logs written to dot_stdout.log and dot_stderr.log")
                .with_note("skipping over it")
                .emit();
            return Err(Error::Diagnostic);
        }
        let out = gen.get_out();
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        inline_fig.write_begin(&mut *out)?;

        write!(out, "\\includegraphics[")?;
        if let Some(Spanned { value: scale, .. }) = scale {
            write!(out, "scale={},", scale)?;
        }
        if let Some(Spanned { value: width, .. }) = width {
            write!(out, "width={},", width)?;
        }
        if let Some(Spanned { value: height, .. }) = height {
            write!(out, "height={},", height)?;
        }
        writeln!(out, "]{{{}.pdf}}", self.path.to_unix()
            .expect(&format!("non-utf8 path: {:?}", self.path)))?;

        inline_fig.write_end(out)?;
        Ok(())
    }
}
