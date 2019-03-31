use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::ops::Range;

use crate::backend::latex::InlineEnvironment;
use crate::backend::{Backend, CodeGenUnit};
use crate::config::Config;
use crate::generator::Generator;
use crate::generator::event::{Event, Graphviz};
use crate::error::Result;

#[derive(Debug)]
pub struct GraphvizGen<'a> {
    path: PathBuf,
    file: File,
    graphviz: Graphviz<'a>,
}

impl<'a> CodeGenUnit<'a, Graphviz<'a>> for GraphvizGen<'a> {
    fn new(
        cfg: &Config, graphviz: Graphviz<'a>, _range: Range<usize>,
        _gen: &mut Generator<'a, impl Backend<'a>, impl Write>,
    ) -> Result<Self> {
        let mut i = 0;
        let (file, path) = loop {
            let p = cfg.out_dir.join(format!("graphviz_{}", i));
            if !p.exists() {
                break (File::create(&p)?, p);
            }
            i += 1;
        };
        Ok(GraphvizGen { file, path, graphviz })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.file)
    }

    fn finish(
        self, gen: &mut Generator<'a, impl Backend<'a>, impl Write>, _peek: Option<(&Event<'a>, Range<usize>)>,
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
            panic!(
                "dot returned error code {:?}. Logs written to dot_stdout.log and dot_stderr.log",
                out.status.code()
            );
        }
        let out = gen.get_out();
        let inline_fig = InlineEnvironment::new_figure(label, caption);
        inline_fig.write_begin(&mut *out)?;

        write!(out, "\\includegraphics[")?;
        if let Some(scale) = scale {
            write!(out, "scale={},", scale)?;
        }
        if let Some(width) = width {
            write!(out, "width={},", width)?;
        }
        if let Some(height) = height {
            write!(out, "height={},", height)?;
        }
        writeln!(out, "]{{{}.pdf}}", self.path.display())?;

        inline_fig.write_end(out)?;
        Ok(())
    }
}
