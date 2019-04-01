use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::process::Command;

use crate::backend::latex::InlineEnvironment;
use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::generator::event::{Event, Graphviz};
use crate::generator::Generator;

#[derive(Debug)]
pub struct GraphvizGen<'a> {
    path: PathBuf,
    file: File,
    graphviz: Graphviz<'a>,
    range: Range<usize>,
}

impl<'a> CodeGenUnit<'a, Graphviz<'a>> for GraphvizGen<'a> {
    fn new(
        cfg: &Config, graphviz: Graphviz<'a>, range: Range<usize>,
        gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
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
                        .bug("error creating temporary graphviz file")
                        .with_info_section(&range, "for this graphviz code")
                        .note(format!("cause: {}", e))
                        .note("skipping over it")
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
        Ok(GraphvizGen { file, path, graphviz, range })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.file)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
        _peek: Option<(&Event<'a>, Range<usize>)>,
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
                .error("graphviz rendering failed")
                .with_error_section(&self.range, "trying to render this graphviz cdoe block")
                .note(format!("`dot` returned error code {:?}", out.status.code()))
                .note("logs written to dot_stdout.log and dot_stderr.log")
                .note("skipping over it")
                .emit();
            return Err(Error::Diagnostic);
        }
        let out = gen.get_out();
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        inline_fig.write_begin(&mut *out)?;

        write!(out, "\\includegraphics[")?;
        if let Some((scale, _)) = scale {
            write!(out, "scale={},", scale)?;
        }
        if let Some((width, _)) = width {
            write!(out, "width={},", width)?;
        }
        if let Some((height, _)) = height {
            write!(out, "height={},", height)?;
        }
        writeln!(out, "]{{{}.pdf}}", self.path.display())?;

        inline_fig.write_end(out)?;
        Ok(())
    }
}
