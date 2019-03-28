#![allow(unused)]

use std::rc::Rc;
use std::ops::Range;
use std::path::PathBuf;
use url::Url;

use codespan::{FileMap, FileName, ByteOffset, Span};
use codespan_reporting::{Diagnostic, Label, LabelStyle, Severity};
use codespan_reporting::termcolor::{ColorChoice, StandardStream};

use crate::resolve::Context;

#[derive(Clone, Debug)]
pub struct Diagnostics<'a> {
    file_map: Rc<FileMap<&'a str>>,
}

pub enum Input {
    File(PathBuf),
    Stdin,
    Url(Url),
}

impl<'a> Diagnostics<'a> {
    pub fn new(markdown: &'a str, input: Input) -> Diagnostics<'a> {
        let source = match input {
            Input::File(path) => FileName::real(path),
            Input::Stdin => FileName::Virtual("stdin".into()),
            Input::Url(url) => FileName::Virtual(url.as_str().to_owned().into()),
        };
        let file_map = Rc::new(FileMap::new(source, markdown));

        Diagnostics {
            file_map,
        }
    }

    pub fn first_line(&self, range: &Range<usize>) -> Range<usize> {
        let start = Span::from_offset(self.file_map.span().start(), ByteOffset(range.start as i64)).end();
        let line = self.file_map.location(start).unwrap().0;
        let line_span = self.file_map.line_span(line).unwrap();
        // get rid of newline
        let len = self.file_map.src_slice(line_span).unwrap().trim_end().len();
        Range {
            start: range.start,
            end: range.start + len,
        }
    }

    fn diagnostic(&self, severity: Severity, message: String) -> DiagnosticBuilder<'a, '_> {
        DiagnosticBuilder {
            file_map: &self.file_map,
            severity,
            message,
            code: None,
            labels: Vec::new(),
        }
    }

    pub fn bug<S: Into<String>>(&self, message: S) -> DiagnosticBuilder<'a, '_> {
        self.diagnostic(Severity::Bug, message.into())
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
    severity: Severity,
    message: String,
    code: Option<String>,
    labels: Vec<Label>,
}

impl<'a: 'b, 'b> DiagnosticBuilder<'a, 'b> {
    pub fn with_error_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    /// message can be empty
    pub fn with_section<S: Into<String>>(mut self, range: &Range<usize>, message: S) -> Self {
        let style = if self.labels.len() == 0 {
            LabelStyle::Primary
        } else {
            LabelStyle::Secondary
        };
        let span = self.file_map.span().subspan(ByteOffset(range.start as i64), ByteOffset(range.end as i64));
        let message = message.into();
        let message = if message.len() > 0 {
            Some(message)
        } else {
            None
        };
        self.labels.push(Label { span, message, style });
        self
    }

    pub fn emit(self) {
        let Self { file_map, severity, message, code, labels } = self;
        // TODO: make this configurable
        let out = StandardStream::stderr(ColorChoice::Auto);

        // ignore output errors, because where would we log them anyway?!
        let _ = codespan_reporting::emit_single(out, file_map, &Diagnostic {
            severity,
            code,
            message,
            labels,
        });
    }
}
