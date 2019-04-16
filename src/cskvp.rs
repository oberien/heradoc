use std::borrow::Cow;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use quoted_string::test_utils::TestSpec;
use single::{self, Single};

use crate::diagnostics::Diagnostics;
use crate::ext::{CowExt, VecExt};
use crate::frontend::range::{SourceRange, WithRange};

#[derive(Debug, PartialEq, Eq)]
struct Diagnostic;

#[derive(Debug)]
pub struct Cskvp<'a> {
    diagnostics: Option<Arc<Diagnostics<'a>>>,
    range: SourceRange,
    label: Option<WithRange<Cow<'a, str>>>,
    caption: Option<WithRange<Cow<'a, str>>>,
    figure: Option<WithRange<bool>>,
    single: Vec<WithRange<Cow<'a, str>>>,
    double: HashMap<Cow<'a, str>, WithRange<Cow<'a, str>>>,
}

impl<'a> Default for Cskvp<'a> {
    fn default() -> Self {
        Cskvp {
            diagnostics: None,
            range: SourceRange { start: 0, end: 0 },
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
        s: Cow<'a, str>, range: SourceRange, content_range: SourceRange,
        diagnostics: Arc<Diagnostics<'a>>,
    ) -> Cskvp<'a> {
        // The content_range may involve unescaped escaped sequences like `\\`, which will only have
        // a length of 1 here. Thus we need to trim the range for the parser.
        // TODO: fix for diagnostics when unescaping is involved. See #
        let parser_range = SourceRange {
            start: content_range.start, end: content_range.start + s.len() };
        let mut parser = Parser::new(s, parser_range);

        let mut single = Vec::new();
        let mut double = HashMap::new();
        let mut label: Option<WithRange<_>> = None;
        while let Ok(Some(WithRange(value, range))) = parser.next(&diagnostics) {
            match value {
                Value::Double(key, value) => {
                    double.insert(key, WithRange(value, range));
                },
                Value::Single(mut value) => {
                    if value.starts_with('#') {
                        if label.is_some() {
                            diagnostics
                                .warning("found two labels")
                                .with_info_section(
                                    label.as_ref().unwrap().1,
                                    "first label defined here",
                                )
                                .with_info_section(range, "second label defined here")
                                .note("using the last")
                                .emit();
                        }
                        value.truncate_start(1);
                        label = Some(WithRange(value, range));
                    } else {
                        single.push(WithRange(value, range));
                    }
                },
            }
        }

        let figure_double = double.remove("figure").and_then(|WithRange(s, range)| match s.parse() {
            Ok(val) => Some(WithRange(val, range)),
            Err(_) => {
                diagnostics
                    .error("cannot parse figure value")
                    .with_error_section(range, "defined here")
                    .note("only `true` and `false` are allowed")
                    .emit();
                None
            },
        });
        let figure = single.remove_element(&"figure").map(|WithRange(_, range)| WithRange(true, range));
        let nofigure = single.remove_element(&"nofigure").map(|WithRange(_, range)| WithRange(false, range));

        let figures = [figure_double, figure, nofigure];
        let figure = match figures.iter().cloned().flatten().single() {
            Ok(val) => Some(val),
            Err(single::Error::NoElements) => None,
            Err(single::Error::MultipleElements) => {
                let mut diag = Some(diagnostics.error("found multiple figure specifiers"));
                for WithRange(_, range) in figures.iter().cloned().flatten() {
                    diag = Some(diag.take().unwrap().with_info_section(range, "one defined here"));
                }
                diag.unwrap()
                    .note(
                        "only one of `figure=true`, `figure=false`, `figure` and `nofigure` is \
                         allowed",
                    )
                    .emit();
                None
            },
        };

        Cskvp {
            diagnostics: Some(diagnostics),
            range,
            label,
            caption: double.remove("caption"),
            figure,
            single,
            double,
        }
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn take_label(&mut self) -> Option<WithRange<Cow<'a, str>>> {
        self.label.take()
    }

    pub fn take_figure(&mut self) -> Option<WithRange<bool>> {
        self.figure.take()
    }

    pub fn take_caption(&mut self) -> Option<WithRange<Cow<'a, str>>> {
        self.caption.take()
    }

    pub fn take_double(&mut self, key: &str) -> Option<WithRange<Cow<'a, str>>> {
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
        let range = self.range;
        let mut diag = self
            .diagnostics
            .as_mut()
            .map(|d| d.warning("unknown attributes in element config")
                .with_info_section(range, "in this element config"));
        if let Some(WithRange(label, range)) = self.label.take() {
            diag = diag.map(|d| {
                d.warning(format!("label ignored: {}", label))
                    .with_info_section(range, "label defined here")
            });
            has_warning = true;
        }
        if let Some(WithRange(figure, range)) = self.figure.take() {
            diag = diag.map(|d| {
                d.warning(format!("figure config ignored: {}", figure))
                    .with_info_section(range, "figure config defined here")
            });
            has_warning = true;
        }
        if let Some(WithRange(caption, range)) = self.caption.take() {
            diag = diag.map(|d| {
                d.warning(format!("caption ignored: {}", caption))
                    .with_info_section(range, "caption defined here")
            });
            has_warning = true;
        }
        for (k, WithRange(v, range)) in self.double.drain() {
            diag = diag.map(|d| {
                d.warning(format!("unknown attribute `{}={}`", k, v))
                    .with_info_section(range, "attribute defined here")
            });
            has_warning = true;
        }
        for WithRange(attr, range) in self.single.drain(..) {
            diag = diag.map(|d| {
                d.warning(format!("unknown attribute `{}`", attr))
                    .with_info_section(range, "attribute defined here")
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
    range: SourceRange,
}

#[derive(Debug, PartialEq, Eq)]
enum Value<'a> {
    Double(Cow<'a, str>, Cow<'a, str>),
    Single(Cow<'a, str>),
}

impl<'a> Parser<'a> {
    fn new(s: Cow<'a, str>, range: SourceRange) -> Parser<'a> {
        Parser { rest: s, range }
    }

    fn next(
        &mut self, diagnostics: &Diagnostics<'a>,
    ) -> Result<Option<WithRange<Value<'a>>>, Diagnostic> {
        if self.rest.is_empty() {
            return Ok(None);
        }

        let WithRange(key, key_range) = self.next_single(&['=', ','], diagnostics)?;

        let (delim, delim_range) = self.skip_delimiter();

        if let Some('=') = delim {
            let WithRange(val, val_range) = self.next_single(&[','], diagnostics)?;
            let range = SourceRange { start: key_range.start, end: val_range.end };
            let res = Some(WithRange(Value::Double(key, val), range));

            let (delim, delim_range) = self.skip_delimiter();
            if !(delim == None || delim == Some(',')) {
                diagnostics.error("invalid comma seperated key-value-pair")
                    .with_info_section(range, "in this key-value-pair")
                    .with_error_section(delim_range, "incorrect delimiter after value")
                    .help("use `,` to separate key-value-pairs")
                    .emit();
                self.rest = Cow::Borrowed("");
                return Ok(None);
            }
            Ok(res)
        } else {
            if !(delim == None || delim == Some(',')) {
                diagnostics.error("invalid comma seperated value")
                    .with_info_section(key_range, "in this value")
                    .with_error_section(delim_range, "incorrect delimiter after value")
                    .help("use `,` to separate values")
                    .emit();
                self.rest = Cow::Borrowed("");
                return Ok(None);
            }
            Ok(Some(WithRange(Value::Single(key), key_range)))
        }
    }

    fn next_quoted(
        &mut self, diagnostics: &Diagnostics<'a>,
    ) -> Result<WithRange<Cow<'a, str>>, Diagnostic> {
        assert!(self.rest.starts_with('"'));
        macro_rules! err {
            ($e:ident) => {{
                diagnostics
                    .error("invalid double-quoted string")
                    .with_error_section(self.range, "double quoted string starts here")
                    .note(format!("cause: {}", $e))
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
                    self.rest.truncate_start(self.rest.len() - parsed.tail.len());
                    (Cow::Owned(content), quoted_string_len)
                },
            },
        };
        let range = SourceRange { start: self.range.start, end: self.range.start + quoted_string_len };
        self.range.start += quoted_string_len;
        assert_eq!(self.range.end - self.range.start, self.rest.len());
        Ok(WithRange(content, range))
    }

    fn next_unquoted(&mut self, delimiters: &[char]) -> WithRange<Cow<'a, str>> {
        let idx = delimiters
            .iter()
            .cloned()
            .filter_map(|delim| self.rest.find(delim))
            .min()
            .unwrap_or(self.rest.len());
        let rest = self.rest.split_off(idx);
        let mut val = mem::replace(&mut self.rest, rest);
        let self_range = self.range;
        self.range.start += idx;

        let len = val.len();
        val.trim_start_inplace();
        let trimmed_start = len - val.len();
        val.trim_end_inplace();
        let trimmed_end = len - trimmed_start - val.len();
        let range = SourceRange {
            start: self_range.start + trimmed_start,
            end: self_range.start + idx - trimmed_end,
        };
        WithRange(val, range)
    }

    fn next_single(
        &mut self, delimiters: &[char], diagnostics: &Diagnostics<'a>,
    ) -> Result<WithRange<Cow<'a, str>>, Diagnostic> {
        let len = self.rest.len();
        self.rest.trim_start_inplace();
        self.range.start += len - self.rest.len();
        if self.rest.starts_with('"') {
            self.next_quoted(diagnostics)
        } else {
            Ok(self.next_unquoted(delimiters))
        }
    }

    fn skip_delimiter(&mut self) -> (Option<char>, SourceRange) {
        let len = self.rest.len();
        self.rest.trim_start_inplace();
        self.range.start += len - self.rest.len();
        let delim = self.rest.chars().next();
        let mut range = SourceRange { start: self.range.start, end: self.range.start };
        if delim.is_some() {
            let delim_len = delim.unwrap().len_utf8();
            self.rest.truncate_start(delim_len);
            self.range.start += delim_len;
            range.end += delim_len;
        }
        (delim, range)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Mutex;
    use codespan_reporting::termcolor::{ColorChoice, StandardStream};
    use crate::diagnostics::Input;

    #[test]
    fn test_parser() {
        let s = r#"foo, bar = " baz, \"qux\"", quux, corge="grault"#;
        let s_range = SourceRange { start: 0, end: s.len() };
        let diagnostics = Diagnostics::new(s, Input::Stdin, Arc::new(Mutex::new(StandardStream::stderr(ColorChoice::Auto))));
        let mut parser = Parser::new(Cow::Borrowed(s), s_range);
        assert_eq!(
            parser.next(&diagnostics).unwrap(),
            Some(WithRange(Value::Single(Cow::Borrowed("foo")), (0..3).into()))
        );
        assert_eq!(
            parser.next(&diagnostics).unwrap(),
            Some(WithRange(
                Value::Double(Cow::Borrowed("bar"), Cow::Owned(r#" baz, "qux""#.to_string())),
                (5..26).into()
            ))
        );
        assert_eq!(
            parser.next(&diagnostics).unwrap(),
            Some(WithRange(Value::Single(Cow::Borrowed("quux")), (28..32).into()))
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
