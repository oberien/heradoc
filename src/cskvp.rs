use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;
use std::mem;

use quoted_string::test_utils::TestSpec;
use single::{self, Single};

use crate::ext::{CowExt, VecExt};
use crate::diagnostics::Diagnostics;

struct Diagnostic;

#[derive(Debug)]
pub struct Cskvp<'a> {
    diagnostics: Option<Diagnostics<'a>>,
    range: Range<usize>,
    label: Option<(Cow<'a, str>, Range<usize>)>,
    caption: Option<(Cow<'a, str>, Range<usize>)>,
    figure: Option<(bool, Range<usize>)>,
    single: Vec<(Cow<'a, str>, Range<usize>)>,
    double: HashMap<Cow<'a, str>, (Cow<'a, str>, Range<usize>)>,
}

impl<'a> Default for Cskvp<'a> {
    fn default() -> Self {
        Cskvp {
            diagnostics: None,
            range: Range { start: 0, end: 0 },
            label: None,
            caption: None,
            figure: None,
            single: Vec::new(),
            double: HashMap::new(),
        }
    }
}

impl<'a> Cskvp<'a> {
    pub fn new(s: Cow<'a, str>, range: Range<usize>, content_range: Range<usize>, mut diagnostics: Diagnostics<'a>) -> Cskvp<'a> {
        let mut parser = Parser::new(s, content_range);

        let mut single = Vec::new();
        let mut double = HashMap::new();
        let mut label: Option<(_, Range<usize>)> = None;
        while let Ok(Some((value, range))) = parser.next(&mut diagnostics) {
            match value {
                Value::Double(key, value) => {
                    double.insert(key, (value, range));
                },
                Value::Single(mut value) => {
                    if value.starts_with('#') {
                        if label.is_some() {
                            diagnostics
                                .warning("found two labels")
                                .with_info_section(&label.as_ref().unwrap().1, "first label defined here")
                                .with_info_section(&range, "second label defined here")
                                .note("using the last")
                                .emit();
                        }
                        value.truncate_start(1);
                        label = Some((value, range));
                    } else {
                        single.push((value, range));
                    }
                },
            }
        }

        let figure_double = double.remove("figure").and_then(|(s, range)| match s.parse() {
            Ok(val) => Some((val, range)),
            Err(_) => {
                diagnostics
                    .error("cannot parse figure value")
                    .with_error_section(&range, "defined here")
                    .note("only `true` and `false` are allowed")
                    .emit();
                None
            },
        });
        let figure = single.remove_element(&"figure").map(|(_, range)| (true, range));
        let nofigure = single.remove_element(&"nofigure").map(|(_, range)| (false, range));

        let figures = [figure_double, figure, nofigure];
        let figure = match figures.iter().cloned().flatten().single() {
            Ok(val) => Some(val),
            Err(single::Error::NoElements) => None,
            Err(single::Error::MultipleElements) => {
                let mut diag = Some(diagnostics
                    .error("found multiple figure specifiers"));
                for (_, range) in figures.iter().cloned().flatten() {
                    diag = Some(diag.take().unwrap().with_info_section(&range, "one defined here"));
                }
                diag.unwrap()
                    .note("only one of `figure=true`, `figure=false`, `figure` and `nofigure` is allowed")
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

    pub fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub fn has_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn take_label(&mut self) -> Option<(Cow<'a, str>, Range<usize>)> {
        self.label.take()
    }

    pub fn take_figure(&mut self) -> Option<(bool, Range<usize>)> {
        self.figure.take()
    }

    pub fn take_caption(&mut self) -> Option<(Cow<'a, str>, Range<usize>)> {
        self.caption.take()
    }

    pub fn take_double(&mut self, key: &str) -> Option<(Cow<'a, str>, Range<usize>)> {
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
        let range = &self.range;
        let mut diag = self.diagnostics
            .as_mut()
            .map(|d| d
                .warning("in element config")
                .with_error_section(range, "")
            );
        if let Some((label, range)) = self.label.as_ref() {
            diag = diag.map(|d| {
                d.warning(format!("label ignored: {}", label))
                    .with_info_section(range, "label defined here")
            });
            has_warning = true;
        }
        if let Some((figure, range)) = self.figure.as_ref() {
            diag = diag.map(|d| {
                d.warning(format!("figure config ignored: {}", figure))
                    .with_info_section(range, "figure config defined here")
            });
            has_warning = true;
        }
        if let Some((caption, range)) = self.caption.as_ref() {
            diag = diag.map(|d| {
                d.warning(format!("caption ignored: {}", caption))
                    .with_info_section(range, "caption defined here")
            });
            has_warning = true;
        }
        for (k, (v, range)) in self.double.drain() {
            diag = diag.map(|d| {
                d.warning(format!("unknown attribute `{}={}`", k, v))
                    .with_info_section(&range, "attribute defined here")
            });
            has_warning = true;
        }
        for (attr, range) in self.single.drain(..) {
            diag = diag.map(|d| {
                d.warning(format!("unknown attribute `{}`", attr))
                    .with_info_section(&range, "attribute defined here")
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

struct Parser<'a> {
    rest: Cow<'a, str>,
    range: Range<usize>,
}

#[derive(Debug, PartialEq, Eq)]
enum Value<'a> {
    Double(Cow<'a, str>, Cow<'a, str>),
    Single(Cow<'a, str>),
}

impl<'a> Parser<'a> {
    fn new(s: Cow<'a, str>, range: Range<usize>) -> Parser<'a> {
        Parser { rest: s, range }
    }

    fn next(&mut self, diagnostics: &mut Diagnostics<'a>) -> Result<Option<(Value<'a>, Range<usize>)>, Diagnostic> {
        if self.rest.is_empty() {
            return Ok(None);
        }

        let (key, key_range) = self.next_single(&['=', ','], diagnostics)?;

        let delim = self.skip_delimiter();

        if let Some('=') = delim {
            let (val, val_range) = self.next_single(&[','], diagnostics)?;
            let range = Range { start: key_range.start, end: val_range.end };
            let res = Some((Value::Double(key, val), range));

            let delim = self.skip_delimiter();
            assert!(delim == Some(',') || delim == None, "invalid comma seperated key value pair");
            Ok(res)
        } else {
            assert!(delim == Some(',') || delim == None, "invalid comma seperated value");
            Ok(Some((Value::Single(key), key_range)))
        }
    }

    fn next_quoted(&mut self, diagnostics: &mut Diagnostics<'a>) -> Result<(Cow<'a, str>, Range<usize>), Diagnostic> {
        assert!(self.rest.starts_with('"'));
        macro_rules! err {
            ($e:ident) => {{
                diagnostics
                    .error("invalid double-quoted string")
                    .with_error_section(&self.range, "double quoted string starts here")
                    .note(format!("cause: {}", $e))
                    .emit();
                return Err(Diagnostic);
            }}
        }

        let (content, quoted_string_len) = match &self.rest {
            Cow::Borrowed(rest) => match quoted_string::parse::<TestSpec>(rest) {
                Err((_, e)) => err!(e),
                Ok(parsed) => {
                    self.rest = Cow::Borrowed(parsed.tail);
                    let content = quoted_string::to_content::<TestSpec>(parsed.quoted_string).unwrap();
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
        let range = Range { start: self.range.start, end: self.range.start + quoted_string_len };
        self.range.start += quoted_string_len;
        assert_eq!(self.range.end - self.range.start, self.rest.len());
        Ok((content, range))
    }

    fn next_unquoted(&mut self, delimiters: &[char]) -> (Cow<'a, str>, Range<usize>) {
        let idx = delimiters
            .iter()
            .cloned()
            .filter_map(|delim| self.rest.find(delim))
            .min()
            .unwrap_or(self.rest.len());
        let rest = self.rest.split_off(idx);
        let mut val = mem::replace(&mut self.rest, rest);
        let self_range = self.range.clone();
        self.range.start += idx;

        let len = val.len();
        val.trim_start_inplace();
        let trimmed_start = len - val.len();
        val.trim_end_inplace();
        let trimmed_end = len - trimmed_start - val.len();
        let range = Range {
            start: self_range.start + trimmed_start,
            end: self_range.start + idx - trimmed_end
        };
        (val, range)
    }

    fn next_single(
        &mut self, delimiters: &[char], diagnostics: &mut Diagnostics<'a>
    ) -> Result<(Cow<'a, str>, Range<usize>), Diagnostic> {
        let len = self.rest.len();
        self.rest.trim_start_inplace();
        self.range.start += len - self.rest.len();
        if self.rest.starts_with('"') {
            self.next_quoted(diagnostics)
        } else {
            Ok(self.next_unquoted(delimiters))
        }
    }

    fn skip_delimiter(&mut self) -> Option<char> {
        let len = self.rest.len();
        self.rest.trim_start_inplace();
        self.range.start += len - self.rest.len();
        let delim = self.rest.chars().next();
        if delim.is_some() {
            self.rest.truncate_start(1);
            self.range.start += 1;
        }
        delim
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::diagnostics::Input;

    #[test]
    fn test_parser() {
        let mut diagnostics = Diagnostics::new("", Input::Stdin);
        let s = r#"foo, bar = " baz, \"qux\"", quux"#;
        let s_range = Range { start: 0, end: s.len() };
        let mut parser = Parser::new(Cow::Borrowed(s), s_range);
        assert_eq!(parser.next(&mut diagnostics), Some(Value::Single(Cow::Borrowed("foo"))));
        assert_eq!(
            parser.next(&mut diagnostics),
            Some(Value::Double(Cow::Borrowed("bar"), Cow::Owned(r#" baz, "qux""#.to_string())))
        );
        assert_eq!(parser.next(&mut diagnostics), Some(Value::Single(Cow::Borrowed("quux"))));
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
