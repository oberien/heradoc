#![allow(dead_code)]

use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use codespan::{ByteOffset, FileMap, FileName, Span};
use codespan_reporting::termcolor::StandardStream;
use codespan_reporting::{Diagnostic, Label, LabelStyle, Severity};
use url::Url;

use crate::frontend::range::SourceRange;

pub struct Diagnostics<'a> {
    file_map: FileMap<&'a str>,
    stderr: Arc<Mutex<StandardStream>>,
}

impl<'a> fmt::Debug for Diagnostics<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diagnostics")
            .field("file_map", &self.file_map)
            .field("out", &"Arc(Mutex(StandardStream))")
            .finish()
    }
}

pub enum Input {
    File(PathBuf),
    Stdin,
    Url(Url),
}

impl<'a> Diagnostics<'a> {
    pub fn new(markdown: &'a str, input: Input, stderr: Arc<Mutex<StandardStream>>) -> Diagnostics<'a> {
        let source = match input {
            Input::File(path) => FileName::real(path),
            Input::Stdin => FileName::Virtual("stdin".into()),
            Input::Url(url) => FileName::Virtual(url.as_str().to_owned().into()),
        };
        let file_map = FileMap::new(source, markdown);

        Diagnostics { file_map, stderr }
    }

    pub fn first_line(&self, range: SourceRange) -> SourceRange {
        let start =
            Span::from_offset(self.file_map.span().start(), ByteOffset(range.start as i64)).end();
        let line = self.file_map.location(start).unwrap().0;
        let line_span = self.file_map.line_span(line).unwrap();
        // get rid of newline
        let len = self.file_map.src_slice(line_span).unwrap().trim_end().len();
        SourceRange { start: range.start, end: range.start + len }
    }

    fn diagnostic(&self, severity: Severity, message: String) -> DiagnosticBuilder<'a, '_> {
        DiagnosticBuilder {
            file_map: &self.file_map,
            // we're borrowing anyway, no need to increase refcount
            stderr: &self.stderr,
            diagnostics: Vec::new(),
            severity,
            message,
            code: None,
            labels: Vec::new(),
        }
    }

    pub fn bug<S: Into<String>>(&self, message: S) -> DiagnosticBuilder<'a, '_> {
        let mut diag =
            Some(self.diagnostic(Severity::Bug, message.into()).note("please report this"));
        backtrace::trace(|frame| {
            let ip = frame.ip();
            backtrace::resolve(ip, |symbol| {
                diag = Some(diag.take().unwrap().note(format!(
                    "in heradoc file {:?} name {:?} line {:?} address {:?}",
                    symbol.filename(),
                    symbol.name(),
                    symbol.lineno(),
                    symbol.addr()
                )));
            });
            true
        });
        diag.unwrap()
    }

    pub fn error<S: Into<String>>(&self, message: S) -> DiagnosticBuilder<'a, '_> {
        self.diagnostic(Severity::Error, message.into())
    }

    pub fn warning<S: Into<String>>(&self, message: S) -> DiagnosticBuilder<'a, '_> {
        self.diagnostic(Severity::Warning, message.into())
    }

    pub fn note<S: Into<String>>(&self, message: S) -> DiagnosticBuilder<'a, '_> {
        self.diagnostic(Severity::Note, message.into())
    }

    pub fn help<S: Into<String>>(&self, message: S) -> DiagnosticBuilder<'a, '_> {
        self.diagnostic(Severity::Help, message.into())
    }
}

#[must_use = "call `emit` to emit the diagnostic"]
pub struct DiagnosticBuilder<'a: 'b, 'b> {
    file_map: &'b FileMap<&'a str>,
    stderr: &'b Arc<Mutex<StandardStream>>,
    diagnostics: Vec<Diagnostic>,

    severity: Severity,
    message: String,
    code: Option<String>,
    labels: Vec<Label>,
}

impl<'a: 'b, 'b> DiagnosticBuilder<'a, 'b> {
    pub fn emit(self) {
        let Self { file_map, stderr, mut diagnostics, severity, message, code, labels } = self;
        diagnostics.push(Diagnostic { severity, message, code, labels });
        let mut stderr = stderr.lock().unwrap();

        // ignore output errors, because where would we log them anyway?!
        for diagnostic in diagnostics {
            codespan_reporting::emit_single(&mut *stderr, file_map, &diagnostic)
                .expect("stdout is gone???");
        }
        writeln!(stderr).expect("stdout is gone???");
    }

    fn diagnostic(self, new_severity: Severity, new_message: String) -> Self {
        let Self { file_map, stderr, mut diagnostics, severity, message, code, labels } = self;
        diagnostics.push(Diagnostic { severity, message, code, labels });

        Self {
            file_map,
            stderr,
            diagnostics,
            severity: new_severity,
            message: new_message,
            code: None,
            labels: Vec::new(),
        }
    }

    pub fn bug<S: Into<String>>(self, message: S) -> Self {
        self.diagnostic(Severity::Bug, message.into())
    }

    pub fn error<S: Into<String>>(self, message: S) -> Self {
        self.diagnostic(Severity::Error, message.into())
    }

    pub fn warning<S: Into<String>>(self, message: S) -> Self {
        self.diagnostic(Severity::Warning, message.into())
    }

    pub fn note<S: Into<String>>(self, message: S) -> Self {
        self.diagnostic(Severity::Note, message.into())
    }

    pub fn help<S: Into<String>>(self, message: S) -> Self {
        self.diagnostic(Severity::Help, message.into())
    }

    pub fn with_error_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    fn with_section<S: Into<String>>(
        mut self, style: LabelStyle, range: SourceRange, message: S,
    ) -> Self {
        let span = self
            .file_map
            .span()
            .subspan(ByteOffset(range.start as i64), ByteOffset(range.end as i64));
        let message = message.into();
        let message = Some(message);
        self.labels.push(Label { span, message, style });
        self
    }

    /// message can be empty
    pub fn with_error_section<S: Into<String>>(self, range: SourceRange, message: S) -> Self {
        self.with_section(LabelStyle::Primary, range, message)
    }

    /// message can be empty
    pub fn with_info_section<S: Into<String>>(self, range: SourceRange, message: S) -> Self {
        self.with_section(LabelStyle::Secondary, range, message)
    }
}
