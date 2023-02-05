use std::borrow::Cow;
use std::collections::HashMap;
use std::mem;
use std::fmt;
use diagnostic::{FileId, Span, Spanned};

use quoted_string::test_utils::TestSpec;
use single::{self, Single};
use crate::Diagnostics;

use crate::error::DiagnosticCode;
use crate::ext::{CowExt, VecExt};

#[derive(Debug, PartialEq, Eq)]
struct Diagnostic;

pub struct Cskvp<'a> {
    diagnostics: Option<&'a Diagnostics>,
    span: Span,
    label: Option<Spanned<Cow<'a, str>>>,
    caption: Option<Spanned<Cow<'a, str>>>,
    figure: Option<Spanned<bool>>,
    single: Vec<Spanned<Cow<'a, str>>>,
    double: HashMap<Cow<'a, str>, Spanned<Cow<'a, str>>>,
}

impl<'a> fmt::Debug for Cskvp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cskvp")
            .field("label", &self.label)
            .field("caption", &self.caption)
            .field("figure", &self.figure)
            .field("single", &self.single)
            .field("double", &self.double)
            .finish()
    }
}

impl<'a> Default for Cskvp<'a> {
    fn default() -> Self {
        Cskvp {
            diagnostics: None,
            span: Span { file: FileId::synthetic("nonexistent"), start: 0, end: 0 },
            label: None,
            caption: None,
            figure: None,
            single: Vec::new(),
            double: HashMap::new(),
        }
    }
}

impl<'a> Cskvp<'a> {
    pub fn new(
        s: Cow<'a, str>, span: Span, content_span: Span,
        diagnostics: &'a Diagnostics,
    ) -> Cskvp<'a> {
        // The content_span may involve unescaped escaped sequences like `\\`, which will only have
        // a length of 1 here. Thus we need to trim the span for the parser.
        // TODO: fix for diagnostics when unescaping is involved. See #
        let parser_span = content_span.with_len(s.len());
        let mut parser = Parser::new(s, parser_span);

        let mut single = Vec::new();
        let mut double = HashMap::new();
        let mut label: Option<Spanned<_>> = None;
        while let Ok(Some(Spanned { value, span })) = parser.next(&diagnostics) {
            match value {
                Value::Double(key, value) => {
                    double.insert(key, Spanned { value, span });
                },
                Value::Single(mut value) => {
                    if value.starts_with('#') {
                        if label.is_some() {
                            diagnostics.warning(DiagnosticCode::FoundTwoLabels)
                                .with_info_label(
                                    label.as_ref().unwrap().span,
                                    "first label defined here",
                                )
                                .with_info_label(span, "second label defined here")
                                .with_note("using the last one")
                                .emit();
                        }
                        value.truncate_start(1);
                        label = Some(Spanned { value, span });
                    } else {
                        single.push(Spanned { value, span });
                    }
                },
            }
        }

        let figure_double = double.remove("figure").and_then(|Spanned { value, span }| match value.parse() {
            Ok(value) => Some(Spanned { value, span }),
            Err(_) => {
                diagnostics
                    .error(DiagnosticCode::InvalidFigureValue)
                    .with_error_label(span, "defined here")
                    .with_note("only `true` and `false` are allowed")
                    .emit();
                None
            },
        });
        let figure = single.remove_element(&"figure").map(|Spanned { span, .. }| Spanned { value: true, span });
        let nofigure = single.remove_element(&"nofigure").map(|Spanned { span, .. }| Spanned { value: false, span });

        let figures = [figure_double, figure, nofigure];
        let figure = match figures.iter().cloned().flatten().single() {
            Ok(val) => Some(val),
            Err(single::Error::NoElements) => None,
            Err(single::Error::MultipleElements) => {
                let mut diag = diagnostics.error(DiagnosticCode::MultipleFigureKeys);
                for Spanned { span, .. } in figures.iter().cloned().flatten() {
                    diag = diag.with_info_label(span, "one defined here");
                }
                diag.with_note(
                        "only one of `figure=true`, `figure=false`, `figure` and `nofigure` is \
                         allowed",
                    ).emit();
                None
            },
        };

        Cskvp {
            diagnostics: Some(diagnostics),
            span,
            label,
            caption: double.remove("caption"),
            figure,
            single,
            double,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn take_label(&mut self) -> Option<Spanned<Cow<'a, str>>> {
        self.label.take()
    }

    pub fn take_figure(&mut self) -> Option<Spanned<bool>> {
        self.figure.take()
    }

    pub fn take_caption(&mut self) -> Option<Spanned<Cow<'a, str>>> {
        self.caption.take()
    }

    pub fn take_double(&mut self, key: &str) -> Option<Spanned<Cow<'a, str>>> {
        self.double.remove(key)
    }

    /// Removes all elements from `self`.
    ///
    /// This can be used before dropping `Cskvp` to omit all "unused attribute" warnings.
    pub fn clear(&mut self) {
        let _ = self.label.take();
        let _ = self.figure.take();
        let _ = self.caption.take();
        self.single.clear();
        self.double.clear();
    }
}

impl<'a> Drop for Cskvp<'a> {
    fn drop(&mut self) {
        let mut has_warning = false;
        let span = self.span;
        let mut diag = self.diagnostics
            .as_mut()
            .map(|d| d.warning(DiagnosticCode::UnknownAttributesInElementConfig)
                .with_info_label(span, "unknown attributes in element config"));
        if let Some(Spanned { value: label, span }) = self.label.take() {
            diag = diag.map(|d| {
                d.with_info_label(span, format!("label ignored: {}", label))
            });
            has_warning = true;
        }
        if let Some(Spanned { value: figure, span }) = self.figure.take() {
            diag = diag.map(|d| {
                d.with_info_label(span, format!("figure config ignored: {}", figure))
            });
            has_warning = true;
        }
        if let Some(Spanned { value: caption, span }) = self.caption.take() {
            diag = diag.map(|d| {
                d.with_info_label(span, format!("caption ignored: {}", caption))
            });
            has_warning = true;
        }
        for (k, Spanned { value: v, span }) in self.double.drain() {
            diag = diag.map(|d| {
                d.with_info_label(span, format!("unknown attribute `{}={}`", k, v))
            });
            has_warning = true;
        }
        for Spanned { value: attr, span } in self.single.drain(..) {
            diag = diag.map(|d| {
                d.with_info_label(span, format!("unknown attribute `{}`", attr))
            });
            has_warning = true;
        }

        if has_warning {
            diag.map(|d| d.emit());
        } else {
            let _ = diag;
        }
    }
}

#[derive(Debug)]
struct Parser<'a> {
    rest: Cow<'a, str>,
    span: Span,
}

#[derive(Debug, PartialEq, Eq)]
enum Value<'a> {
    Double(Cow<'a, str>, Cow<'a, str>),
    Single(Cow<'a, str>),
}

impl<'a> Parser<'a> {
    fn new(s: Cow<'a, str>, span: Span) -> Parser<'a> {
        Parser { rest: s, span }
    }

    fn next(
        &mut self, diagnostics: &'a Diagnostics,
    ) -> Result<Option<Spanned<Value<'a>>>, Diagnostic> {
        if self.rest.is_empty() {
            return Ok(None);
        }

        let Spanned { value: key, span: key_span } = self.next_single(&['=', ','], diagnostics)?;

        let (delim, delim_span) = self.skip_delimiter();

        if let Some('=') = delim {
            let Spanned { value, span: value_span } = self.next_single(&[','], diagnostics)?;
            let span = key_span.map_end(|_| value_span.end);
            let res = Some(Spanned::new(Value::Double(key, value), span));

            let (delim, delim_span) = self.skip_delimiter();
            if !(delim == None || delim == Some(',')) {
                diagnostics.error(DiagnosticCode::InvalidCskvp)
                    .with_info_label(span, "in this key-value-pair")
                    .with_error_label(delim_span, "incorrect delimiter after value")
                    .with_note("use `,` to separate key-value-pairs")
                    .emit();
                self.rest = Cow::Borrowed("");
                return Ok(None);
            }
            Ok(res)
        } else {
            if !(delim == None || delim == Some(',')) {
                diagnostics.error(DiagnosticCode::InvalidCskv)
                    .with_info_label(key_span, "in this value")
                    .with_error_label(delim_span, "incorrect delimiter after value")
                    .with_note("use `,` to separate values")
                    .emit();
                self.rest = Cow::Borrowed("");
                return Ok(None);
            }
            Ok(Some(Spanned::new(Value::Single(key), key_span)))
        }
    }

    fn next_quoted(
        &mut self, diagnostics: &'a Diagnostics,
    ) -> Result<Spanned<Cow<'a, str>>, Diagnostic> {
        assert!(self.rest.starts_with('"'));
        macro_rules! err {
            ($e:ident) => {{
                diagnostics
                    .error(DiagnosticCode::InvalidDoubleQuotedString)
                    .with_error_label(self.span, "invalid double quoted string starts here")
                    .with_note(format!("cause: {}", $e))
                    .emit();
                return Err(Diagnostic);
            }};
        }

        let (content, quoted_string_len) = match &self.rest {
            Cow::Borrowed(rest) => match quoted_string::parse::<TestSpec>(rest) {
                Err((_, e)) => err!(e),
                Ok(parsed) => {
                    self.rest = Cow::Borrowed(parsed.tail);
                    let content =
                        quoted_string::to_content::<TestSpec>(parsed.quoted_string).unwrap();
                    (content, parsed.quoted_string.len())
                },
            },
            Cow::Owned(rest) => match quoted_string::parse::<TestSpec>(rest) {
                Err((_, e)) => err!(e),
                Ok(parsed) => {
                    let content = quoted_string::to_content::<TestSpec>(parsed.quoted_string)
                        .unwrap()
                        .into_owned();
                    let quoted_string_len = parsed.quoted_string.len();
                    let rest_len = self.rest.len();
                    let tail_len = parsed.tail.len();
                    self.rest.truncate_start(rest_len - tail_len);
                    (Cow::Owned(content), quoted_string_len)
                },
            },
        };
        let span = self.span.with_len(quoted_string_len);
        self.span.start += quoted_string_len;
        assert_eq!(self.span.end - self.span.start, self.rest.len());
        Ok(Spanned::new(content, span))
    }

    fn next_unquoted(&mut self, delimiters: &[char]) -> Spanned<Cow<'a, str>> {
        let idx = delimiters
            .iter()
            .cloned()
            .filter_map(|delim| self.rest.find(delim))
            .min()
            .unwrap_or(self.rest.len());
        let rest = self.rest.split_off(idx);
        let mut val = mem::replace(&mut self.rest, rest);
        let self_span = self.span;
        self.span.start += idx;

        let len = val.len();
        val.trim_start_inplace();
        let trimmed_start = len - val.len();
        val.trim_end_inplace();
        let trimmed_end = len - trimmed_start - val.len();
        let span = Span {
            file: self_span.file,
            start: self_span.start + trimmed_start,
            end: self_span.start + idx - trimmed_end,
        };
        Spanned::new(val, span)
    }

    fn next_single(
        &mut self, delimiters: &[char], diagnostics: &'a Diagnostics,
    ) -> Result<Spanned<Cow<'a, str>>, Diagnostic> {
        let len = self.rest.len();
        self.rest.trim_start_inplace();
        self.span.start += len - self.rest.len();
        if self.rest.starts_with('"') {
            self.next_quoted(diagnostics)
        } else {
            Ok(self.next_unquoted(delimiters))
        }
    }

    fn skip_delimiter(&mut self) -> (Option<char>, Span) {
        let len = self.rest.len();
        self.rest.trim_start_inplace();
        self.span.start += len - self.rest.len();
        let delim = self.rest.chars().next();
        let mut span = self.span.with_len(0);
        if delim.is_some() {
            let delim_len = delim.unwrap().len_utf8();
            self.rest.truncate_start(delim_len);
            self.span.start += delim_len;
            span.end += delim_len;
        }
        (delim, span)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use diagnostic::FileId;
    use crate::Diagnostics;

    #[test]
    fn test_parser() {
        let s = r#"foo, bar = " baz, \"qux\"", quux, corge="grault"#;
        let fileid = FileId::synthetic("test");
        let s_span = Span { file: fileid, start: 0, end: s.len() };
        let diagnostics = Diagnostics::new();
        diagnostics.add_synthetic_file("test", s.to_string());
        let mut parser = Parser::new(Cow::Borrowed(s), s_span);
        assert_eq!(
            parser.next(&diagnostics).unwrap(),
            Some(Spanned::new(Value::Single(Cow::Borrowed("foo")), Span::new(fileid, 0, 3)))
        );
        assert_eq!(
            parser.next(&diagnostics).unwrap(),
            Some(Spanned::new(
                Value::Double(Cow::Borrowed("bar"), Cow::Owned(r#" baz, "qux""#.to_string())),
                Span::new(fileid, 5, 26).into()
            ))
        );
        assert_eq!(
            parser.next(&diagnostics).unwrap(),
            Some(Spanned::new(Value::Single(Cow::Borrowed("quux")), Span::new(fileid, 28, 32)))
        );
        assert_eq!(parser.next(&diagnostics), Err(Diagnostic));
    }

    #[test]
    fn test_quoted_string() {
        let parsed = quoted_string::parse::<TestSpec>(r#""foo = \"bar\"" tail"#).unwrap();
        assert_eq!(parsed.quoted_string, r#""foo = \"bar\"""#);
        assert_eq!(parsed.tail, " tail");
        let content = quoted_string::to_content::<TestSpec>(parsed.quoted_string).unwrap();
        let expected: Cow<'_, str> = Cow::Owned(r#"foo = "bar""#.to_string());
        assert_eq!(content, expected);
        let content = quoted_string::to_content::<TestSpec>(r#""foo = bar""#).unwrap();
        assert_eq!(content, Cow::Borrowed("foo = bar"));
    }
}
