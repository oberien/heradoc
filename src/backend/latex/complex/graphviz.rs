use std::io::{Result, Write};
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::backend::{Backend, CodeGenUnit};
use crate::generator::PrimitiveGenerator;
use crate::generator::event::{Event, Graphviz};

#[derive(Debug)]
pub struct GraphvizGen<'a> {
    path: PathBuf,
    file: File,
    graphviz: Graphviz<'a>,
}

impl<'a> CodeGenUnit<'a, Graphviz<'a>> for GraphvizGen<'a> {
    fn new(cfg: &Config, graphviz: Graphviz<'a>, _gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>) -> Result<Self> {
        let mut i = 0;
        let (file, path) = loop {
            let p = cfg.out_dir.join(format!("graphviz_{}", i));
            if !p.exists() {
                break (File::create(&p)?, p);
            }
            i += 1;
        };
        Ok(GraphvizGen {
            file,
            path,
            graphviz,
        })
    }

    fn output_redirect(&mut self) -> Option<&mut dyn Write> {
        Some(&mut self.file)
    }


    fn finish(self, gen: &mut PrimitiveGenerator<'a, impl Backend<'a>, impl Write>, _peek: Option<&Event<'a>>) -> Result<()> {
        drop(self.file);
        let out = Command::new("dot").args(&["-T", "pdf", "-O"]).arg(&self.path).output()
            .expect("Error executing `dot` to generate graphviz output");
        if !out.status.success() {
            let _ = File::create("dot_stdout.log")
                .map(|mut f| f.write_all(&out.stdout));
            let _ = File::create("dot_stderr.log")
                .map(|mut f| f.write_all(&out.stderr));
            // TODO: provide better info about signals
            panic!("dot returned error code {:?}. Logs written to dot_stdout.log and dot_stderr.log", out.status.code());
        }
        let out = gen.get_out();
        writeln!(out, "\\begin{{figure}}")?;
        write!(out, "\\includegraphics[")?;
        if let Some(scale) = self.graphviz.scale {
            write!(out, "scale={},", scale)?;
        }
        if let Some(width) = self.graphviz.width {
            write!(out, "width={},", width)?;
        }
        if let Some(height) = self.graphviz.height {
            write!(out, "height={},", height)?;
        }
        writeln!(out, "]{{{}.pdf}}", self.path.display())?;

        if let Some(caption) = self.graphviz.caption {
            writeln!(out, "\\caption{{{}}}", caption)?;
        }
        writeln!(out, "\\end{{figure}}")?;

        Ok(())
    }
}
